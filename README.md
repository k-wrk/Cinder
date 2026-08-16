# Cinder CLI

> **Interactive Terminal Utility for macOS Disk Cleanup.**

**Cinder CLI** is a console tool written in **Go** using the **Charm** ecosystem (`bubbletea`, `lipgloss`, and `bubbles`). It was designed to help you free up **System Data** space and organize your **Documents** directory in a 100% safe and interactive way.

## Quick Installation

To install **Cinder CLI** via Homebrew, run:

```bash
brew tap k-wrk/tap
brew install k-wrk/tap/cinder
```

---

## Key Features

* **Safe Cleanup (No Data Loss):**
  * Instead of using the destructive `rm -rf`, all selected files and directories are **moved to the macOS Trash (`~/.Trash`)** so that you can restore them if needed.
  * **Smart Whitelist:** Strict protection based on keywords and application signatures prevents the removal of active system containers or well-known applications (such as Microsoft Office, Docker, Adobe, Slack, etc.).
* **Smart App Suggestions:** Maps applications occupying more than 50MB of disk space that haven't been opened for more than 30 days (or have never been opened!), displaying their size and inactivity duration.
* **Paginated Documents Report:** Lists the 50 largest files in your `~/Documents` folder, organized in pages of 10 items.
* **Local Time Machine Snapshot Deletion:** Integrated, interactive deletion of temporary local disk backups (one of the biggest culprits of *System Data* bloat) with administrator privileges.
* **Developer Caches Cleanup:** Scans and cleans gigabytes of storage accumulated by compilers and package managers such as **Xcode (iOS DeviceSupport, Simulators, and Archives), Go (Modules and Build Cache), Cargo (Rust registry/git), Python (uv, pip, and Poetry), Yarn, npm, JetBrains IDEs, Android SDK, Hugging Face AI, Cypress, Phpactor, CocoaPods, and Homebrew**.
* **General Caches & Logs Cleanup:** Interactive selection to clean everyday application caches and diagnostics, including **Apple Mail Downloads (attachments), Spotify Cache, Telegram Cache, Slack & Discord Caches, macOS Diagnostic/Crash Reports, and User Cache Logs**.
* **Docker Clean:** Cleans up unused Docker containers, images, and volumes, and moves Docker Desktop virtual machine storage to the Trash to regain massive amounts of space.
* **Ollama Models Management:** Lists installed Ollama models and tags, allowing you to move selected models along with their massive weight blobs directly to the Trash.

---

## Installation

To install **Cinder CLI** globally on your macOS system, you can use one of the following methods:

### Method 1: Using `just` (Recommended)
Run the following command in the project root folder to automatically compile and install the utility to `~/.local/bin`:
```bash
just install
```

### Method 2: Manual Installation
Build the executable and move it to a folder in your `$PATH` (like `~/.local/bin`):
```bash
go build -o cinder ./cmd/cinder
mkdir -p ~/.local/bin
mv cinder ~/.local/bin/
```

### Method 3: Direct Installation via Go
If you have Go installed on your machine, you can download, compile, and install the latest version of the binary directly without manually downloading or cloning the code:
```bash
go install github.com/k-wrk/cinder/cmd/cinder@latest
```
*(Make sure `~/go/bin` is in your system's `$PATH`)*.

Once installed, you can launch the interactive cleanup utility from anywhere in your terminal by running:
```bash
cinder
```

---

## How to Build and Run

This project uses [just](https://just.systems/) to facilitate project execution.

### Prerequisites
* **macOS** Operating System
* **Go** programming language installed (version 1.18 or higher)
* **`just`** utility installed (optional, but highly recommended)

### Quick Commands:

* **Build and run the application immediately:**
  ```bash
  just run
  ```

* **Only build the executable in the project root:**
  ```bash
  just build
  ```

* **Remove the built binary from the root:**
  ```bash
  just clean
  ```

* **Organize and download Go dependencies:**
  ```bash
  just tidy
  ```

*(If you don't have `just` installed, you can build manually with the command: `go build -o cinder ./cmd/cinder` and execute it with `./cinder`)*.
