package navigation

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"github.com/k-wrk/cinder/tui/scanners"

	tea "github.com/charmbracelet/bubbletea"
)

func (m model) updateBrowsers(key string) (model, tea.Cmd) {
	switch key {
	case "up", "k":
		if m.browsersCursor > 0 {
			m.browsersCursor--
		}
	case "down", "j":
		start := m.browsersPage * m.itemsPerPage
		end := start + m.itemsPerPage
		if end > len(m.browsersItems) {
			end = len(m.browsersItems)
		}
		pageSize := end - start
		if m.browsersCursor < pageSize-1 {
			m.browsersCursor++
		}
	case "left", "p":
		if m.browsersPage > 0 {
			m.browsersPage--
			m.browsersCursor = 0
		}
	case "right", "n":
		totalPages := (len(m.browsersItems) + m.itemsPerPage - 1) / m.itemsPerPage
		if m.browsersPage < totalPages-1 {
			m.browsersPage++
			m.browsersCursor = 0
		}
	case " ":
		if len(m.browsersItems) > 0 {
			start := m.browsersPage * m.itemsPerPage
			itemIdx := start + m.browsersCursor
			if itemIdx < len(m.browsersItems) {
				m.browsersItems[itemIdx].Selected = !m.browsersItems[itemIdx].Selected
			}
		}
	case "enter":
		var selectedItems []scanners.BrowserItem
		var itemNames []string
		for _, item := range m.browsersItems {
			if item.Selected {
				selectedItems = append(selectedItems, item)
				itemNames = append(itemNames, item.Name)
			}
		}

		if len(selectedItems) == 0 {
			if len(m.browsersItems) > 0 {
				start := m.browsersPage * m.itemsPerPage
				itemIdx := start + m.browsersCursor
				if itemIdx < len(m.browsersItems) {
					selectedItems = append(selectedItems, m.browsersItems[itemIdx])
					itemNames = append(itemNames, m.browsersItems[itemIdx].Name)
				}
			}
		}

		if len(selectedItems) > 0 {
			m.previousState = stateBrowsers
			m.confirmPrompt = fmt.Sprintf("Do you want to move the following browser caches to the trash?\n\n- %s", strings.Join(itemNames, "\n- "))
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
				m.state = stateScanningBrowsers
				return m, tea.Batch(m.scanBrowsers(), m.spinner.Tick)
			}
			m.state = stateConfirmTrash
		}
	case "esc", "b":
		m.state = stateMainMenu
	}
	return m, nil
}
