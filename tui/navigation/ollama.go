package navigation

import (
	"fmt"

	"github.com/k-wrk/cinder/tui/scanners"

	tea "github.com/charmbracelet/bubbletea"
)

func (m model) updateOllama(key string) (model, tea.Cmd) {
	switch key {
	case "up", "k":
		if m.ollamaCursor > 0 {
			m.ollamaCursor--
		}
	case "down", "j":
		start := m.ollamaPage * m.itemsPerPage
		end := start + m.itemsPerPage
		if end > len(m.ollamaItems) {
			end = len(m.ollamaItems)
		}
		pageSize := end - start
		if m.ollamaCursor < pageSize-1 {
			m.ollamaCursor++
		}
	case "left", "p":
		if m.ollamaPage > 0 {
			m.ollamaPage--
			m.ollamaCursor = 0
		}
	case "right", "n":
		totalPages := (len(m.ollamaItems) + m.itemsPerPage - 1) / m.itemsPerPage
		if m.ollamaPage < totalPages-1 {
			m.ollamaPage++
			m.ollamaCursor = 0
		}
	case "enter":
		if len(m.ollamaItems) > 0 {
			start := m.ollamaPage * m.itemsPerPage
			itemIdx := start + m.ollamaCursor
			if itemIdx < len(m.ollamaItems) {
				targetItem := m.ollamaItems[itemIdx]
				m.previousState = stateOllama
				m.confirmPrompt = fmt.Sprintf("Do you want to move the Ollama model '%s' (%s) to the trash?", targetItem.Name, scanners.FormatSize(targetItem.Size))
				m.onConfirm = func() (model, tea.Cmd) {
					_ = deleteOllamaModel(targetItem.Path)
					m.ollamaItems = append(m.ollamaItems[:itemIdx], m.ollamaItems[itemIdx+1:]...)
					if m.ollamaCursor >= len(m.ollamaItems)-(m.ollamaPage*m.itemsPerPage) && m.ollamaCursor > 0 {
						m.ollamaCursor--
					}
					m.state = stateOllama
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
