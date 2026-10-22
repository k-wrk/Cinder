use std::path::PathBuf;

#[derive(Clone)]
pub struct CleanupItem {
    pub id: String,
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
    pub scanned: bool,
    pub selected: bool,
    pub custom_clean: Option<fn(bool) -> (String, Result<(), String>)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppState {
    Scanning,
    MainMenu,
    Selection,
    Cleaning,
    Finished,
    DocumentsReport,
    ScanningReport,
    ScanningApps,
    AppsSuggestions,
    ScanningHf,
    HuggingFace,
    ScanningOllama,
    Ollama,
    ScanningBrowsers,
    Browsers,
    ScanningDev,
    DevTools,
    ScanningFonts,
    Fonts,
    ScanningAppSupport,
    AppSupport,
    ScanningDocker,
    Docker,
    ConfirmTrash,
}
