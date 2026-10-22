pub mod scanners;
pub mod theme;
pub mod types;
pub mod ui;

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::fs;
use std::io;
use std::path::PathBuf;
use std::process::Command;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use types::*;

const MENU_OPTIONS: &[&str] = &[
    "Select and Clean Caches / Containers",
    "Largest Files Report (~/Documents, Downloads, Dumps)",
    "Delete Local Snapshots (Time Machine via Sudo)",
    "App Uninstall Suggestions (Apps & Bulky Casks)",
    "Hugging Face Models & Cache",
    "Ollama Models",
    "Browser Caches & Data",
    "Developer Tools Caches (Multi-select)",
    "Nerd Fonts Optimizer (~/Library/Fonts)",
    "Application Support Caches & Data",
    "Docker Cleanup & Caches",
    "Open macOS Storage Settings",
    "Exit",
];

const SPINNER_CHARS: &[char] = &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];

enum AppEvent {
    ScanFinished(Vec<CleanupItem>),
    ReportFinished(Vec<scanners::documents::FileInfo>),
    AppsFinished(Vec<scanners::apps::AppSuggestion>),
    HfFinished(Vec<scanners::huggingface::HFItem>),
    OllamaFinished(Vec<scanners::ollama::OllamaItem>),
    BrowsersFinished(Vec<scanners::browsers::BrowserItem>),
    DevFinished(Vec<scanners::devtools::DevItem>),
    FontsFinished(Vec<scanners::fonts::FontItem>),
    AppSupportFinished(Vec<scanners::appsupport::AppSupportItem>),
    DockerFinished(Vec<scanners::docker::DockerItem>),
    CleanItemDone(usize, u64),
    CleanProgress(usize, usize, u64),
    CleanAllDone(u64),
}

// Custom clean helpers
fn clean_spotify(dry_run: bool) -> (String, Result<(), String>) {
    let home = dirs::home_dir().unwrap_or_default();
    let paths = [
        home.join("Library/Caches/com.spotify.client"),
        home.join("Library/Application Support/Spotify/PersistentCache"),
    ];
    for p in paths {
        let _ = scanners::move_to_trash(p, dry_run);
    }
    ("Spotify caches cleaned".to_string(), Ok(()))
}

fn clean_slack(dry_run: bool) -> (String, Result<(), String>) {
    let home = dirs::home_dir().unwrap_or_default();
    let paths = [
        home.join("Library/Application Support/Slack/Cache"),
        home.join("Library/Application Support/Slack/Code Cache"),
        home.join("Library/Caches/com.tinyspeck.slackmacgap"),
    ];
    for p in paths {
        let _ = scanners::move_to_trash(p, dry_run);
    }
    ("Slack caches cleaned".to_string(), Ok(()))
}

fn clean_discord(dry_run: bool) -> (String, Result<(), String>) {
    let home = dirs::home_dir().unwrap_or_default();
    let paths = [
        home.join("Library/Application Support/discord/Cache"),
        home.join("Library/Application Support/discord/Code Cache"),
        home.join("Library/Caches/com.hnc.Discord"),
    ];
    for p in paths {
        let _ = scanners::move_to_trash(p, dry_run);
    }
    ("Discord caches cleaned".to_string(), Ok(()))
}

fn clean_user_logs(dry_run: bool) -> (String, Result<(), String>) {
    let home = dirs::home_dir().unwrap_or_default();
    let logs_dir = home.join("Library/Logs");
    if let Ok(entries) = fs::read_dir(&logs_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let name = entry.file_name().to_string_lossy().to_string();
            if name == "DiagnosticReports" {
                continue;
            }
            let _ = scanners::move_to_trash(entry.path(), dry_run);
        }
    }
    ("User logs cleaned".to_string(), Ok(()))
}

fn clean_orphan_containers(dry_run: bool) -> (String, Result<(), String>) {
    let (msg, res) = scanners::containers::clean_orphan_containers(dry_run);
    (msg, res.map_err(|e| e.to_string()))
}

fn clean_thunderbird(dry_run: bool) -> (String, Result<(), String>) {
    let home = dirs::home_dir().unwrap_or_default();
    let tb_cache = home.join("Library/Caches/Thunderbird");
    if tb_cache.exists() {
        let _ = scanners::move_to_trash(tb_cache, dry_run);
    }
    ("Thunderbird cache cleared".to_string(), Ok(()))
}

fn initial_items() -> Vec<CleanupItem> {
    let home = dirs::home_dir().unwrap_or_default();
    vec![
        CleanupItem {
            id: "snapshots".to_string(),
            name: "Time Machine Local Snapshots".to_string(),
            path: PathBuf::from("tm_snapshots"),
            size: 0,
            scanned: false,
            selected: true,
            custom_clean: Some(|dry_run| {
                if !dry_run {
                    let _ = Command::new("tmutil").args(["deletelocalsnapshots", "/"]).output();
                }
                ("Snapshots cleared".to_string(), Ok(()))
            }),
        },
        CleanupItem {
            id: "spotify".to_string(),
            name: "Spotify Cache".to_string(),
            path: home.join("Library/Caches/com.spotify.client"),
            size: 0,
            scanned: false,
            selected: true,
            custom_clean: Some(clean_spotify),
        },
        CleanupItem {
            id: "slack".to_string(),
            name: "Slack Cache".to_string(),
            path: home.join("Library/Application Support/Slack/Cache"),
            size: 0,
            scanned: false,
            selected: true,
            custom_clean: Some(clean_slack),
        },
        CleanupItem {
            id: "discord".to_string(),
            name: "Discord Cache".to_string(),
            path: home.join("Library/Application Support/discord/Cache"),
            size: 0,
            scanned: false,
            selected: true,
            custom_clean: Some(clean_discord),
        },
        CleanupItem {
            id: "telegram".to_string(),
            name: "Telegram Cache & Media".to_string(),
            path: home.join("Library/Group Containers/5U85Y5W795.ru.keepcoder.Telegram"),
            size: 0,
            scanned: false,
            selected: true,
            custom_clean: None,
        },
        CleanupItem {
            id: "mail_downloads".to_string(),
            name: "Apple Mail Downloads & Attachments".to_string(),
            path: home.join("Library/Containers/com.apple.mail/Data/Library/Mail Downloads"),
            size: 0,
            scanned: false,
            selected: true,
            custom_clean: None,
        },
        CleanupItem {
            id: "diagnostic_reports".to_string(),
            name: "macOS Diagnostic / Crash Reports".to_string(),
            path: home.join("Library/Logs/DiagnosticReports"),
            size: 0,
            scanned: false,
            selected: true,
            custom_clean: None,
        },
        CleanupItem {
            id: "user_logs".to_string(),
            name: "User Cache Logs".to_string(),
            path: home.join("Library/Logs"),
            size: 0,
            scanned: false,
            selected: true,
            custom_clean: Some(clean_user_logs),
        },
        CleanupItem {
            id: "containers".to_string(),
            name: "Orphan Containers (Third-Party)".to_string(),
            path: PathBuf::from("containers_scan"),
            size: 0,
            scanned: false,
            selected: true,
            custom_clean: Some(clean_orphan_containers),
        },
        CleanupItem {
            id: "thunderbird".to_string(),
            name: "Thunderbird Email Caches".to_string(),
            path: home.join("Library/Caches/Thunderbird"),
            size: 0,
            scanned: false,
            selected: true,
            custom_clean: Some(clean_thunderbird),
        },
    ]
}

fn get_spotify_size() -> u64 {
    let home = dirs::home_dir().unwrap_or_default();
    scanners::dir_size(home.join("Library/Caches/com.spotify.client"))
        + scanners::dir_size(home.join("Library/Application Support/Spotify/PersistentCache"))
}

fn get_slack_size() -> u64 {
    let home = dirs::home_dir().unwrap_or_default();
    scanners::dir_size(home.join("Library/Application Support/Slack/Cache"))
        + scanners::dir_size(home.join("Library/Application Support/Slack/Code Cache"))
        + scanners::dir_size(home.join("Library/Caches/com.tinyspeck.slackmacgap"))
}

fn get_discord_size() -> u64 {
    let home = dirs::home_dir().unwrap_or_default();
    scanners::dir_size(home.join("Library/Application Support/discord/Cache"))
        + scanners::dir_size(home.join("Library/Application Support/discord/Code Cache"))
        + scanners::dir_size(home.join("Library/Caches/com.hnc.Discord"))
}

fn get_user_logs_size() -> u64 {
    let home = dirs::home_dir().unwrap_or_default();
    let logs_dir = home.join("Library/Logs");
    let diag_dir = home.join("Library/Logs/DiagnosticReports");
    let logs_size = scanners::dir_size(logs_dir);
    let diag_size = scanners::dir_size(diag_dir);
    logs_size.saturating_sub(diag_size)
}

fn scan_cleanup_items(mut items: Vec<CleanupItem>, tx: mpsc::Sender<AppEvent>) {
    thread::spawn(move || {
        let mut handles = Vec::new();
        for mut item in items.drain(..) {
            let handle = thread::spawn(move || {
                let size = if item.id == "containers" {
                    scanners::containers::get_orphan_containers_size()
                } else if item.id == "snapshots" {
                    let out = Command::new("tmutil")
                        .args(["listlocalsnapshots", "/"])
                        .output();
                    let count = match out {
                        Ok(o) => {
                            let text = String::from_utf8_lossy(&o.stdout);
                            text.lines().filter(|l| l.contains("com.apple.TimeMachine")).count()
                        }
                        Err(_) => 0,
                    };
                    (count as u64) * 1500 * 1024 * 1024
                } else if item.id == "brew" {
                    200 * 1024 * 1024
                } else if item.id == "spotify" {
                    get_spotify_size()
                } else if item.id == "slack" {
                    get_slack_size()
                } else if item.id == "discord" {
                    get_discord_size()
                } else if item.id == "user_logs" {
                    get_user_logs_size()
                } else {
                    scanners::dir_size(&item.path)
                };
                item.size = size;
                item.scanned = true;
                item
            });
            handles.push(handle);
        }

        let mut scanned_items = Vec::new();
        for h in handles {
            if let Ok(item) = h.join() {
                scanned_items.push(item);
            }
        }
        let _ = tx.send(AppEvent::ScanFinished(scanned_items));
    });
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let dry_run = args.iter().any(|a| a == "--dry-run" || a == "-n" || a == "--dry");

    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!("Cinder CLI is an interactive terminal utility written in Rust to inspect disk space, analyze caches, and safely clean accumulated system data on macOS using Ratatui and Crossterm.");
        println!("\nUsage: cinder [OPTIONS]\n");
        println!("Options:");
        println!("  -n, --dry-run, --dry   Run in simulation mode (does not delete or modify any files)");
        println!("  -h, --help             Print help information");
        return Ok(());
    }

    // Install panic hook to restore terminal on panic
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen, crossterm::cursor::Show);
        original_hook(panic_info);
    }));

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        crossterm::terminal::Clear(crossterm::terminal::ClearType::All),
        crossterm::cursor::MoveTo(0, 0)
    )?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let res = run_app(&mut terminal, dry_run).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error running Cinder CLI: {:?}", err);
    }

    Ok(())
}

async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    dry_run: bool,
) -> io::Result<()> {
    let (tx, rx) = mpsc::channel::<AppEvent>();

    let mut state = AppState::Scanning;
    let mut items = initial_items();
    let mut cursor = 0;
    let mut selection_page = 0;
    let mut menu_cursor = 0;
    let mut status_msg = String::new();

    let mut large_files: Vec<scanners::documents::FileInfo> = Vec::new();
    let mut report_cursor = 0;
    let mut report_page = 0;

    let mut app_suggestions: Vec<scanners::apps::AppSuggestion> = Vec::new();
    let mut app_cursor = 0;
    let mut app_page = 0;

    let mut hf_items: Vec<scanners::huggingface::HFItem> = Vec::new();
    let mut hf_cursor = 0;
    let mut hf_page = 0;

    let mut ollama_items: Vec<scanners::ollama::OllamaItem> = Vec::new();
    let mut ollama_cursor = 0;
    let mut ollama_page = 0;

    let mut browsers_items: Vec<scanners::browsers::BrowserItem> = Vec::new();
    let mut browsers_cursor = 0;
    let mut browsers_page = 0;

    let mut dev_items: Vec<scanners::devtools::DevItem> = Vec::new();
    let mut dev_cursor = 0;
    let mut dev_page = 0;

    let mut font_items: Vec<scanners::fonts::FontItem> = Vec::new();
    let mut font_cursor = 0;
    let mut font_page = 0;

    let mut app_support_items: Vec<scanners::appsupport::AppSupportItem> = Vec::new();
    let mut app_support_cursor = 0;
    let mut app_support_page = 0;

    let mut docker_items: Vec<scanners::docker::DockerItem> = Vec::new();
    let mut docker_cursor = 0;
    let mut docker_page = 0;

    let mut previous_state = AppState::MainMenu;
    let mut confirm_prompt = String::new();
    let mut on_confirm_action: Option<Box<dyn FnOnce() -> Option<AppEvent> + Send>> = None;

    let mut freed_size: u64 = 0;
    let mut items_to_clean: Vec<usize> = Vec::new();
    let mut current_clean_idx: usize;
    let mut clean_percent = 0.0;

    let mut items_per_page = 10;
    let mut spinner_idx = 0;
    let mut last_tick = Instant::now();

    // Start initial scan
    scan_cleanup_items(items.clone(), tx.clone());

    loop {
        // Handle background async events
        while let Ok(event) = rx.try_recv() {
            match event {
                AppEvent::ScanFinished(new_items) => {
                    items = new_items;
                    state = AppState::MainMenu;
                }
                AppEvent::ReportFinished(files) => {
                    large_files = files;
                    report_page = 0;
                    report_cursor = 0;
                    state = AppState::DocumentsReport;
                }
                AppEvent::AppsFinished(apps) => {
                    app_suggestions = apps;
                    app_page = 0;
                    app_cursor = 0;
                    state = AppState::AppsSuggestions;
                }
                AppEvent::HfFinished(hf) => {
                    hf_items = hf;
                    hf_page = 0;
                    hf_cursor = 0;
                    state = AppState::HuggingFace;
                }
                AppEvent::OllamaFinished(ollama) => {
                    ollama_items = ollama;
                    ollama_page = 0;
                    ollama_cursor = 0;
                    state = AppState::Ollama;
                }
                AppEvent::BrowsersFinished(browsers) => {
                    browsers_items = browsers;
                    browsers_page = 0;
                    browsers_cursor = 0;
                    state = AppState::Browsers;
                }
                AppEvent::DevFinished(devs) => {
                    dev_items = devs;
                    dev_page = 0;
                    dev_cursor = 0;
                    state = AppState::DevTools;
                }
                AppEvent::FontsFinished(fonts) => {
                    font_items = fonts;
                    font_page = 0;
                    font_cursor = 0;
                    state = AppState::Fonts;
                }
                AppEvent::AppSupportFinished(app_support) => {
                    app_support_items = app_support;
                    app_support_page = 0;
                    app_support_cursor = 0;
                    state = AppState::AppSupport;
                }
                AppEvent::DockerFinished(dockers) => {
                    docker_items = dockers;
                    docker_page = 0;
                    docker_cursor = 0;
                    state = AppState::Docker;
                }
                AppEvent::CleanItemDone(idx, size_freed) => {
                    freed_size += size_freed;
                    current_clean_idx = idx + 1;
                    if !items_to_clean.is_empty() {
                        clean_percent = (current_clean_idx as f64) / (items_to_clean.len() as f64);
                    }
                    if current_clean_idx >= items_to_clean.len() {
                        state = AppState::Finished;
                    } else {
                        // Launch next clean task
                        let item_idx = items_to_clean[current_clean_idx];
                        let item = items[item_idx].clone();
                        let tx_clone = tx.clone();
                        let curr_idx = current_clean_idx;
                        thread::spawn(move || {
                            let freed = if let Some(clean_fn) = item.custom_clean {
                                let (_, res) = clean_fn(dry_run);
                                if res.is_ok() { item.size } else { 0 }
                            } else {
                                match scanners::move_to_trash(&item.path, dry_run) {
                                    Ok(_) => item.size,
                                    Err(_) => 0,
                                }
                            };
                            thread::sleep(Duration::from_millis(300));
                            let _ = tx_clone.send(AppEvent::CleanItemDone(curr_idx, freed));
                        });
                    }
                }
                AppEvent::CleanProgress(current, total, current_freed) => {
                    freed_size += current_freed;
                    if total > 0 {
                        clean_percent = (current as f64) / (total as f64);
                    }
                    state = AppState::Cleaning;
                }
                AppEvent::CleanAllDone(total_freed) => {
                    freed_size = total_freed;
                    clean_percent = 1.0;
                    state = AppState::Finished;
                }
            }
        }

        // Advance spinner animation
        if last_tick.elapsed() >= Duration::from_millis(100) {
            spinner_idx = (spinner_idx + 1) % SPINNER_CHARS.len();
            last_tick = Instant::now();
        }

        // Dynamically compute items_per_page based on available terminal height
        if let Ok(size) = terminal.size() {
            // Reserve ~10 lines for header banner, borders, page indicator, summary box, and help footer
            let available = size.height.saturating_sub(10) as usize;
            items_per_page = available.clamp(3, 20);

            // Clamp page and cursor across all views to prevent out-of-bounds index/slice crashes
            let clamp_view = |len: usize, page: &mut usize, cur: &mut usize| {
                if len == 0 {
                    *page = 0;
                    *cur = 0;
                } else {
                    let total_pages = len.div_ceil(items_per_page);
                    if *page >= total_pages {
                        *page = total_pages.saturating_sub(1);
                    }
                    let start = *page * items_per_page;
                    let end = (start + items_per_page).min(len);
                    let page_size = end.saturating_sub(start);
                    if page_size > 0 && *cur >= page_size {
                        *cur = page_size.saturating_sub(1);
                    }
                }
            };

            clamp_view(items.len(), &mut selection_page, &mut cursor);
            clamp_view(large_files.len(), &mut report_page, &mut report_cursor);
            clamp_view(app_suggestions.len(), &mut app_page, &mut app_cursor);
            clamp_view(hf_items.len(), &mut hf_page, &mut hf_cursor);
            clamp_view(ollama_items.len(), &mut ollama_page, &mut ollama_cursor);
            clamp_view(browsers_items.len(), &mut browsers_page, &mut browsers_cursor);
            clamp_view(dev_items.len(), &mut dev_page, &mut dev_cursor);
            clamp_view(font_items.len(), &mut font_page, &mut font_cursor);
            clamp_view(app_support_items.len(), &mut app_support_page, &mut app_support_cursor);
            clamp_view(docker_items.len(), &mut docker_page, &mut docker_cursor);
        }

        // Draw terminal UI
        terminal.draw(|f| {
            ui::render_ui(
                f,
                state,
                &items,
                cursor,
                selection_page,
                menu_cursor,
                MENU_OPTIONS,
                &status_msg,
                &large_files,
                report_cursor,
                report_page,
                &app_suggestions,
                app_cursor,
                app_page,
                &hf_items,
                hf_cursor,
                hf_page,
                &ollama_items,
                ollama_cursor,
                ollama_page,
                &browsers_items,
                browsers_cursor,
                browsers_page,
                &dev_items,
                dev_cursor,
                dev_page,
                &font_items,
                font_cursor,
                font_page,
                &app_support_items,
                app_support_cursor,
                app_support_page,
                &docker_items,
                docker_cursor,
                docker_page,
                &confirm_prompt,
                freed_size,
                clean_percent,
                SPINNER_CHARS[spinner_idx],
                items_per_page,
                dry_run,
            );
        })?;

        // Poll for keyboard input (short timeout for responsive animations)
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    // Global quit shortcuts (q or Ctrl+C)
                    if (key.code == KeyCode::Char('q') && state != AppState::ConfirmTrash)
                        || (key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c'))
                    {
                        return Ok(());
                    }

                    match state {
                        AppState::MainMenu => {
                            status_msg.clear();
                            match key.code {
                                KeyCode::Up | KeyCode::Char('k') => {
                                    menu_cursor = menu_cursor.saturating_sub(1);
                                }
                                KeyCode::Down | KeyCode::Char('j') => {
                                    if menu_cursor < MENU_OPTIONS.len() - 1 {
                                        menu_cursor += 1;
                                    }
                                }
                                KeyCode::Enter => match menu_cursor {
                                    0 => {
                                        state = AppState::Selection;
                                        cursor = 0;
                                    }
                                    1 => {
                                        state = AppState::ScanningReport;
                                        let tx_c = tx.clone();
                                        thread::spawn(move || {
                                            let res = scanners::documents::scan_documents();
                                            let _ = tx_c.send(AppEvent::ReportFinished(res));
                                        });
                                    }
                                    2 => {
                                        let mut snap_size = 0;
                                        for it in &items {
                                            if it.id == "snapshots" {
                                                snap_size = it.size;
                                                break;
                                            }
                                        }
                                        previous_state = AppState::MainMenu;
                                        confirm_prompt = "Do you want to delete Time Machine Local Snapshots (requires sudo privileges)?".to_string();
                                        on_confirm_action = Some(Box::new(move || {
                                            if !dry_run {
                                                // Temporarily release terminal raw mode so sudo can prompt for password cleanly
                                                let _ = disable_raw_mode();
                                                let mut stdout = io::stdout();
                                                let _ = execute!(stdout, LeaveAlternateScreen, crossterm::cursor::Show);

                                                println!("\n[Cinder] Requesting administrator privileges to delete Time Machine snapshots...\n");
                                                let status = Command::new("sudo")
                                                    .args(["tmutil", "deletelocalsnapshots", "/"])
                                                    .status();

                                                if let Ok(st) = status {
                                                    if !st.success() {
                                                        eprintln!("\nCommand failed or was cancelled. Press Enter to continue...");
                                                        let mut buf = String::new();
                                                        let _ = io::stdin().read_line(&mut buf);
                                                    }
                                                }

                                                // Re-enter raw mode and alternate screen
                                                let _ = enable_raw_mode();
                                                let _ = execute!(
                                                    stdout,
                                                    EnterAlternateScreen,
                                                    crossterm::cursor::Hide,
                                                    crossterm::terminal::Clear(crossterm::terminal::ClearType::All)
                                                );
                                            }
                                            Some(AppEvent::CleanAllDone(snap_size))
                                        }));
                                        state = AppState::ConfirmTrash;
                                    }
                                    3 => {
                                        state = AppState::ScanningApps;
                                        let tx_c = tx.clone();
                                        thread::spawn(move || {
                                            let res = scanners::apps::scan_apps();
                                            let _ = tx_c.send(AppEvent::AppsFinished(res));
                                        });
                                    }
                                    4 => {
                                        state = AppState::ScanningHf;
                                        let tx_c = tx.clone();
                                        thread::spawn(move || {
                                            let res = scanners::huggingface::scan_hugging_face();
                                            let _ = tx_c.send(AppEvent::HfFinished(res));
                                        });
                                    }
                                    5 => {
                                        state = AppState::ScanningOllama;
                                        let tx_c = tx.clone();
                                        thread::spawn(move || {
                                            let res = scanners::ollama::scan_ollama();
                                            let _ = tx_c.send(AppEvent::OllamaFinished(res));
                                        });
                                    }
                                    6 => {
                                        state = AppState::ScanningBrowsers;
                                        let tx_c = tx.clone();
                                        thread::spawn(move || {
                                            let res = scanners::browsers::scan_browsers();
                                            let _ = tx_c.send(AppEvent::BrowsersFinished(res));
                                        });
                                    }
                                    7 => {
                                        state = AppState::ScanningDev;
                                        let tx_c = tx.clone();
                                        thread::spawn(move || {
                                            let res = scanners::devtools::scan_dev_tools();
                                            let _ = tx_c.send(AppEvent::DevFinished(res));
                                        });
                                    }
                                    8 => {
                                        state = AppState::ScanningFonts;
                                        let tx_c = tx.clone();
                                        thread::spawn(move || {
                                            let res = scanners::fonts::scan_fonts();
                                            let _ = tx_c.send(AppEvent::FontsFinished(res));
                                        });
                                    }
                                    9 => {
                                        state = AppState::ScanningAppSupport;
                                        let tx_c = tx.clone();
                                        thread::spawn(move || {
                                            let res = scanners::appsupport::scan_app_support();
                                            let _ = tx_c.send(AppEvent::AppSupportFinished(res));
                                        });
                                    }
                                    10 => {
                                        state = AppState::ScanningDocker;
                                        let tx_c = tx.clone();
                                        thread::spawn(move || {
                                            let res = scanners::docker::scan_docker();
                                            let _ = tx_c.send(AppEvent::DockerFinished(res));
                                        });
                                    }
                                    11 => {
                                        let _ = Command::new("open").arg("x-apple.systempreferences:com.apple.settings.Storage").output();
                                        status_msg = "Settings opened successfully!".to_string();
                                    }
                                    12 => return Ok(()),
                                    _ => {}
                                },
                                _ => {}
                            }
                        }
                        AppState::Selection => match key.code {
                            KeyCode::Up | KeyCode::Char('k') => {
                                cursor = cursor.saturating_sub(1);
                            }
                            KeyCode::Down | KeyCode::Char('j') => {
                                let start = selection_page * items_per_page;
                                let end = (start + items_per_page).min(items.len());
                                let page_size = end - start;
                                if page_size > 0 && cursor < page_size - 1 {
                                    cursor += 1;
                                }
                            }
                            KeyCode::Left | KeyCode::Char('p') => {
                                if selection_page > 0 {
                                    selection_page -= 1;
                                    cursor = 0;
                                }
                            }
                            KeyCode::Right | KeyCode::Char('n') => {
                                let total_pages = items.len().div_ceil(items_per_page);
                                if total_pages > 0 && selection_page < total_pages - 1 {
                                    selection_page += 1;
                                    cursor = 0;
                                }
                            }
                            KeyCode::Char(' ') => {
                                let item_idx = selection_page * items_per_page + cursor;
                                if item_idx < items.len() {
                                    items[item_idx].selected = !items[item_idx].selected;
                                }
                            }
                            KeyCode::Esc | KeyCode::Char('b') => {
                                state = AppState::MainMenu;
                            }
                            KeyCode::Enter => {
                                items_to_clean = items
                                    .iter()
                                    .enumerate()
                                    .filter(|(_, it)| it.selected && it.size > 0)
                                    .map(|(i, _)| i)
                                    .collect();

                                if items_to_clean.is_empty() {
                                    freed_size = 0;
                                    state = AppState::Finished;
                                } else {
                                    let names: Vec<String> = items_to_clean
                                        .iter()
                                        .map(|&i| items[i].name.clone())
                                        .collect();

                                    previous_state = AppState::Selection;
                                    confirm_prompt = format!(
                                        "Do you want to move the following selected items to the trash?\n\n- {}",
                                        names.join("\n- ")
                                    );

                                    let to_clean = items_to_clean.clone();
                                    let items_clone = items.clone();
                                    let tx_c = tx.clone();

                                    on_confirm_action = Some(Box::new(move || {
                                        let item_idx = to_clean[0];
                                        let item = items_clone[item_idx].clone();
                                        thread::spawn(move || {
                                            let freed = if let Some(clean_fn) = item.custom_clean {
                                                let (_, res) = clean_fn(dry_run);
                                                if res.is_ok() { item.size } else { 0 }
                                            } else {
                                                match scanners::move_to_trash(&item.path, dry_run) {
                                                    Ok(_) => item.size,
                                                    Err(_) => 0,
                                                }
                                            };
                                            thread::sleep(Duration::from_millis(300));
                                            let _ = tx_c.send(AppEvent::CleanItemDone(0, freed));
                                        });
                                        None
                                    }));

                                    state = AppState::ConfirmTrash;
                                }
                            }
                            _ => {}
                        },
                        AppState::DocumentsReport => match key.code {
                            KeyCode::Up | KeyCode::Char('k') => {
                                report_cursor = report_cursor.saturating_sub(1);
                            }
                            KeyCode::Down | KeyCode::Char('j') => {
                                let start = report_page * items_per_page;
                                let end = (start + items_per_page).min(large_files.len());
                                let page_size = end - start;
                                if page_size > 0 && report_cursor < page_size - 1 {
                                    report_cursor += 1;
                                }
                            }
                            KeyCode::Left | KeyCode::Char('p') => {
                                if report_page > 0 {
                                    report_page -= 1;
                                    report_cursor = 0;
                                }
                            }
                            KeyCode::Right | KeyCode::Char('n') => {
                                let total_pages = large_files.len().div_ceil(items_per_page);
                                if total_pages > 0 && report_page < total_pages - 1 {
                                    report_page += 1;
                                    report_cursor = 0;
                                }
                            }
                            KeyCode::Char(' ') => {
                                let item_idx = report_page * items_per_page + report_cursor;
                                if item_idx < large_files.len() {
                                    large_files[item_idx].selected = !large_files[item_idx].selected;
                                }
                            }
                            KeyCode::Enter => {
                                let mut to_clean: Vec<scanners::documents::FileInfo> = large_files
                                    .iter()
                                    .filter(|f| f.selected)
                                    .cloned()
                                    .collect();

                                if to_clean.is_empty() {
                                    let item_idx = report_page * items_per_page + report_cursor;
                                    if item_idx < large_files.len() {
                                        to_clean.push(large_files[item_idx].clone());
                                    }
                                }

                                if !to_clean.is_empty() {
                                    let names: Vec<String> = to_clean
                                        .iter()
                                        .map(|f| {
                                            let p = f.path.to_string_lossy();
                                            let home = dirs::home_dir().unwrap_or_default();
                                            p.replace(&home.to_string_lossy().to_string(), "~")
                                        })
                                        .collect();

                                    previous_state = AppState::DocumentsReport;
                                    let total_to_free: u64 = to_clean.iter().map(|f| f.size).sum();
                                    confirm_prompt = format!(
                                        "Do you want to move the following {} file(s) ({}) to the trash?\n\n- {}",
                                        to_clean.len(),
                                        scanners::format_size(total_to_free),
                                        names.join("\n- ")
                                    );

                                    let tx_c = tx.clone();
                                    on_confirm_action = Some(Box::new(move || {
                                        thread::spawn(move || {
                                            let total_items = to_clean.len();
                                            let mut accumulated = 0;
                                            for (idx, it) in to_clean.into_iter().enumerate() {
                                                let _ = scanners::move_to_trash(&it.path, dry_run);
                                                accumulated += it.size;
                                                let _ = tx_c.send(AppEvent::CleanProgress(idx + 1, total_items, it.size));
                                            }
                                            let res = scanners::documents::scan_documents();
                                            let _ = tx_c.send(AppEvent::ReportFinished(res));
                                            let _ = tx_c.send(AppEvent::CleanAllDone(accumulated));
                                        });
                                        None
                                    }));
                                    state = AppState::ConfirmTrash;
                                }
                            }
                            KeyCode::Esc | KeyCode::Char('b') => {
                                state = AppState::MainMenu;
                            }
                            _ => {}
                        },
                        AppState::AppsSuggestions => match key.code {
                            KeyCode::Up | KeyCode::Char('k') => {
                                app_cursor = app_cursor.saturating_sub(1);
                            }
                            KeyCode::Down | KeyCode::Char('j') => {
                                let start = app_page * items_per_page;
                                let end = (start + items_per_page).min(app_suggestions.len());
                                let page_size = end - start;
                                if page_size > 0 && app_cursor < page_size - 1 {
                                    app_cursor += 1;
                                }
                            }
                            KeyCode::Left | KeyCode::Char('p') => {
                                if app_page > 0 {
                                    app_page -= 1;
                                    app_cursor = 0;
                                }
                            }
                            KeyCode::Right | KeyCode::Char('n') => {
                                let total_pages = app_suggestions.len().div_ceil(items_per_page);
                                if total_pages > 0 && app_page < total_pages - 1 {
                                    app_page += 1;
                                    app_cursor = 0;
                                }
                            }
                            KeyCode::Char('o') => {
                                let item_idx = app_page * items_per_page + app_cursor;
                                if item_idx < app_suggestions.len() {
                                    let _ = Command::new("open")
                                        .arg("-R")
                                        .arg(&app_suggestions[item_idx].path)
                                        .output();
                                }
                            }
                            KeyCode::Enter => {
                                let item_idx = app_page * items_per_page + app_cursor;
                                if item_idx < app_suggestions.len() {
                                    let target_app = app_suggestions[item_idx].clone();
                                    previous_state = AppState::AppsSuggestions;
                                    confirm_prompt = format!(
                                        "Do you want to move the application '{}' ({}) to the trash?",
                                        target_app.name,
                                        scanners::format_size(target_app.size)
                                    );

                                    let target_size = target_app.size;
                                    on_confirm_action = Some(Box::new(move || {
                                        if !dry_run {
                                            // Attempt Finder osascript first, then fallback
                                            let script = format!(
                                                r#"tell application "Finder" to move POSIX file "{}" to trash"#,
                                                target_app.path.display()
                                            );
                                            let out = Command::new("osascript").args(["-e", &script]).output();
                                            if out.is_err() || !out.as_ref().unwrap().status.success() {
                                                let _ = scanners::move_to_trash(&target_app.path, dry_run);
                                            }
                                        }
                                        Some(AppEvent::CleanAllDone(target_size))
                                    }));
                                    state = AppState::ConfirmTrash;
                                }
                            }
                            KeyCode::Esc | KeyCode::Char('b') => {
                                state = AppState::MainMenu;
                            }
                            _ => {}
                        },
                        AppState::HuggingFace => match key.code {
                            KeyCode::Up | KeyCode::Char('k') => {
                                hf_cursor = hf_cursor.saturating_sub(1);
                            }
                            KeyCode::Down | KeyCode::Char('j') => {
                                let start = hf_page * items_per_page;
                                let end = (start + items_per_page).min(hf_items.len());
                                let page_size = end - start;
                                if page_size > 0 && hf_cursor < page_size - 1 {
                                    hf_cursor += 1;
                                }
                            }
                            KeyCode::Left | KeyCode::Char('p') => {
                                if hf_page > 0 {
                                    hf_page -= 1;
                                    hf_cursor = 0;
                                }
                            }
                            KeyCode::Right | KeyCode::Char('n') => {
                                let total_pages = hf_items.len().div_ceil(items_per_page);
                                if total_pages > 0 && hf_page < total_pages - 1 {
                                    hf_page += 1;
                                    hf_cursor = 0;
                                }
                            }
                            KeyCode::Enter => {
                                let item_idx = hf_page * items_per_page + hf_cursor;
                                if item_idx < hf_items.len() {
                                    let target_item = hf_items[item_idx].clone();
                                    previous_state = AppState::HuggingFace;
                                    confirm_prompt = format!(
                                        "Do you want to move the Hugging Face item/model '{}' ({}) to the trash?",
                                        target_item.name,
                                        scanners::format_size(target_item.size)
                                    );

                                    let target_size = target_item.size;
                                    on_confirm_action = Some(Box::new(move || {
                                        let _ = scanners::move_to_trash(&target_item.path, dry_run);
                                        Some(AppEvent::CleanAllDone(target_size))
                                    }));
                                    state = AppState::ConfirmTrash;
                                }
                            }
                            KeyCode::Esc | KeyCode::Char('b') => {
                                state = AppState::MainMenu;
                            }
                            _ => {}
                        },
                        AppState::Ollama => match key.code {
                            KeyCode::Up | KeyCode::Char('k') => {
                                ollama_cursor = ollama_cursor.saturating_sub(1);
                            }
                            KeyCode::Down | KeyCode::Char('j') => {
                                let start = ollama_page * items_per_page;
                                let end = (start + items_per_page).min(ollama_items.len());
                                let page_size = end - start;
                                if page_size > 0 && ollama_cursor < page_size - 1 {
                                    ollama_cursor += 1;
                                }
                            }
                            KeyCode::Left | KeyCode::Char('p') => {
                                if ollama_page > 0 {
                                    ollama_page -= 1;
                                    ollama_cursor = 0;
                                }
                            }
                            KeyCode::Right | KeyCode::Char('n') => {
                                let total_pages = ollama_items.len().div_ceil(items_per_page);
                                if total_pages > 0 && ollama_page < total_pages - 1 {
                                    ollama_page += 1;
                                    ollama_cursor = 0;
                                }
                            }
                            KeyCode::Enter => {
                                let item_idx = ollama_page * items_per_page + ollama_cursor;
                                if item_idx < ollama_items.len() {
                                    let target_item = ollama_items[item_idx].clone();
                                    previous_state = AppState::Ollama;
                                    confirm_prompt = format!(
                                        "Do you want to move the Ollama model '{}' ({}) to the trash?",
                                        target_item.name,
                                        scanners::format_size(target_item.size)
                                    );

                                    let target_size = target_item.size;
                                    on_confirm_action = Some(Box::new(move || {
                                        let _ = scanners::ollama::delete_ollama_model(&target_item.path, dry_run);
                                        Some(AppEvent::CleanAllDone(target_size))
                                    }));
                                    state = AppState::ConfirmTrash;
                                }
                            }
                            KeyCode::Esc | KeyCode::Char('b') => {
                                state = AppState::MainMenu;
                            }
                            _ => {}
                        },
                        AppState::Browsers => match key.code {
                            KeyCode::Up | KeyCode::Char('k') => {
                                browsers_cursor = browsers_cursor.saturating_sub(1);
                            }
                            KeyCode::Down | KeyCode::Char('j') => {
                                let start = browsers_page * items_per_page;
                                let end = (start + items_per_page).min(browsers_items.len());
                                let page_size = end - start;
                                if page_size > 0 && browsers_cursor < page_size - 1 {
                                    browsers_cursor += 1;
                                }
                            }
                            KeyCode::Left | KeyCode::Char('p') => {
                                if browsers_page > 0 {
                                    browsers_page -= 1;
                                    browsers_cursor = 0;
                                }
                            }
                            KeyCode::Right | KeyCode::Char('n') => {
                                let total_pages = browsers_items.len().div_ceil(items_per_page);
                                if total_pages > 0 && browsers_page < total_pages - 1 {
                                    browsers_page += 1;
                                    browsers_cursor = 0;
                                }
                            }
                            KeyCode::Char(' ') => {
                                let item_idx = browsers_page * items_per_page + browsers_cursor;
                                if item_idx < browsers_items.len() {
                                    browsers_items[item_idx].selected = !browsers_items[item_idx].selected;
                                }
                            }
                            KeyCode::Esc | KeyCode::Char('b') => {
                                state = AppState::MainMenu;
                            }
                            KeyCode::Enter => {
                                let mut to_clean: Vec<scanners::browsers::BrowserItem> = browsers_items
                                    .iter()
                                    .filter(|it| it.selected)
                                    .cloned()
                                    .collect();

                                if to_clean.is_empty() {
                                    let item_idx = browsers_page * items_per_page + browsers_cursor;
                                    if item_idx < browsers_items.len() {
                                        to_clean.push(browsers_items[item_idx].clone());
                                    }
                                }

                                if !to_clean.is_empty() {
                                    let total_to_free: u64 = to_clean.iter().map(|it| it.size).sum();
                                    let names: Vec<String> = to_clean.iter().map(|it| it.name.clone()).collect();
                                    previous_state = AppState::Browsers;
                                    confirm_prompt = format!(
                                        "Do you want to move the following {} browser cache(s) ({}) to the trash?\n\n- {}",
                                        to_clean.len(),
                                        scanners::format_size(total_to_free),
                                        names.join("\n- ")
                                    );

                                    let tx_c = tx.clone();
                                    on_confirm_action = Some(Box::new(move || {
                                        thread::spawn(move || {
                                            let total_items = to_clean.len();
                                            let mut accumulated = 0;
                                            for (idx, it) in to_clean.into_iter().enumerate() {
                                                let _ = scanners::move_to_trash(&it.path, dry_run);
                                                accumulated += it.size;
                                                let _ = tx_c.send(AppEvent::CleanProgress(idx + 1, total_items, it.size));
                                            }
                                            let res = scanners::browsers::scan_browsers();
                                            let _ = tx_c.send(AppEvent::BrowsersFinished(res));
                                            let _ = tx_c.send(AppEvent::CleanAllDone(accumulated));
                                        });
                                        None
                                    }));
                                    state = AppState::ConfirmTrash;
                                }
                            }
                            _ => {}
                        },
                        AppState::DevTools => match key.code {
                            KeyCode::Up | KeyCode::Char('k') => {
                                dev_cursor = dev_cursor.saturating_sub(1);
                            }
                            KeyCode::Down | KeyCode::Char('j') => {
                                let start = dev_page * items_per_page;
                                let end = (start + items_per_page).min(dev_items.len());
                                let page_size = end - start;
                                if page_size > 0 && dev_cursor < page_size - 1 {
                                    dev_cursor += 1;
                                }
                            }
                            KeyCode::Left | KeyCode::Char('p') => {
                                if dev_page > 0 {
                                    dev_page -= 1;
                                    dev_cursor = 0;
                                }
                            }
                            KeyCode::Right | KeyCode::Char('n') => {
                                let total_pages = dev_items.len().div_ceil(items_per_page);
                                if total_pages > 0 && dev_page < total_pages - 1 {
                                    dev_page += 1;
                                    dev_cursor = 0;
                                }
                            }
                            KeyCode::Char(' ') => {
                                let item_idx = dev_page * items_per_page + dev_cursor;
                                if item_idx < dev_items.len() {
                                    dev_items[item_idx].selected = !dev_items[item_idx].selected;
                                }
                            }
                            KeyCode::Char('a') => {
                                let all_selected = dev_items.iter().all(|it| it.selected);
                                for it in &mut dev_items {
                                    it.selected = !all_selected;
                                }
                            }
                            KeyCode::Enter => {
                                let mut to_clean: Vec<scanners::devtools::DevItem> = dev_items
                                    .iter()
                                    .filter(|it| it.selected)
                                    .cloned()
                                    .collect();

                                if to_clean.is_empty() {
                                    let item_idx = dev_page * items_per_page + dev_cursor;
                                    if item_idx < dev_items.len() {
                                        to_clean.push(dev_items[item_idx].clone());
                                    }
                                }

                                if !to_clean.is_empty() {
                                    let total_to_free: u64 = to_clean.iter().map(|it| it.size).sum();
                                    let summary_lines: Vec<String> = to_clean
                                        .iter()
                                        .map(|it| {
                                            format!(
                                                "- [{}] {} ({})",
                                                it.action_type,
                                                it.name,
                                                scanners::format_size(it.size)
                                            )
                                        })
                                        .collect();

                                    previous_state = AppState::DevTools;
                                    confirm_prompt = format!(
                                        "Execute cleanup for the following {} item(s) (Total: {})?\n\n{}",
                                        to_clean.len(),
                                        scanners::format_size(total_to_free),
                                        summary_lines.join("\n")
                                    );

                                    let tx_c = tx.clone();
                                    on_confirm_action = Some(Box::new(move || {
                                        thread::spawn(move || {
                                            let total_items = to_clean.len();
                                            let mut accumulated = 0;
                                            for (idx, it) in to_clean.into_iter().enumerate() {
                                                let freed = if let Some(clean_fn) = it.custom_clean {
                                                    let (_, res) = clean_fn(dry_run);
                                                    if res.is_ok() { it.size } else { 0 }
                                                } else {
                                                    match scanners::move_to_trash(&it.path, dry_run) {
                                                        Ok(_) => it.size,
                                                        Err(_) => 0,
                                                    }
                                                };
                                                accumulated += freed;
                                                let _ = tx_c.send(AppEvent::CleanProgress(idx + 1, total_items, freed));
                                            }
                                            let res = scanners::devtools::scan_dev_tools();
                                            let _ = tx_c.send(AppEvent::DevFinished(res));
                                            let _ = tx_c.send(AppEvent::CleanAllDone(accumulated));
                                        });
                                        None
                                    }));
                                    state = AppState::ConfirmTrash;
                                }
                            }
                            KeyCode::Esc | KeyCode::Char('b') => {
                                state = AppState::MainMenu;
                            }
                            _ => {}
                        },
                        AppState::Fonts => match key.code {
                            KeyCode::Up | KeyCode::Char('k') => {
                                font_cursor = font_cursor.saturating_sub(1);
                            }
                            KeyCode::Down | KeyCode::Char('j') => {
                                let start = font_page * items_per_page;
                                let end = (start + items_per_page).min(font_items.len());
                                let page_size = end - start;
                                if page_size > 0 && font_cursor < page_size - 1 {
                                    font_cursor += 1;
                                }
                            }
                            KeyCode::Left | KeyCode::Char('p') => {
                                if font_page > 0 {
                                    font_page -= 1;
                                    font_cursor = 0;
                                }
                            }
                            KeyCode::Right | KeyCode::Char('n') => {
                                let total_pages = font_items.len().div_ceil(items_per_page);
                                if total_pages > 0 && font_page < total_pages - 1 {
                                    font_page += 1;
                                    font_cursor = 0;
                                }
                            }
                            KeyCode::Char(' ') => {
                                let item_idx = font_page * items_per_page + font_cursor;
                                if item_idx < font_items.len() {
                                    font_items[item_idx].selected = !font_items[item_idx].selected;
                                }
                            }
                            KeyCode::Char('a') => {
                                let all_selected = font_items.iter().all(|it| it.selected);
                                for it in &mut font_items {
                                    it.selected = !all_selected;
                                }
                            }
                            KeyCode::Enter => {
                                let mut to_clean: Vec<scanners::fonts::FontItem> = font_items
                                    .iter()
                                    .filter(|it| it.selected)
                                    .cloned()
                                    .collect();

                                if to_clean.is_empty() {
                                    let item_idx = font_page * items_per_page + font_cursor;
                                    if item_idx < font_items.len() {
                                        to_clean.push(font_items[item_idx].clone());
                                    }
                                }

                                if !to_clean.is_empty() {
                                    let total_to_free: u64 = to_clean.iter().map(|it| it.size).sum();
                                    let summary_lines: Vec<String> = to_clean
                                        .iter()
                                        .map(|it| {
                                            format!(
                                                "- {} ({} files, {})",
                                                it.family,
                                                it.count,
                                                scanners::format_size(it.size)
                                            )
                                        })
                                        .collect();

                                    previous_state = AppState::Fonts;
                                    confirm_prompt = format!(
                                        "Move the following {} font families (Total: {}) to macOS Trash?\n\n{}",
                                        to_clean.len(),
                                        scanners::format_size(total_to_free),
                                        summary_lines.join("\n")
                                    );

                                    let tx_c = tx.clone();
                                    on_confirm_action = Some(Box::new(move || {
                                        thread::spawn(move || {
                                            let total_items = to_clean.len();
                                            let mut accumulated = 0;
                                            for (idx, it) in to_clean.into_iter().enumerate() {
                                                for file in it.files {
                                                    let _ = scanners::move_to_trash(&file, dry_run);
                                                }
                                                accumulated += it.size;
                                                let _ = tx_c.send(AppEvent::CleanProgress(idx + 1, total_items, it.size));
                                            }
                                            let res = scanners::fonts::scan_fonts();
                                            let _ = tx_c.send(AppEvent::FontsFinished(res));
                                            let _ = tx_c.send(AppEvent::CleanAllDone(accumulated));
                                        });
                                        None
                                    }));
                                    state = AppState::ConfirmTrash;
                                }
                            }
                            KeyCode::Esc | KeyCode::Char('b') => {
                                state = AppState::MainMenu;
                            }
                            _ => {}
                        },
                        AppState::AppSupport => match key.code {
                            KeyCode::Up | KeyCode::Char('k') => {
                                app_support_cursor = app_support_cursor.saturating_sub(1);
                            }
                            KeyCode::Down | KeyCode::Char('j') => {
                                let start = app_support_page * items_per_page;
                                let end = (start + items_per_page).min(app_support_items.len());
                                let page_size = end - start;
                                if page_size > 0 && app_support_cursor < page_size - 1 {
                                    app_support_cursor += 1;
                                }
                            }
                            KeyCode::Left | KeyCode::Char('p') => {
                                if app_support_page > 0 {
                                    app_support_page -= 1;
                                    app_support_cursor = 0;
                                }
                            }
                            KeyCode::Right | KeyCode::Char('n') => {
                                let total_pages = app_support_items.len().div_ceil(items_per_page);
                                if total_pages > 0 && app_support_page < total_pages - 1 {
                                    app_support_page += 1;
                                    app_support_cursor = 0;
                                }
                            }
                            KeyCode::Char(' ') => {
                                let item_idx = app_support_page * items_per_page + app_support_cursor;
                                if item_idx < app_support_items.len() {
                                    app_support_items[item_idx].selected = !app_support_items[item_idx].selected;
                                }
                            }
                            KeyCode::Esc | KeyCode::Char('b') => {
                                state = AppState::MainMenu;
                            }
                            KeyCode::Enter => {
                                let mut to_clean: Vec<scanners::appsupport::AppSupportItem> = app_support_items
                                    .iter()
                                    .filter(|it| it.selected)
                                    .cloned()
                                    .collect();

                                if to_clean.is_empty() {
                                    let item_idx = app_support_page * items_per_page + app_support_cursor;
                                    if item_idx < app_support_items.len() {
                                        to_clean.push(app_support_items[item_idx].clone());
                                    }
                                }

                                if !to_clean.is_empty() {
                                    let total_to_free: u64 = to_clean.iter().map(|it| it.size).sum();
                                    let names: Vec<String> = to_clean.iter().map(|it| it.name.clone()).collect();
                                    previous_state = AppState::AppSupport;
                                    confirm_prompt = format!(
                                        "Do you want to move the following {} Application Support directorie(s) ({}) to the trash?\n\n- {}",
                                        to_clean.len(),
                                        scanners::format_size(total_to_free),
                                        names.join("\n- ")
                                    );

                                    let tx_c = tx.clone();
                                    on_confirm_action = Some(Box::new(move || {
                                        thread::spawn(move || {
                                            let total_items = to_clean.len();
                                            let mut accumulated = 0;
                                            for (idx, it) in to_clean.into_iter().enumerate() {
                                                let _ = scanners::move_to_trash(&it.path, dry_run);
                                                accumulated += it.size;
                                                let _ = tx_c.send(AppEvent::CleanProgress(idx + 1, total_items, it.size));
                                            }
                                            let res = scanners::appsupport::scan_app_support();
                                            let _ = tx_c.send(AppEvent::AppSupportFinished(res));
                                            let _ = tx_c.send(AppEvent::CleanAllDone(accumulated));
                                        });
                                        None
                                    }));
                                    state = AppState::ConfirmTrash;
                                }
                            }
                            _ => {}
                        },
                        AppState::Docker => match key.code {
                            KeyCode::Up | KeyCode::Char('k') => {
                                docker_cursor = docker_cursor.saturating_sub(1);
                            }
                            KeyCode::Down | KeyCode::Char('j') => {
                                let start = docker_page * items_per_page;
                                let end = (start + items_per_page).min(docker_items.len());
                                let page_size = end - start;
                                if page_size > 0 && docker_cursor < page_size - 1 {
                                    docker_cursor += 1;
                                }
                            }
                            KeyCode::Left | KeyCode::Char('p') => {
                                if docker_page > 0 {
                                    docker_page -= 1;
                                    docker_cursor = 0;
                                }
                            }
                            KeyCode::Right | KeyCode::Char('n') => {
                                let total_pages = docker_items.len().div_ceil(items_per_page);
                                if total_pages > 0 && docker_page < total_pages - 1 {
                                    docker_page += 1;
                                    docker_cursor = 0;
                                }
                            }
                            KeyCode::Enter => {
                                let item_idx = docker_page * items_per_page + docker_cursor;
                                if item_idx < docker_items.len() {
                                    let target_item = docker_items[item_idx].clone();
                                    previous_state = AppState::Docker;
                                    confirm_prompt = format!(
                                        "Do you want to clean/delete '{}' ({})?",
                                        target_item.name,
                                        scanners::format_size(target_item.size)
                                    );

                                    let target_size = target_item.size;
                                    on_confirm_action = Some(Box::new(move || {
                                        if let Some(clean_fn) = target_item.custom_clean {
                                            let _ = clean_fn(dry_run);
                                        } else {
                                            let _ = scanners::move_to_trash(&target_item.path, dry_run);
                                        }
                                        Some(AppEvent::CleanAllDone(target_size))
                                    }));
                                    state = AppState::ConfirmTrash;
                                }
                            }
                            KeyCode::Esc | KeyCode::Char('b') => {
                                state = AppState::MainMenu;
                            }
                            _ => {}
                        },
                        AppState::ConfirmTrash => match key.code {
                            KeyCode::Char('y') | KeyCode::Char('s') | KeyCode::Enter => {
                                clean_percent = 0.0;
                                freed_size = 0;
                                state = AppState::Cleaning;

                                if let Some(action) = on_confirm_action.take() {
                                    if let Some(event) = action() {
                                        let _ = tx.send(event);
                                    }
                                }
                            }
                            KeyCode::Char('n') | KeyCode::Esc | KeyCode::Char('b') => {
                                on_confirm_action = None;
                                state = previous_state;
                            }
                            _ => {}
                        },
                        AppState::Finished => match key.code {
                            KeyCode::Esc | KeyCode::Enter | KeyCode::Char('b') => {
                                state = AppState::Scanning;
                                scan_cleanup_items(items.clone(), tx.clone());
                            }
                            _ => {}
                        },
                        _ => {}
                    }
                }
            }
        }
    }
}
