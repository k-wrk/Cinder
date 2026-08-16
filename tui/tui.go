package tui

import (
	"github.com/k-wrk/cinder/tui/navigation"

	tea "github.com/charmbracelet/bubbletea"
)

// InitialModel forwards initialization to the navigation package
func InitialModel() tea.Model {
	return navigation.InitialModel()
}
