package navigation

import (
	"fmt"
	"os"
	"path/filepath"

	"github.com/k-wrk/cinder/tui/scanners"

	tea "github.com/charmbracelet/bubbletea"
)

func (m model) updateHuggingFace(key string) (model, tea.Cmd) {
	switch key {
	case "up", "k":
		if m.hfCursor > 0 {
			m.hfCursor--
		}
	case "down", "j":
		start := m.hfPage * m.itemsPerPage
		end := start + m.itemsPerPage
		if end > len(m.hfItems) {
			end = len(m.hfItems)
		}
		pageSize := end - start
		if m.hfCursor < pageSize-1 {
			m.hfCursor++
		}
	case "left", "p":
		if m.hfPage > 0 {
			m.hfPage--
			m.hfCursor = 0
		}
	case "right", "n":
		totalPages := (len(m.hfItems) + m.itemsPerPage - 1) / m.itemsPerPage
		if m.hfPage < totalPages-1 {
			m.hfPage++
			m.hfCursor = 0
		}
	case "enter":
		if len(m.hfItems) > 0 {
			start := m.hfPage * m.itemsPerPage
			itemIdx := start + m.hfCursor
			if itemIdx < len(m.hfItems) {
				targetItem := m.hfItems[itemIdx]
				m.previousState = stateHuggingFace
				m.confirmPrompt = fmt.Sprintf("Do you want to move the Hugging Face item/model '%s' (%s) to the trash?", targetItem.Name, scanners.FormatSize(targetItem.Size))
				m.onConfirm = func() (model, tea.Cmd) {
					home, _ := os.UserHomeDir()
					trashPath := filepath.Join(home, ".Trash", filepath.Base(targetItem.Path))
					if _, err := os.Stat(trashPath); err == nil {
						_ = os.RemoveAll(trashPath)
					}
					err := os.Rename(targetItem.Path, trashPath)
					if err != nil {
						_ = os.RemoveAll(targetItem.Path)
					}
					m.hfItems = append(m.hfItems[:itemIdx], m.hfItems[itemIdx+1:]...)
					if m.hfCursor >= len(m.hfItems)-(m.hfPage*m.itemsPerPage) && m.hfCursor > 0 {
						m.hfCursor--
					}
					m.state = stateHuggingFace
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
