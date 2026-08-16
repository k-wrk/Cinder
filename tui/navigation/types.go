package navigation

import (
	"github.com/k-wrk/cinder/tui/scanners"

	"github.com/charmbracelet/bubbles/progress"
	"github.com/charmbracelet/bubbles/spinner"
	tea "github.com/charmbracelet/bubbletea"
)

// model represents the general state of the CLI application
type model struct {
	state          appState
	items          []*CleanupItem
	cursor         int
	selectionPage  int
	menuCursor     int // Main Menu Cursor
	progress       progress.Model
	spinner        spinner.Model // Animated spinner for loading states
	totalSize      int64
	freedSize      int64
	statusMsg      string
	err            error
	width          int
	height         int
	// Documents Report
	largeFiles     []scanners.FileInfo
	reportPage     int
	itemsPerPage   int
	// Step-by-step Cleanup Control
	itemsToClean   []int
	currentCleanIdx int
	// App Suggestions
	appSuggestions []scanners.AppSuggestion
	appCursor      int
	appPage        int
	// Hugging Face Cache
	hfItems        []scanners.HFItem
	hfCursor       int
	hfPage         int
	// Ollama Cache
	ollamaItems    []scanners.OllamaItem
	ollamaCursor   int
	ollamaPage     int
	// Browser Caches
	browsersItems  []scanners.BrowserItem
	browsersCursor int
	browsersPage   int
	// Developer Tools Caches
	devItems       []scanners.DevItem
	devCursor      int
	devPage        int
	// Application Support Data
	appSupportItems  []scanners.AppSupportItem
	appSupportCursor int
	appSupportPage   int
	// Docker Cache
	dockerItems      []scanners.DockerItem
	dockerCursor     int
	dockerPage       int
	// Delete Confirmation
	previousState  appState
	confirmPrompt  string
	onConfirm      func() (model, tea.Cmd)
}

// CleanupItem represents a folder or cache that can be cleaned
type CleanupItem struct {
	ID          string
	Name        string
	Path        string
	Size        int64 // in Bytes
	Scanned     bool
	Selected    bool
	CustomClean func() (string, error) // For special cleanup tasks
}

// appState defines the possible application states
type appState int

const (
	stateScanning appState = iota
	stateMainMenu
	stateSelection
	stateCleaning
	stateFinished
	stateDocumentsReport
	stateScanningReport
	stateScanningApps
	stateAppsSuggestions
	stateScanningHF
	stateHuggingFace
	stateScanningOllama
	stateOllama
	stateScanningBrowsers
	stateBrowsers
	stateScanningDev
	stateDevTools
	stateScanningAppSupport
	stateAppSupport
	stateScanningDocker
	stateDocker
	stateConfirmTrash
)

// Async internal Bubble Tea messages
type scanFinishedMsg struct{}
type cleanFinishedMsg struct {
	freedBytes int64
}
type reportFinishedMsg struct {
	files []scanners.FileInfo
}
type appsFinishedMsg struct {
	apps []scanners.AppSuggestion
}
type hfFinishedMsg struct {
	items []scanners.HFItem
}
type ollamaFinishedMsg struct {
	items []scanners.OllamaItem
}
type browsersFinishedMsg struct {
	items []scanners.BrowserItem
}
type devFinishedMsg struct {
	items []scanners.DevItem
}
type appSupportFinishedMsg struct {
	items []scanners.AppSupportItem
}
type dockerFinishedMsg struct {
	items []scanners.DockerItem
}
type cleanItemFinishedMsg struct {
	index     int
	sizeFreed int64
}
