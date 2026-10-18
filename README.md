# Cinder

An interactive terminal utility written in Rust to inspect disk space, analyze caches, and safely clean accumulated system data on macOS using Ratatui and Crossterm.

---

## Features

- **Safe Trash Removal:** All deletions move files and directories to the macOS Trash (`~/.Trash`) rather than permanently deleting them with destructive commands.
- **Top 50 Documents Report:** Paginated report identifying the largest files in `~/Documents`, with interactive selection to move them to Trash.
- **Smart App Suggestions:** Maps applications larger than 50MB that have been inactive for over 30 days or never opened, with shortcut to reveal in Finder (`o`).
- **Developer Caches Cleanup:** Scans caches accumulated by developer toolchains, including Xcode (DerivedData, Simulators, Archives), Cargo, Go build caches, Python (uv, pip, poetry), Gradle, npm, Yarn, CocoaPods, and Homebrew.
- **Caches & Sandboxed Containers:** Safely inspects and clears user caches (Spotify, Slack, Discord, Telegram, Apple Mail Downloads, macOS crash logs) and orphan containers with smart whitelisting.
- **Browser Caches:** Detects and cleans cache stores for Chrome, Arc, Brave, Safari, Firefox, Zen, and DuckDuckGo.
- **Ollama Models & Hugging Face:** Inspects local model weights in `~/.ollama/models` and `~/.cache/huggingface`, allowing quick reclamation of tens of gigabytes.
- **Docker Cleanup:** Inspects VM raw disk images (`Docker.raw`) and triggers `docker system prune` for unused containers, images, and volumes.
- **Time Machine Local Snapshots:** Manages and purges local backup snapshots that consume System Data.
- **Dry-Run Simulation:** Full simulation mode (`--dry-run`) to preview scans and cleanups without touching disk.

---

## Prerequisites

1. **Operating System:** macOS (Apple Silicon or Intel)
2. **Rust & Cargo:** (Rust 1.75+ recommended)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

---

## Installation

### Via Homebrew (macOS)

```bash
brew tap k-wrk/tap
brew install k-wrk/tap/cinder
```

### From Source (Cargo)

Clone the repository and compile the optimized release binary:

```bash
git clone https://github.com/k-wrk/Cinder.git
cd Cinder
cargo build --release
```

To install the binary globally into your system path (`~/.cargo/bin/cinder`):

```bash
cargo install --path .
```

---

## CLI Options

| Flag | Short | Description | Default |
| :--- | :--- | :--- | :--- |
| `--dry-run` | `-n`, `--dry` | Run in simulation mode (skips trash operations and shell commands) | `false` |
| `--help` | `-h` | Print help information | - |

---

## Keybindings & Navigation

| Key | Action |
| :--- | :--- |
| `↑` / `k` | Move cursor up |
| `↓` / `j` | Move cursor down |
| `←` / `p` | Previous page (in paginated lists) |
| `→` / `n` | Next page (in paginated lists) |
| `Space` | Toggle checkbox (Selection, Documents, Browsers, Application Support) |
| `Enter` | Select menu option / Clean selected items / Confirm deletion |
| `o` | Reveal selected application in Finder (in Apps report) |
| `Esc` / `b` | Go back to previous screen or main menu |
| `Ctrl + C` / `q` | Exit application safely |

---

## Usage Examples

### 1. Launch the interactive TUI
```bash
cinder
```

### 2. Run in Dry-Run mode (simulation without deletions)
```bash
cinder --dry-run
```

---

## License

Distributed under the **PolyForm Noncommercial License 1.0.0**.

- Free for personal, academic, educational, and non-commercial research use.
- Commercial use or use for commercial gain is **strictly prohibited** without prior written agreement.
