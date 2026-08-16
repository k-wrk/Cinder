package navigation

import (
	"bytes"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"sync"

	"github.com/k-wrk/cinder/tui/scanners"

	tea "github.com/charmbracelet/bubbletea"
)

// cleanOrphanContainers wraps scanners.CleanOrphanContainers
func cleanOrphanContainers() (string, error) {
	return scanners.CleanOrphanContainers()
}

// startScanning executes parallel goroutines to measure disk space
func (m model) startScanning() tea.Cmd {
	return func() tea.Msg {
		var wg sync.WaitGroup
		for i, item := range m.items {
			wg.Add(1)
			go func(idx int, it *CleanupItem) {
				defer wg.Done()
				var size int64
				if it.ID == "containers" {
					size = scanners.GetOrphanContainersSize()
				} else if it.ID == "snapshots" {
					cmd := exec.Command("tmutil", "listlocalsnapshots", "/")
					var out bytes.Buffer
					cmd.Stdout = &out
					_ = cmd.Run()
					lines := strings.Split(strings.TrimSpace(out.String()), "\n")
					count := 0
					for _, line := range lines {
						if strings.Contains(line, "com.apple.TimeMachine") {
							count++
						}
					}
					size = int64(count) * 1500 * 1024 * 1024
				} else if it.ID == "brew" {
					size = 200 * 1024 * 1024
				} else if it.ID == "spotify" {
					size = getSpotifySize()
				} else if it.ID == "slack" {
					size = getSlackSize()
				} else if it.ID == "discord" {
					size = getDiscordSize()
				} else if it.ID == "user_logs" {
					size = getUserLogsSize()
				} else {
					size = scanners.DirSize(it.Path)
				}
				it.Size = size
				it.Scanned = true
			}(i, item)
		}
		wg.Wait()
		return scanFinishedMsg{}
	}
}

// scanDocumentsReport scans the ~/Documents folder for the largest files
func (m model) scanDocumentsReport() tea.Cmd {
	return func() tea.Msg {
		return reportFinishedMsg{files: scanners.ScanDocuments()}
	}
}

// scanAppsSuggestions scans installed apps and checks last usage
func (m model) scanAppsSuggestions() tea.Cmd {
	return func() tea.Msg {
		return appsFinishedMsg{apps: scanners.ScanApps()}
	}
}

// scanHuggingFace scans ~/.cache/huggingface for models and cache items
func (m model) scanHuggingFace() tea.Cmd {
	return func() tea.Msg {
		return hfFinishedMsg{items: scanners.ScanHuggingFace()}
	}
}

// scanOllama scans ~/.ollama/models/manifests for installed models
func (m model) scanOllama() tea.Cmd {
	return func() tea.Msg {
		return ollamaFinishedMsg{items: scanners.ScanOllama()}
	}
}

// deleteOllamaModel wraps scanners.DeleteOllamaModel
func deleteOllamaModel(manifestPath string) error {
	return scanners.DeleteOllamaModel(manifestPath)
}

// scanBrowsers scans browser cache directories
func (m model) scanBrowsers() tea.Cmd {
	return func() tea.Msg {
		return browsersFinishedMsg{items: scanners.ScanBrowsers()}
	}
}

// scanDevTools scans developer cache directories
func (m model) scanDevTools() tea.Cmd {
	return func() tea.Msg {
		return devFinishedMsg{items: scanners.ScanDevTools()}
	}
}

// scanAppSupport scans Library/Application Support subdirectories larger than 10MB
func (m model) scanAppSupport() tea.Cmd {
	return func() tea.Msg {
		return appSupportFinishedMsg{items: scanners.ScanAppSupport()}
	}
}

// scanDocker scans disk for Docker cache items
func (m model) scanDocker() tea.Cmd {
	return func() tea.Msg {
		return dockerFinishedMsg{items: scanners.ScanDocker()}
	}
}

func getSpotifySize() int64 {
	home, _ := os.UserHomeDir()
	return scanners.DirSize(filepath.Join(home, "Library/Caches/com.spotify.client")) +
		scanners.DirSize(filepath.Join(home, "Library/Application Support/Spotify/PersistentCache"))
}

func cleanSpotify() (string, error) {
	home, _ := os.UserHomeDir()
	paths := []string{
		filepath.Join(home, "Library/Caches/com.spotify.client"),
		filepath.Join(home, "Library/Application Support/Spotify/PersistentCache"),
	}
	for _, p := range paths {
		trashPath := filepath.Join(home, ".Trash", filepath.Base(p))
		_ = os.RemoveAll(trashPath)
		_ = os.Rename(p, trashPath)
	}
	return "Spotify caches cleaned", nil
}

func getSlackSize() int64 {
	home, _ := os.UserHomeDir()
	return scanners.DirSize(filepath.Join(home, "Library/Application Support/Slack/Cache")) +
		scanners.DirSize(filepath.Join(home, "Library/Application Support/Slack/Code Cache")) +
		scanners.DirSize(filepath.Join(home, "Library/Caches/com.tinyspeck.slackmacgap"))
}

func cleanSlack() (string, error) {
	home, _ := os.UserHomeDir()
	paths := []string{
		filepath.Join(home, "Library/Application Support/Slack/Cache"),
		filepath.Join(home, "Library/Application Support/Slack/Code Cache"),
		filepath.Join(home, "Library/Caches/com.tinyspeck.slackmacgap"),
	}
	for _, p := range paths {
		trashPath := filepath.Join(home, ".Trash", filepath.Base(p))
		_ = os.RemoveAll(trashPath)
		_ = os.Rename(p, trashPath)
	}
	return "Slack caches cleaned", nil
}

func getDiscordSize() int64 {
	home, _ := os.UserHomeDir()
	return scanners.DirSize(filepath.Join(home, "Library/Application Support/discord/Cache")) +
		scanners.DirSize(filepath.Join(home, "Library/Application Support/discord/Code Cache")) +
		scanners.DirSize(filepath.Join(home, "Library/Caches/com.hnc.Discord"))
}

func cleanDiscord() (string, error) {
	home, _ := os.UserHomeDir()
	paths := []string{
		filepath.Join(home, "Library/Application Support/discord/Cache"),
		filepath.Join(home, "Library/Application Support/discord/Code Cache"),
		filepath.Join(home, "Library/Caches/com.hnc.Discord"),
	}
	for _, p := range paths {
		trashPath := filepath.Join(home, ".Trash", filepath.Base(p))
		_ = os.RemoveAll(trashPath)
		_ = os.Rename(p, trashPath)
	}
	return "Discord caches cleaned", nil
}

func getUserLogsSize() int64 {
	home, _ := os.UserHomeDir()
	logsDir := filepath.Join(home, "Library/Logs")
	diagDir := filepath.Join(home, "Library/Logs/DiagnosticReports")
	logsSize := scanners.DirSize(logsDir)
	diagSize := scanners.DirSize(diagDir)
	if logsSize > diagSize {
		return logsSize - diagSize
	}
	return 0
}

func cleanUserLogs() (string, error) {
	home, _ := os.UserHomeDir()
	logsDir := filepath.Join(home, "Library/Logs")
	entries, err := os.ReadDir(logsDir)
	if err != nil {
		return "", err
	}
	for _, entry := range entries {
		name := entry.Name()
		if name == "DiagnosticReports" {
			continue
		}
		p := filepath.Join(logsDir, name)
		trashPath := filepath.Join(home, ".Trash", name)
		_ = os.RemoveAll(trashPath)
		_ = os.Rename(p, trashPath)
	}
	return "User logs cleaned", nil
}
