## 📂 Project Architecture

Cinder is written in **Rust** using **Ratatui** for terminal UI and **Crossterm** for cross-platform terminal control and event handling.

### Directory Structure

* **`src/`**:
  * [main.rs](file:///Users/ricardo/Project/k-wrk/Cinder/src/main.rs): Application entry point, event loop, panic hooks, and state machine orchestration.
  * [types.rs](file:///Users/ricardo/Project/k-wrk/Cinder/src/types.rs): Application states, events, and core data models (`CleanupItem`, `AppState`, `AppEvent`).
  * [ui.rs](file:///Users/ricardo/Project/k-wrk/Cinder/src/ui.rs): Ratatui rendering pipelines for all views, menus, headers, help bars, and confirmation modals.
  * **`scanners/`**: Self-contained background filesystem scanning modules and cleaning actions:
    * `mod.rs`: Directory sizing with sparse file block calculations, binary availability check, and Trash operations.
    * `apps.rs`: Inactive application detector (>50MB, >30 days unopened).
    * `appsupport.rs`: Application Support directory caches.
    * `browsers.rs`: Caches for Chrome, Arc, Brave, Safari, Firefox, Zen, and DuckDuckGo.
    * `containers.rs`: Whitelist-protected macOS sandboxed containers.
    * `devtools.rs`: Developer toolchain caches (Xcode, Cargo, Go, Python, npm, Homebrew, etc.).
    * `docker.rs`: Docker VM disk images and Docker system prune execution.
    * `documents.rs`: Top 50 largest files scanner in `~/Documents`.
    * `huggingface.rs`: Hugging Face model weight cache.
    * `ollama.rs`: Ollama manifest and model blob scanner.