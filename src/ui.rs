use crate::scanners;
use crate::theme::*;
use crate::types::*;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};

pub fn render_ui(
    f: &mut Frame,
    state: AppState,
    items: &[CleanupItem],
    cursor: usize,
    selection_page: usize,
    menu_cursor: usize,
    menu_options: &[&str],
    status_msg: &str,
    large_files: &[scanners::documents::FileInfo],
    report_cursor: usize,
    report_page: usize,
    app_suggestions: &[scanners::apps::AppSuggestion],
    app_cursor: usize,
    app_page: usize,
    hf_items: &[scanners::huggingface::HFItem],
    hf_cursor: usize,
    hf_page: usize,
    ollama_items: &[scanners::ollama::OllamaItem],
    ollama_cursor: usize,
    ollama_page: usize,
    browsers_items: &[scanners::browsers::BrowserItem],
    browsers_cursor: usize,
    browsers_page: usize,
    dev_items: &[scanners::devtools::DevItem],
    dev_cursor: usize,
    dev_page: usize,
    font_items: &[scanners::fonts::FontItem],
    font_cursor: usize,
    font_page: usize,
    app_support_items: &[scanners::appsupport::AppSupportItem],
    app_support_cursor: usize,
    app_support_page: usize,
    docker_items: &[scanners::docker::DockerItem],
    docker_cursor: usize,
    docker_page: usize,
    confirm_prompt: &str,
    freed_size: u64,
    clean_percent: f64,
    spinner_char: char,
    items_per_page: usize,
    dry_run: bool,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Title
            Constraint::Min(10),   // Content
            Constraint::Length(2), // Help
        ])
        .split(f.area());

    // Title banner
    let mut title_spans = vec![
        Span::styled(
            " CINDER CLI: NAVIGATION MENU ",
            Style::default()
                .fg(FG_THEME)
                .bg(PURPLE_THEME)
                .add_modifier(Modifier::BOLD),
        ),
    ];
    if dry_run {
        title_spans.push(Span::raw(" "));
        title_spans.push(Span::styled(
            " [DRY-RUN MODE - NO FILES WILL BE DELETED] ",
            Style::default()
                .fg(FG_THEME)
                .bg(YELLOW_THEME)
                .add_modifier(Modifier::BOLD),
        ));
    }
    let title_line = Line::from(title_spans);
    f.render_widget(Paragraph::new(title_line), chunks[0]);

    // Content area
    render_content(
        f,
        chunks[1],
        state,
        items,
        cursor,
        selection_page,
        menu_cursor,
        menu_options,
        status_msg,
        large_files,
        report_cursor,
        report_page,
        app_suggestions,
        app_cursor,
        app_page,
        hf_items,
        hf_cursor,
        hf_page,
        ollama_items,
        ollama_cursor,
        ollama_page,
        browsers_items,
        browsers_cursor,
        browsers_page,
        dev_items,
        dev_cursor,
        dev_page,
        font_items,
        font_cursor,
        font_page,
        app_support_items,
        app_support_cursor,
        app_support_page,
        docker_items,
        docker_cursor,
        docker_page,
        confirm_prompt,
        freed_size,
        clean_percent,
        spinner_char,
        items_per_page,
    );

    // Help bar
    let help_text = get_help_text(state);
    let help_line = Line::from(vec![Span::styled(
        help_text,
        Style::default().fg(GRAY_THEME),
    )]);
    f.render_widget(Paragraph::new(help_line), chunks[2]);
}

fn get_help_text(state: AppState) -> &'static str {
    match state {
        AppState::MainMenu => "↑/↓: Navigate • [Enter]: Choose Option • [q]: Exit",
        AppState::Selection => "↑/↓: Navigate • [Space]: Select • [Enter]: Clean Selected • [Esc/b]: Back",
        AppState::DocumentsReport => "↑/↓: Navigate • [Space]: Select • [Enter]: Clean Selected • ←/p: Prev • →/n: Next • [Esc/b]: Back",
        AppState::AppsSuggestions => "↑/↓: Navigate • [Enter]: Move to Trash • [o]: Reveal in Finder • [Esc/b]: Back",
        AppState::HuggingFace => "↑/↓: Navigate • [Enter]: Move to Trash • [Esc/b]: Back",
        AppState::Ollama => "↑/↓: Navigate • [Enter]: Move to Trash • [Esc/b]: Back",
        AppState::Browsers => "↑/↓: Navigate • [Space]: Select • [Enter]: Clean Selected • [Esc/b]: Back",
        AppState::DevTools => "↑/↓: Navigate • [Space]: Toggle • [a]: Select All • [Enter]: Clean Selected • [Esc/b]: Back",
        AppState::Fonts => "↑/↓: Navigate • [Space]: Toggle • [a]: Select All • [Enter]: Move to Trash • [Esc/b]: Back",
        AppState::AppSupport => "↑/↓: Navigate • [Space]: Select • [Enter]: Clean Selected • [Esc/b]: Back",
        AppState::Docker => "↑/↓: Navigate • [Enter]: Clean • [Esc/b]: Back",
        AppState::ConfirmTrash => "[S/Enter]: Yes, continue • [N/Esc/b]: Cancel and go back",
        AppState::Finished => "[Esc/Enter/b]: Navigation Menu • [q]: Exit",
        _ => "Loading...",
    }
}

fn render_content(
    f: &mut Frame,
    area: Rect,
    state: AppState,
    items: &[CleanupItem],
    cursor: usize,
    selection_page: usize,
    menu_cursor: usize,
    menu_options: &[&str],
    status_msg: &str,
    large_files: &[scanners::documents::FileInfo],
    report_cursor: usize,
    report_page: usize,
    app_suggestions: &[scanners::apps::AppSuggestion],
    app_cursor: usize,
    app_page: usize,
    hf_items: &[scanners::huggingface::HFItem],
    hf_cursor: usize,
    hf_page: usize,
    ollama_items: &[scanners::ollama::OllamaItem],
    ollama_cursor: usize,
    ollama_page: usize,
    browsers_items: &[scanners::browsers::BrowserItem],
    browsers_cursor: usize,
    browsers_page: usize,
    dev_items: &[scanners::devtools::DevItem],
    dev_cursor: usize,
    dev_page: usize,
    font_items: &[scanners::fonts::FontItem],
    font_cursor: usize,
    font_page: usize,
    app_support_items: &[scanners::appsupport::AppSupportItem],
    app_support_cursor: usize,
    app_support_page: usize,
    docker_items: &[scanners::docker::DockerItem],
    docker_cursor: usize,
    docker_page: usize,
    confirm_prompt: &str,
    freed_size: u64,
    clean_percent: f64,
    spinner_char: char,
    items_per_page: usize,
) {
    match state {
        AppState::Scanning => {
            render_spinner_view(f, area, spinner_char, "Calculating accumulated files on disk...");
        }
        AppState::ScanningReport => {
            render_spinner_view(f, area, spinner_char, "Scanning ~/Documents folder to find the largest files...");
        }
        AppState::ScanningApps => {
            render_spinner_view(f, area, spinner_char, "Scanning installed applications and measuring usage...");
        }
        AppState::ScanningHf => {
            render_spinner_view(f, area, spinner_char, "Scanning Hugging Face models and caches...");
        }
        AppState::ScanningOllama => {
            render_spinner_view(f, area, spinner_char, "Scanning Ollama models...");
        }
        AppState::ScanningBrowsers => {
            render_spinner_view(f, area, spinner_char, "Scanning browser caches...");
        }
        AppState::ScanningDev => {
            render_spinner_view(f, area, spinner_char, "Scanning developer tool caches...");
        }
        AppState::ScanningFonts => {
            render_spinner_view(f, area, spinner_char, "Scanning ~/Library/Fonts to analyze Nerd Fonts redundancy...");
        }
        AppState::ScanningAppSupport => {
            render_spinner_view(f, area, spinner_char, "Scanning Application Support caches and data...");
        }
        AppState::ScanningDocker => {
            render_spinner_view(f, area, spinner_char, "Scanning Docker VM storage files and caches...");
        }
        AppState::MainMenu => {
            let mut lines = vec![
                Line::from(Span::styled("Select a tool option:\n", Style::default().fg(FG_THEME))),
            ];

            let mut menu_lines = Vec::new();
            for (i, opt) in menu_options.iter().enumerate() {
                if menu_cursor == i {
                    menu_lines.push(Line::from(vec![
                        Span::styled("> ", Style::default().fg(CYAN_THEME).add_modifier(Modifier::BOLD)),
                        Span::styled(*opt, Style::default().fg(CYAN_THEME).add_modifier(Modifier::BOLD)),
                    ]));
                } else {
                    menu_lines.push(Line::from(vec![
                        Span::raw("  "),
                        Span::styled(*opt, Style::default().fg(FG_THEME)),
                    ]));
                }
            }

            let menu_block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(PURPLE_THEME));
            let content_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1),
                    Constraint::Min(4),
                    Constraint::Length(2),
                ])
                .split(area);

            // Compute scroll offset for menu if terminal height is constrained
            // Inner height is content_layout[1].height minus 2 for borders
            let inner_height = content_layout[1].height.saturating_sub(2) as usize;
            let scroll_offset = if inner_height > 0 && menu_cursor >= inner_height {
                (menu_cursor - inner_height + 1) as u16
            } else {
                0
            };

            let menu_p = Paragraph::new(menu_lines)
                .block(menu_block)
                .scroll((scroll_offset, 0));

            f.render_widget(Paragraph::new(lines.remove(0)), content_layout[0]);
            f.render_widget(menu_p, content_layout[1]);

            if !status_msg.is_empty() {
                let status_line = Line::from(Span::styled(
                    status_msg,
                    Style::default().fg(GREEN_THEME).add_modifier(Modifier::BOLD),
                ));
                f.render_widget(Paragraph::new(status_line), content_layout[2]);
            }
        }
        AppState::Selection => {
            let start = selection_page * items_per_page;
            let end = (start + items_per_page).min(items.len());
            let total_pages = items.len().div_ceil(items_per_page);

            let mut selected_total: u64 = 0;
            for it in items {
                if it.selected {
                    selected_total += it.size;
                }
            }

            let mut list_lines = Vec::new();
            for (idx, item) in items[start..end].iter().enumerate() {
                let is_current = cursor == idx;
                let cursor_str = if is_current { "> " } else { "  " };
                let checked_str = if item.selected { "x" } else { " " };
                let size_str = scanners::format_size(item.size);

                let content = format!("[{}] {:<50} {}", checked_str, item.name, size_str);

                if is_current {
                    list_lines.push(Line::from(vec![
                        Span::styled(cursor_str, Style::default().fg(CYAN_THEME).add_modifier(Modifier::BOLD)),
                        Span::styled(content, Style::default().fg(CYAN_THEME).add_modifier(Modifier::BOLD)),
                    ]));
                } else {
                    list_lines.push(Line::from(vec![
                        Span::raw(cursor_str),
                        Span::styled(content, Style::default().fg(FG_THEME)),
                    ]));
                }
            }

            let box_block = Block::default()
                .title(" SELECT AND CLEAN CACHES / CONTAINERS ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(CYAN_THEME));

            let summary_text = format!("Estimated Space to Free: {}", scanners::format_size(selected_total));
            let summary_block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(PURPLE_THEME));

            let page_text = if total_pages > 1 {
                format!("Page {} of {} (Items {}-{} of {})", selection_page + 1, total_pages, start + 1, end, items.len())
            } else {
                String::new()
            };

            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(4),
                    Constraint::Length(1),
                    Constraint::Length(3),
                ])
                .split(area);

            f.render_widget(Paragraph::new(list_lines).block(box_block), layout[0]);
            f.render_widget(Paragraph::new(Line::from(Span::styled(page_text, Style::default().fg(YELLOW_THEME)))), layout[1]);
            f.render_widget(Paragraph::new(Line::from(Span::styled(summary_text, Style::default().fg(FG_THEME).add_modifier(Modifier::BOLD)))).block(summary_block), layout[2]);
        }
        AppState::DocumentsReport => {
            let start = report_page * items_per_page;
            let end = (start + items_per_page).min(large_files.len());
            let total_pages = large_files.len().div_ceil(items_per_page);

            let mut list_lines = Vec::new();
            if large_files.is_empty() {
                list_lines.push(Line::from(Span::styled("No large files found in ~/Documents.", Style::default().fg(GRAY_THEME))));
            } else {
                let home = dirs::home_dir().unwrap_or_default();
                for (idx, file) in large_files[start..end].iter().enumerate() {
                    let is_current = report_cursor == idx;
                    let cursor_str = if is_current { "> " } else { "  " };
                    let checked_str = if file.selected { "x" } else { " " };

                    let file_num = start + idx + 1;
                    let full_path = file.path.to_string_lossy();
                    let mut display_path = full_path.replace(&home.to_string_lossy().to_string(), "~");
                    let chars: Vec<char> = display_path.chars().collect();
                    if chars.len() > 46 {
                        let prefix: String = chars.iter().take(18).collect();
                        let suffix: String = chars.iter().skip(chars.len().saturating_sub(25)).collect();
                        display_path = format!("{}...{}", prefix, suffix);
                    }
                    let size_str = scanners::format_size(file.size);

                    let content = format!("[{}] {:2}. {:<46} {}", checked_str, file_num, display_path, size_str);

                    if is_current {
                        list_lines.push(Line::from(vec![
                            Span::styled(cursor_str, Style::default().fg(CYAN_THEME).add_modifier(Modifier::BOLD)),
                            Span::styled(content, Style::default().fg(CYAN_THEME).add_modifier(Modifier::BOLD)),
                        ]));
                    } else {
                        list_lines.push(Line::from(vec![
                            Span::raw(cursor_str),
                            Span::styled(content, Style::default().fg(FG_THEME)),
                        ]));
                    }
                }
            }

            let box_block = Block::default()
                .title(" TOP 50 LARGEST FILES IN ~/Documents ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(CYAN_THEME));

            let page_text = if total_pages > 1 {
                format!("Page {} of {} (Items {}-{} of {})", report_page + 1, total_pages, start + 1, end, large_files.len())
            } else {
                String::new()
            };

            let selected_total: u64 = large_files.iter().filter(|f| f.selected).map(|f| f.size).sum();
            let has_selected = selected_total > 0;

            let constraints = if has_selected {
                vec![
                    Constraint::Min(4),
                    Constraint::Length(1),
                    Constraint::Length(3),
                ]
            } else {
                vec![
                    Constraint::Min(4),
                    Constraint::Length(1),
                ]
            };

            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(constraints)
                .split(area);

            f.render_widget(Paragraph::new(list_lines).block(box_block), layout[0]);
            f.render_widget(Paragraph::new(Line::from(Span::styled(page_text, Style::default().fg(YELLOW_THEME)))), layout[1]);

            if has_selected {
                let summary_text = format!("Selected Files to Move to Trash: {}", scanners::format_size(selected_total));
                let summary_block = Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(PURPLE_THEME));
                f.render_widget(Paragraph::new(Line::from(Span::styled(summary_text, Style::default().fg(FG_THEME).add_modifier(Modifier::BOLD)))).block(summary_block), layout[2]);
            }
        }
        AppState::AppsSuggestions => {
            let start = app_page * items_per_page;
            let end = (start + items_per_page).min(app_suggestions.len());
            let total_pages = app_suggestions.len().div_ceil(items_per_page);

            let mut list_lines = Vec::new();
            if app_suggestions.is_empty() {
                list_lines.push(Line::from(Span::styled("No large and unused applications found.", Style::default().fg(GRAY_THEME))));
            } else {
                let now = chrono::Utc::now();
                for (idx, app) in app_suggestions[start..end].iter().enumerate() {
                    let is_current = app_cursor == idx;
                    let cursor_str = if is_current { "> " } else { "  " };

                    let opened_status = if app.never_used {
                        "Never opened".to_string()
                    } else if let Some(last) = app.last_used {
                        format!("{} days ago", (now - last).num_days())
                    } else {
                        "Never opened".to_string()
                    };

                    let content = format!("{:<28} {:<12} {}", app.name, scanners::format_size(app.size), opened_status);

                    if is_current {
                        list_lines.push(Line::from(vec![
                            Span::styled(cursor_str, Style::default().fg(CYAN_THEME).add_modifier(Modifier::BOLD)),
                            Span::styled(content, Style::default().fg(CYAN_THEME).add_modifier(Modifier::BOLD)),
                        ]));
                    } else {
                        list_lines.push(Line::from(vec![
                            Span::raw(cursor_str),
                            Span::styled(content, Style::default().fg(FG_THEME)),
                        ]));
                    }
                }
            }

            let box_block = Block::default()
                .title(" UNUSED / LARGE APPLICATION SUGGESTIONS ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(CYAN_THEME));

            let page_text = if total_pages > 1 {
                format!("Page {} of {} (Apps {}-{} of {})", app_page + 1, total_pages, start + 1, end, app_suggestions.len())
            } else {
                String::new()
            };

            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(4),
                    Constraint::Length(1),
                ])
                .split(area);

            f.render_widget(Paragraph::new(list_lines).block(box_block), layout[0]);
            f.render_widget(Paragraph::new(Line::from(Span::styled(page_text, Style::default().fg(YELLOW_THEME)))), layout[1]);
        }
        AppState::HuggingFace => {
            let start = hf_page * items_per_page;
            let end = (start + items_per_page).min(hf_items.len());
            let total_pages = hf_items.len().div_ceil(items_per_page);

            let mut list_lines = Vec::new();
            if hf_items.is_empty() {
                list_lines.push(Line::from(Span::styled("No Hugging Face models or caches found.", Style::default().fg(GRAY_THEME))));
            } else {
                for (idx, item) in hf_items[start..end].iter().enumerate() {
                    let is_current = hf_cursor == idx;
                    let cursor_str = if is_current { "> " } else { "  " };
                    let content = format!("{:<50} {}", item.name, scanners::format_size(item.size));

                    if is_current {
                        list_lines.push(Line::from(vec![
                            Span::styled(cursor_str, Style::default().fg(CYAN_THEME).add_modifier(Modifier::BOLD)),
                            Span::styled(content, Style::default().fg(CYAN_THEME).add_modifier(Modifier::BOLD)),
                        ]));
                    } else {
                        list_lines.push(Line::from(vec![
                            Span::raw(cursor_str),
                            Span::styled(content, Style::default().fg(FG_THEME)),
                        ]));
                    }
                }
            }

            let box_block = Block::default()
                .title(" HUGGING FACE MODELS & CACHE ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(CYAN_THEME));

            let page_text = if total_pages > 1 {
                format!("Page {} of {} (Items {}-{} of {})", hf_page + 1, total_pages, start + 1, end, hf_items.len())
            } else {
                String::new()
            };

            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(4),
                    Constraint::Length(1),
                ])
                .split(area);

            f.render_widget(Paragraph::new(list_lines).block(box_block), layout[0]);
            f.render_widget(Paragraph::new(Line::from(Span::styled(page_text, Style::default().fg(YELLOW_THEME)))), layout[1]);
        }
        AppState::Ollama => {
            let start = ollama_page * items_per_page;
            let end = (start + items_per_page).min(ollama_items.len());
            let total_pages = ollama_items.len().div_ceil(items_per_page);

            let mut list_lines = Vec::new();
            if ollama_items.is_empty() {
                list_lines.push(Line::from(Span::styled("No Ollama models found.", Style::default().fg(GRAY_THEME))));
            } else {
                for (idx, item) in ollama_items[start..end].iter().enumerate() {
                    let is_current = ollama_cursor == idx;
                    let cursor_str = if is_current { "> " } else { "  " };
                    let content = format!("{:<50} {}", item.name, scanners::format_size(item.size));

                    if is_current {
                        list_lines.push(Line::from(vec![
                            Span::styled(cursor_str, Style::default().fg(CYAN_THEME).add_modifier(Modifier::BOLD)),
                            Span::styled(content, Style::default().fg(CYAN_THEME).add_modifier(Modifier::BOLD)),
                        ]));
                    } else {
                        list_lines.push(Line::from(vec![
                            Span::raw(cursor_str),
                            Span::styled(content, Style::default().fg(FG_THEME)),
                        ]));
                    }
                }
            }

            let box_block = Block::default()
                .title(" OLLAMA MODELS ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(CYAN_THEME));

            let page_text = if total_pages > 1 {
                format!("Page {} of {} (Items {}-{} of {})", ollama_page + 1, total_pages, start + 1, end, ollama_items.len())
            } else {
                String::new()
            };

            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(4),
                    Constraint::Length(1),
                ])
                .split(area);

            f.render_widget(Paragraph::new(list_lines).block(box_block), layout[0]);
            f.render_widget(Paragraph::new(Line::from(Span::styled(page_text, Style::default().fg(YELLOW_THEME)))), layout[1]);
        }
        AppState::Browsers => {
            let start = browsers_page * items_per_page;
            let end = (start + items_per_page).min(browsers_items.len());
            let total_pages = browsers_items.len().div_ceil(items_per_page);

            let mut list_lines = Vec::new();
            if browsers_items.is_empty() {
                list_lines.push(Line::from(Span::styled("No browser caches found.", Style::default().fg(GRAY_THEME))));
            } else {
                for (idx, item) in browsers_items[start..end].iter().enumerate() {
                    let is_current = browsers_cursor == idx;
                    let cursor_str = if is_current { "> " } else { "  " };
                    let checked_str = if item.selected { "x" } else { " " };
                    let content = format!("[{}] {:<50} {}", checked_str, item.name, scanners::format_size(item.size));

                    if is_current {
                        list_lines.push(Line::from(vec![
                            Span::styled(cursor_str, Style::default().fg(CYAN_THEME).add_modifier(Modifier::BOLD)),
                            Span::styled(content, Style::default().fg(CYAN_THEME).add_modifier(Modifier::BOLD)),
                        ]));
                    } else {
                        list_lines.push(Line::from(vec![
                            Span::raw(cursor_str),
                            Span::styled(content, Style::default().fg(FG_THEME)),
                        ]));
                    }
                }
            }

            let box_block = Block::default()
                .title(" BROWSER CACHES ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(CYAN_THEME));

            let page_text = if total_pages > 1 {
                format!("Page {} of {} (Items {}-{} of {})", browsers_page + 1, total_pages, start + 1, end, browsers_items.len())
            } else {
                String::new()
            };

            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(4),
                    Constraint::Length(1),
                ])
                .split(area);

            f.render_widget(Paragraph::new(list_lines).block(box_block), layout[0]);
            f.render_widget(Paragraph::new(Line::from(Span::styled(page_text, Style::default().fg(YELLOW_THEME)))), layout[1]);
        }
        AppState::DevTools => {
            let start = dev_page * items_per_page;
            let end = (start + items_per_page).min(dev_items.len());
            let total_pages = dev_items.len().div_ceil(items_per_page);

            let mut list_lines = Vec::new();
            if dev_items.is_empty() {
                list_lines.push(Line::from(Span::styled("No developer tool caches found.", Style::default().fg(GRAY_THEME))));
            } else {
                for (idx, item) in dev_items[start..end].iter().enumerate() {
                    let is_current = dev_cursor == idx;
                    let cursor_str = if is_current { "> " } else { "  " };
                    let checked_str = if item.selected { "x" } else { " " };
                    let tag = format!("[{}]", item.action_type);
                    let tag_style = if item.action_type.starts_with("CLI") {
                        Style::default().fg(GREEN_THEME)
                    } else {
                        Style::default().fg(YELLOW_THEME)
                    };

                    let size_str = scanners::format_size(item.size);

                    if is_current {
                        list_lines.push(Line::from(vec![
                            Span::styled(cursor_str, Style::default().fg(CYAN_THEME).add_modifier(Modifier::BOLD)),
                            Span::styled(format!(" [{}] ", checked_str), Style::default().fg(CYAN_THEME).add_modifier(Modifier::BOLD)),
                            Span::styled(format!("{:<16} ", tag), tag_style.add_modifier(Modifier::BOLD)),
                            Span::styled(format!("{:<42} ", item.name), Style::default().fg(CYAN_THEME).add_modifier(Modifier::BOLD)),
                            Span::styled(size_str, Style::default().fg(CYAN_THEME).add_modifier(Modifier::BOLD)),
                        ]));
                    } else {
                        list_lines.push(Line::from(vec![
                            Span::raw(cursor_str),
                            Span::styled(format!(" [{}] ", checked_str), Style::default().fg(FG_THEME)),
                            Span::styled(format!("{:<16} ", tag), tag_style),
                            Span::styled(format!("{:<42} ", item.name), Style::default().fg(FG_THEME)),
                            Span::styled(size_str, Style::default().fg(GRAY_THEME)),
                        ]));
                    }
                }
            }

            let selected_count = dev_items.iter().filter(|it| it.selected).count();
            let selected_size: u64 = dev_items.iter().filter(|it| it.selected).map(|it| it.size).sum();

            let title = if selected_count > 0 {
                format!(
                    " DEVELOPER TOOLS CACHES ({} selected • {}) ",
                    selected_count,
                    scanners::format_size(selected_size)
                )
            } else {
                " DEVELOPER TOOLS CACHES ".to_string()
            };

            let box_block = Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(CYAN_THEME));

            let page_text = if total_pages > 1 {
                format!("Page {} of {} (Items {}-{} of {})", dev_page + 1, total_pages, start + 1, end, dev_items.len())
            } else {
                String::new()
            };

            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(4),
                    Constraint::Length(1),
                ])
                .split(area);

            f.render_widget(Paragraph::new(list_lines).block(box_block), layout[0]);
            f.render_widget(Paragraph::new(Line::from(Span::styled(page_text, Style::default().fg(YELLOW_THEME)))), layout[1]);
        }
        AppState::Fonts => {
            let start = font_page * items_per_page;
            let end = (start + items_per_page).min(font_items.len());
            let total_pages = font_items.len().div_ceil(items_per_page);

            let mut list_lines = Vec::new();
            if font_items.is_empty() {
                list_lines.push(Line::from(Span::styled("No custom fonts found in ~/Library/Fonts.", Style::default().fg(GRAY_THEME))));
            } else {
                for (idx, item) in font_items[start..end].iter().enumerate() {
                    let is_current = font_cursor == idx;
                    let cursor_str = if is_current { "> " } else { "  " };
                    let checked_str = if item.selected { "x" } else { " " };
                    let files_str = format!("{} files", item.count);
                    let size_str = scanners::format_size(item.size);

                    if is_current {
                        list_lines.push(Line::from(vec![
                            Span::styled(cursor_str, Style::default().fg(CYAN_THEME).add_modifier(Modifier::BOLD)),
                            Span::styled(format!(" [{}] ", checked_str), Style::default().fg(CYAN_THEME).add_modifier(Modifier::BOLD)),
                            Span::styled(format!("{:<40} ", item.family), Style::default().fg(CYAN_THEME).add_modifier(Modifier::BOLD)),
                            Span::styled(format!("{:<15} ", files_str), Style::default().fg(YELLOW_THEME).add_modifier(Modifier::BOLD)),
                            Span::styled(size_str, Style::default().fg(CYAN_THEME).add_modifier(Modifier::BOLD)),
                        ]));
                    } else {
                        list_lines.push(Line::from(vec![
                            Span::raw(cursor_str),
                            Span::styled(format!(" [{}] ", checked_str), Style::default().fg(FG_THEME)),
                            Span::styled(format!("{:<40} ", item.family), Style::default().fg(FG_THEME)),
                            Span::styled(format!("{:<15} ", files_str), Style::default().fg(YELLOW_THEME)),
                            Span::styled(size_str, Style::default().fg(GRAY_THEME)),
                        ]));
                    }
                }
            }

            let selected_count = font_items.iter().filter(|it| it.selected).count();
            let selected_size: u64 = font_items.iter().filter(|it| it.selected).map(|it| it.size).sum();

            let title = if selected_count > 0 {
                format!(
                    " NERD FONTS OPTIMIZER ({} selected • {}) ",
                    selected_count,
                    scanners::format_size(selected_size)
                )
            } else {
                " NERD FONTS OPTIMIZER (~/Library/Fonts) ".to_string()
            };

            let box_block = Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(CYAN_THEME));

            let page_text = if total_pages > 1 {
                format!("Page {} of {} (Families {}-{} of {})", font_page + 1, total_pages, start + 1, end, font_items.len())
            } else {
                String::new()
            };

            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(4),
                    Constraint::Length(1),
                ])
                .split(area);

            f.render_widget(Paragraph::new(list_lines).block(box_block), layout[0]);
            f.render_widget(Paragraph::new(Line::from(Span::styled(page_text, Style::default().fg(YELLOW_THEME)))), layout[1]);
        }
        AppState::AppSupport => {
            let start = app_support_page * items_per_page;
            let end = (start + items_per_page).min(app_support_items.len());
            let total_pages = app_support_items.len().div_ceil(items_per_page);

            let mut list_lines = Vec::new();
            if app_support_items.is_empty() {
                list_lines.push(Line::from(Span::styled("No Application Support caches or data found.", Style::default().fg(GRAY_THEME))));
            } else {
                for (idx, item) in app_support_items[start..end].iter().enumerate() {
                    let is_current = app_support_cursor == idx;
                    let cursor_str = if is_current { "> " } else { "  " };
                    let checked_str = if item.selected { "x" } else { " " };
                    let status_label = if item.orphan { "Orphaned (Safe)" } else { "Active" };
                    let content = format!("[{}] {:<25} {:<15} {}", checked_str, item.name, status_label, scanners::format_size(item.size));

                    if is_current {
                        list_lines.push(Line::from(vec![
                            Span::styled(cursor_str, Style::default().fg(CYAN_THEME).add_modifier(Modifier::BOLD)),
                            Span::styled(content, Style::default().fg(CYAN_THEME).add_modifier(Modifier::BOLD)),
                        ]));
                    } else {
                        let color = if item.orphan { GREEN_THEME } else { FG_THEME };
                        list_lines.push(Line::from(vec![
                            Span::raw(cursor_str),
                            Span::styled(content, Style::default().fg(color)),
                        ]));
                    }
                }
            }

            let box_block = Block::default()
                .title(" APPLICATION SUPPORT DATA ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(CYAN_THEME));

            let page_text = if total_pages > 1 {
                format!("Page {} of {} (Items {}-{} of {})", app_support_page + 1, total_pages, start + 1, end, app_support_items.len())
            } else {
                String::new()
            };

            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(4),
                    Constraint::Length(1),
                ])
                .split(area);

            f.render_widget(Paragraph::new(list_lines).block(box_block), layout[0]);
            f.render_widget(Paragraph::new(Line::from(Span::styled(page_text, Style::default().fg(YELLOW_THEME)))), layout[1]);
        }
        AppState::Docker => {
            let start = docker_page * items_per_page;
            let end = (start + items_per_page).min(docker_items.len());
            let total_pages = docker_items.len().div_ceil(items_per_page);

            let mut list_lines = Vec::new();
            if docker_items.is_empty() {
                list_lines.push(Line::from(Span::styled("No Docker VM storage files or caches found.", Style::default().fg(GRAY_THEME))));
            } else {
                for (idx, item) in docker_items[start..end].iter().enumerate() {
                    let is_current = docker_cursor == idx;
                    let cursor_str = if is_current { "> " } else { "  " };
                    let content = format!("{:<50} {}", item.name, scanners::format_size(item.size));

                    if is_current {
                        list_lines.push(Line::from(vec![
                            Span::styled(cursor_str, Style::default().fg(CYAN_THEME).add_modifier(Modifier::BOLD)),
                            Span::styled(content, Style::default().fg(CYAN_THEME).add_modifier(Modifier::BOLD)),
                        ]));
                    } else {
                        list_lines.push(Line::from(vec![
                            Span::raw(cursor_str),
                            Span::styled(content, Style::default().fg(FG_THEME)),
                        ]));
                    }
                }
            }

            let box_block = Block::default()
                .title(" DOCKER CLEANUP & CACHES ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(CYAN_THEME));

            let page_text = if total_pages > 1 {
                format!("Page {} of {} (Items {}-{} of {})", docker_page + 1, total_pages, start + 1, end, docker_items.len())
            } else {
                String::new()
            };

            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(4),
                    Constraint::Length(1),
                ])
                .split(area);

            f.render_widget(Paragraph::new(list_lines).block(box_block), layout[0]);
            f.render_widget(Paragraph::new(Line::from(Span::styled(page_text, Style::default().fg(YELLOW_THEME)))), layout[1]);
        }
        AppState::Cleaning => {
            let status = Paragraph::new(Line::from(vec![
                Span::styled(format!("{} ", spinner_char), Style::default().fg(CYAN_THEME).add_modifier(Modifier::BOLD)),
                Span::styled(
                    "Safely cleaning selected components...",
                    Style::default().fg(FG_THEME).add_modifier(Modifier::BOLD),
                ),
            ]));

            let gauge = Gauge::default()
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(PURPLE_THEME))
                        .title(" PROGRESS "),
                )
                .gauge_style(Style::default().fg(CYAN_THEME).bg(PURPLE_THEME))
                .percent((clean_percent * 100.0) as u16);

            // Left-aligned with aesthetic max width (64 columns)
            let bar_width = 64.min(area.width.saturating_sub(2));

            let h_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Length(bar_width),
                    Constraint::Min(1),
                ])
                .split(area);

            let v_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(2),
                    Constraint::Length(3),
                    Constraint::Min(1),
                ])
                .split(h_layout[0]);

            f.render_widget(status, v_layout[0]);
            f.render_widget(gauge, v_layout[1]);
        }
        AppState::ConfirmTrash => {
            let title = Line::from(Span::styled(
                "DELETE CONFIRMATION\n",
                Style::default().fg(RED_THEME).add_modifier(Modifier::BOLD),
            ));

            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(PURPLE_THEME));

            let prompt_p = Paragraph::new(confirm_prompt).block(block);

            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(2),
                    Constraint::Length(10),
                ])
                .split(area);

            f.render_widget(Paragraph::new(title), layout[0]);
            f.render_widget(prompt_p, layout[1]);
        }
        AppState::Finished => {
            let summary = format!(
                "CLEANUP COMPLETED SUCCESSFULLY!\n\nEstimated space freed: {}\n\nAll selected items have been processed.",
                scanners::format_size(freed_size)
            );

            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(GREEN_THEME));

            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(7),
                ])
                .split(area);

            f.render_widget(Paragraph::new(summary).block(block), layout[0]);
        }
    }
}

fn render_spinner_view(f: &mut Frame, area: Rect, spinner: char, text: &str) {
    let line = Line::from(vec![
        Span::styled(format!("{} ", spinner), Style::default().fg(CYAN_THEME).add_modifier(Modifier::BOLD)),
        Span::styled(text, Style::default().fg(GRAY_THEME).add_modifier(Modifier::ITALIC)),
    ]);
    f.render_widget(Paragraph::new(line), area);
}
