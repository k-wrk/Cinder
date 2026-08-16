package navigation

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"github.com/k-wrk/cinder/tui/scanners"

	tea "github.com/charmbracelet/bubbletea"
)

func (m model) updateAppSupport(key string) (model, tea.Cmd) {
	switch key {
	case "up", "k":
		if m.appSupportCursor > 0 {
			m.appSupportCursor--
		}
	case "down", "j":
		start := m.appSupportPage * m.itemsPerPage
		end := start + m.itemsPerPage
		if end > len(m.appSupportItems) {
			end = len(m.appSupportItems)
		}
		pageSize := end - start
		if m.appSupportCursor < pageSize-1 {
			m.appSupportCursor++
		}
	case "left", "p":
		if m.appSupportPage > 0 {
			m.appSupportPage--
			m.appSupportCursor = 0
		}
	case "right", "n":
		totalPages := (len(m.appSupportItems) + m.itemsPerPage - 1) / m.itemsPerPage
		if m.appSupportPage < totalPages-1 {
			m.appSupportPage++
			m.appSupportCursor = 0
		}
	case " ":
		if len(m.appSupportItems) > 0 {
			start := m.appSupportPage * m.itemsPerPage
			itemIdx := start + m.appSupportCursor
			if itemIdx < len(m.appSupportItems) {
				m.appSupportItems[itemIdx].Selected = !m.appSupportItems[itemIdx].Selected
			}
		}
	case "enter":
		var selectedItems []scanners.AppSupportItem
		var itemNames []string
		for _, item := range m.appSupportItems {
			if item.Selected {
				selectedItems = append(selectedItems, item)
				itemNames = append(itemNames, item.Name)
			}
		}

		if len(selectedItems) == 0 {
			if len(m.appSupportItems) > 0 {
				start := m.appSupportPage * m.itemsPerPage
				itemIdx := start + m.appSupportCursor
				if itemIdx < len(m.appSupportItems) {
					selectedItems = append(selectedItems, m.appSupportItems[itemIdx])
					itemNames = append(itemNames, m.appSupportItems[itemIdx].Name)
				}
			}
		}

		if len(selectedItems) > 0 {
			m.previousState = stateAppSupport
			m.confirmPrompt = fmt.Sprintf("Do you want to move the following Application Support directories to the trash?\n\n- %s", strings.Join(itemNames, "\n- "))
			m.onConfirm = func() (model, tea.Cmd) {
				home, _ := os.UserHomeDir()
				for _, item := range selectedItems {
					trashPath := filepath.Join(home, ".Trash", filepath.Base(item.Path))
					if _, err := os.Stat(trashPath); err == nil {
						_ = os.RemoveAll(trashPath)
					}
					err := os.Rename(item.Path, trashPath)
					if err != nil {
						_ = os.RemoveAll(item.Path)
					}
				}
				m.state = stateScanningAppSupport
				return m, tea.Batch(m.scanAppSupport(), m.spinner.Tick)
			}
			m.state = stateConfirmTrash
		}
	case "esc", "b":
		m.state = stateMainMenu
	}
	return m, nil
}
