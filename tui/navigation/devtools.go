package navigation

import (
	"fmt"
	"os"
	"path/filepath"

	"github.com/k-wrk/cinder/tui/scanners"

	tea "github.com/charmbracelet/bubbletea"
)

func (m model) updateDevTools(key string) (model, tea.Cmd) {
	switch key {
	case "up", "k":
		if m.devCursor > 0 {
			m.devCursor--
		}
	case "down", "j":
		start := m.devPage * m.itemsPerPage
		end := start + m.itemsPerPage
		if end > len(m.devItems) {
			end = len(m.devItems)
		}
		pageSize := end - start
		if m.devCursor < pageSize-1 {
			m.devCursor++
		}
	case "left", "p":
		if m.devPage > 0 {
			m.devPage--
			m.devCursor = 0
		}
	case "right", "n":
		totalPages := (len(m.devItems) + m.itemsPerPage - 1) / m.itemsPerPage
		if m.devPage < totalPages-1 {
			m.devPage++
			m.devCursor = 0
		}
	case "enter":
		if len(m.devItems) > 0 {
			start := m.devPage * m.itemsPerPage
			itemIdx := start + m.devCursor
			if itemIdx < len(m.devItems) {
				targetItem := m.devItems[itemIdx]
				m.previousState = stateDevTools
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
					m.devItems = append(m.devItems[:itemIdx], m.devItems[itemIdx+1:]...)
					if m.devCursor >= len(m.devItems)-(m.devPage*m.itemsPerPage) && m.devCursor > 0 {
						m.devCursor--
					}
					m.state = stateDevTools
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
