package navigation

import (
	"fmt"
	"os"
	"os/exec"
	"path/filepath"

	"github.com/k-wrk/cinder/tui/scanners"

	tea "github.com/charmbracelet/bubbletea"
)

func (m model) updateAppsSuggestions(key string) (model, tea.Cmd) {
	switch key {
	case "up", "k":
		if m.appCursor > 0 {
			m.appCursor--
		}
	case "down", "j":
		start := m.appPage * m.itemsPerPage
		end := start + m.itemsPerPage
		if end > len(m.appSuggestions) {
			end = len(m.appSuggestions)
		}
		pageSize := end - start
		if m.appCursor < pageSize-1 {
			m.appCursor++
		}
	case "left", "p":
		if m.appPage > 0 {
			m.appPage--
			m.appCursor = 0
		}
	case "right", "n":
		totalPages := (len(m.appSuggestions) + m.itemsPerPage - 1) / m.itemsPerPage
		if m.appPage < totalPages-1 {
			m.appPage++
			m.appCursor = 0
		}
	case "o":
		if len(m.appSuggestions) > 0 {
			start := m.appPage * m.itemsPerPage
			appIdx := start + m.appCursor
			if appIdx < len(m.appSuggestions) {
				_ = exec.Command("open", "-R", m.appSuggestions[appIdx].Path).Run()
			}
		}
	case "enter":
		if len(m.appSuggestions) > 0 {
			start := m.appPage * m.itemsPerPage
			appIdx := start + m.appCursor
			if appIdx < len(m.appSuggestions) {
				targetApp := m.appSuggestions[appIdx]
				m.previousState = stateAppsSuggestions
				m.confirmPrompt = fmt.Sprintf("Do you want to move the application '%s' (%s) to the trash?", targetApp.Name, scanners.FormatSize(targetApp.Size))
				m.onConfirm = func() (model, tea.Cmd) {
					cmd := exec.Command("osascript", "-e", fmt.Sprintf(`tell application "Finder" to move POSIX file "%s" to trash`, targetApp.Path))
					err := cmd.Run()
					if err != nil {
						home, _ := os.UserHomeDir()
						trashPath := filepath.Join(home, ".Trash", filepath.Base(targetApp.Path))
						if _, err := os.Stat(trashPath); err == nil {
							_ = os.RemoveAll(trashPath)
						}
						_ = os.Rename(targetApp.Path, trashPath)
					}
					m.appSuggestions = append(m.appSuggestions[:appIdx], m.appSuggestions[appIdx+1:]...)
					if m.appCursor >= len(m.appSuggestions)-(m.appPage*m.itemsPerPage) && m.appCursor > 0 {
						m.appCursor--
					}
					m.state = stateAppsSuggestions
					return m, nil
				}
				m.state = stateConfirmTrash
			}
		}
	case "esc", "b":
		m.state = stateMainMenu
	}
	return m, nil
}
