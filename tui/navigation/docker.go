package navigation

import (
	"fmt"
	"os"
	"path/filepath"

	"github.com/k-wrk/cinder/tui/scanners"

	tea "github.com/charmbracelet/bubbletea"
)

func (m model) updateDocker(key string) (model, tea.Cmd) {
	switch key {
	case "up", "k":
		if m.dockerCursor > 0 {
			m.dockerCursor--
		}
	case "down", "j":
		start := m.dockerPage * m.itemsPerPage
		end := start + m.itemsPerPage
		if end > len(m.dockerItems) {
			end = len(m.dockerItems)
		}
		pageSize := end - start
		if m.dockerCursor < pageSize-1 {
			m.dockerCursor++
		}
	case "left", "p":
		if m.dockerPage > 0 {
			m.dockerPage--
			m.dockerCursor = 0
		}
	case "right", "n":
		totalPages := (len(m.dockerItems) + m.itemsPerPage - 1) / m.itemsPerPage
		if m.dockerPage < totalPages-1 {
			m.dockerPage++
			m.dockerCursor = 0
		}
	case "enter":
		if len(m.dockerItems) > 0 {
			start := m.dockerPage * m.itemsPerPage
			itemIdx := start + m.dockerCursor
			if itemIdx < len(m.dockerItems) {
				targetItem := m.dockerItems[itemIdx]
				m.previousState = stateDocker
				m.confirmPrompt = fmt.Sprintf("Do you want to clean/delete '%s' (%s)?", targetItem.Name, scanners.FormatSize(targetItem.Size))
				m.onConfirm = func() (model, tea.Cmd) {
					if targetItem.CustomClean != nil {
						_, _ = targetItem.CustomClean()
					} else {
						home, _ := os.UserHomeDir()
						trashPath := filepath.Join(home, ".Trash", filepath.Base(targetItem.Path))
						if _, err := os.Stat(trashPath); err == nil {
							_ = os.RemoveAll(trashPath)
						}
						err := os.Rename(targetItem.Path, trashPath)
						if err != nil {
							_ = os.RemoveAll(targetItem.Path)
						}
					}
					m.dockerItems = append(m.dockerItems[:itemIdx], m.dockerItems[itemIdx+1:]...)
					if m.dockerCursor >= len(m.dockerItems)-(m.dockerPage*m.itemsPerPage) && m.dockerCursor > 0 {
						m.dockerCursor--
					}
					m.state = stateDocker
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
