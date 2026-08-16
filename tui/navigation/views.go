package navigation

import (
	"fmt"
	"os"
	"strings"
	"time"

	"github.com/k-wrk/cinder/tui/scanners"

	"github.com/charmbracelet/lipgloss"
)

// View draws each state visually on the screen
func (m model) View() string {
	var s strings.Builder

	s.WriteString(titleStyle.Render("🧹 CINDER CLI: NAVIGATION MENU") + "\n\n")

	switch m.state {
	case stateScanning:
		s.WriteString(m.spinner.View() + statusStyle.Render(" Calculating accumulated files on disk...") + "\n\n")

	case stateScanningReport:
		s.WriteString(m.spinner.View() + statusStyle.Render(" Scanning ~/Documents folder to find the largest files...") + "\n\n")

	case stateScanningApps:
		s.WriteString(m.spinner.View() + statusStyle.Render(" Scanning installed applications and measuring usage...") + "\n\n")

	case stateScanningHF:
		s.WriteString(m.spinner.View() + statusStyle.Render(" Scanning Hugging Face models and caches...") + "\n\n")

	case stateScanningOllama:
		s.WriteString(m.spinner.View() + statusStyle.Render(" Scanning Ollama models...") + "\n\n")

	case stateScanningBrowsers:
		s.WriteString(m.spinner.View() + statusStyle.Render(" Scanning browser caches...") + "\n\n")

	case stateScanningDev:
		s.WriteString(m.spinner.View() + statusStyle.Render(" Scanning developer tool caches...") + "\n\n")

	case stateScanningAppSupport:
		s.WriteString(m.spinner.View() + statusStyle.Render(" Scanning Application Support caches and data...") + "\n\n")

	case stateScanningDocker:
		s.WriteString(m.spinner.View() + statusStyle.Render(" Scanning Docker VM storage files and caches...") + "\n\n")

	case stateMainMenu:
		s.WriteString("Select a tool option:\n\n")

		var menuText strings.Builder
		for i, opt := range menuOptions {
			cursorStr := "  "
			if m.menuCursor == i {
				cursorStr = cyanStyle("> ")
				menuText.WriteString(fmt.Sprintf("%s%s\n", cursorStr, selectedItemStyle.Render(opt)))
			} else {
				menuText.WriteString(fmt.Sprintf("%s%s\n", cursorStr, itemStyle.Render(opt)))
			}
		}
		s.WriteString(menuBoxStyle.Render(menuText.String()) + "\n\n")

		if m.statusMsg != "" {
			s.WriteString(lipgloss.NewStyle().Foreground(greenTheme).Bold(true).Render(m.statusMsg) + "\n\n")
		}

		s.WriteString(helpStyle.Render("↑/↓: Navigate • [Enter]: Choose Option • [q]: Exit"))

	case stateSelection:
		s.WriteString(lipgloss.NewStyle().Foreground(cyanTheme).Bold(true).Render("🧹 SELECT AND CLEAN CACHES / CONTAINERS") + "\n\n")

		start := m.selectionPage * m.itemsPerPage
		end := start + m.itemsPerPage
		if end > len(m.items) {
			end = len(m.items)
		}

		totalPages := (len(m.items) + m.itemsPerPage - 1) / m.itemsPerPage

		var currentTotal int64
		for _, item := range m.items {
			if item.Selected {
				currentTotal += item.Size
			}
		}

		var reportText strings.Builder
		for idx, item := range m.items[start:end] {
			cursorStr := "  "
			if m.cursor == idx {
				cursorStr = cyanStyle("> ")
			}

			checked := " "
			if item.Selected {
				checked = "x"
			}

			var row string
			if m.cursor == idx {
				row = selectedItemStyle.Render(fmt.Sprintf("[%s] %-50s %s", checked, item.Name, scanners.FormatSize(item.Size)))
			} else {
				row = itemStyle.Render(fmt.Sprintf("[%s] %-50s %s", checked, item.Name, scanners.FormatSize(item.Size)))
			}

			reportText.WriteString(fmt.Sprintf("%s%s\n", cursorStr, row))
		}

		s.WriteString(reportBoxStyle.Render(reportText.String()) + "\n\n")

		if totalPages > 1 {
			s.WriteString(lipgloss.NewStyle().Foreground(yellowTheme).Render(fmt.Sprintf("Page %d of %d (Items %d-%d of %d)", m.selectionPage+1, totalPages, start+1, end, len(m.items))) + "\n\n")
		}

		summary := fmt.Sprintf("Estimated Space to Free: %s", scanners.FormatSize(currentTotal))
		s.WriteString(boxStyle.Render(summary) + "\n\n")

		s.WriteString(helpStyle.Render("↑/↓: Navigate • [Space]: Select • [Enter]: Clean Selected • [Esc/b]: Back"))

	case stateDocumentsReport:
		s.WriteString(lipgloss.NewStyle().Foreground(cyanTheme).Bold(true).Render("📊 TOP 50 LARGEST FILES IN ~/Documents") + "\n\n")

		if len(m.largeFiles) == 0 {
			s.WriteString(statusStyle.Render("No large files found in ~/Documents.") + "\n")
		} else {
			start := m.reportPage * m.itemsPerPage
			end := start + m.itemsPerPage
			if end > len(m.largeFiles) {
				end = len(m.largeFiles)
			}

			totalPages := (len(m.largeFiles) + m.itemsPerPage - 1) / m.itemsPerPage

			var reportText strings.Builder
			for idx, file := range m.largeFiles[start:end] {
				fileNum := start + idx + 1
				displayPath := file.Path
				home, _ := os.UserHomeDir()
				displayPath = strings.Replace(displayPath, home, "~", 1)
				if len(displayPath) > 50 {
					displayPath = displayPath[:20] + "..." + displayPath[len(displayPath)-27:]
				}

				reportText.WriteString(fmt.Sprintf("%2d. %-50s %s\n", fileNum, itemStyle.Render(displayPath), sizeStyle.Render(scanners.FormatSize(file.Size))))
			}

			s.WriteString(reportBoxStyle.Render(reportText.String()) + "\n\n")
			s.WriteString(lipgloss.NewStyle().Foreground(yellowTheme).Render(fmt.Sprintf("Page %d of %d (Items %d-%d of %d)", m.reportPage+1, totalPages, start+1, end, len(m.largeFiles))) + "\n")
		}

		s.WriteString(helpStyle.Render("←/p: Prev Page • →/n: Next Page • [Esc/b]: Back to Menu • [q]: Exit"))

	case stateAppsSuggestions:
		s.WriteString(lipgloss.NewStyle().Foreground(cyanTheme).Bold(true).Render("📱 UNUSED / LARGE APPLICATION SUGGESTIONS") + "\n\n")

		if len(m.appSuggestions) == 0 {
			s.WriteString(statusStyle.Render("No large and unused applications found.") + "\n")
		} else {
			start := m.appPage * m.itemsPerPage
			end := start + m.itemsPerPage
			if end > len(m.appSuggestions) {
				end = len(m.appSuggestions)
			}

			totalPages := (len(m.appSuggestions) + m.itemsPerPage - 1) / m.itemsPerPage

			var reportText strings.Builder
			for idx, app := range m.appSuggestions[start:end] {
				cursorStr := "  "
				if m.appCursor == idx {
					cursorStr = cyanStyle("> ")
				}

				openedStatus := "Never opened"
				if !app.NeverUsed {
					openedStatus = fmt.Sprintf("%d days ago", int(time.Since(app.LastUsed).Hours()/24))
				}

				var row string
				if m.appCursor == idx {
					row = selectedItemStyle.Render(fmt.Sprintf("%-28s %-12s %s", app.Name, scanners.FormatSize(app.Size), openedStatus))
				} else {
					row = itemStyle.Render(fmt.Sprintf("%-28s %-12s %s", app.Name, scanners.FormatSize(app.Size), openedStatus))
				}

				reportText.WriteString(fmt.Sprintf("%s%s\n", cursorStr, row))
			}

			s.WriteString(reportBoxStyle.Render(reportText.String()) + "\n\n")
			s.WriteString(lipgloss.NewStyle().Foreground(yellowTheme).Render(fmt.Sprintf("Page %d of %d (Apps %d-%d of %d)", m.appPage+1, totalPages, start+1, end, len(m.appSuggestions))) + "\n")
		}

		s.WriteString(helpStyle.Render("↑/↓: Navigate • [Enter]: Move to Trash • [o]: Reveal in Finder • [Esc/b]: Back"))

	case stateHuggingFace:
		s.WriteString(lipgloss.NewStyle().Foreground(cyanTheme).Bold(true).Render("🤗 HUGGING FACE MODELS & CACHE") + "\n\n")

		if len(m.hfItems) == 0 {
			s.WriteString(statusStyle.Render("No Hugging Face models or caches found.") + "\n")
		} else {
			start := m.hfPage * m.itemsPerPage
			end := start + m.itemsPerPage
			if end > len(m.hfItems) {
				end = len(m.hfItems)
			}

			totalPages := (len(m.hfItems) + m.itemsPerPage - 1) / m.itemsPerPage

			var reportText strings.Builder
			for idx, item := range m.hfItems[start:end] {
				cursorStr := "  "
				if m.hfCursor == idx {
					cursorStr = cyanStyle("> ")
				}

				var row string
				if m.hfCursor == idx {
					row = selectedItemStyle.Render(fmt.Sprintf("%-50s %s", item.Name, scanners.FormatSize(item.Size)))
				} else {
					row = itemStyle.Render(fmt.Sprintf("%-50s %s", item.Name, scanners.FormatSize(item.Size)))
				}

				reportText.WriteString(fmt.Sprintf("%s%s\n", cursorStr, row))
			}

			s.WriteString(reportBoxStyle.Render(reportText.String()) + "\n\n")
			s.WriteString(lipgloss.NewStyle().Foreground(yellowTheme).Render(fmt.Sprintf("Page %d of %d (Items %d-%d of %d)", m.hfPage+1, totalPages, start+1, end, len(m.hfItems))) + "\n")
		}

		s.WriteString(helpStyle.Render("↑/↓: Navigate • [Enter]: Move to Trash • [Esc/b]: Back"))

	case stateOllama:
		s.WriteString(lipgloss.NewStyle().Foreground(cyanTheme).Bold(true).Render("🦙 OLLAMA MODELS") + "\n\n")

		if len(m.ollamaItems) == 0 {
			s.WriteString(statusStyle.Render("No Ollama models found.") + "\n")
		} else {
			start := m.ollamaPage * m.itemsPerPage
			end := start + m.itemsPerPage
			if end > len(m.ollamaItems) {
				end = len(m.ollamaItems)
			}

			totalPages := (len(m.ollamaItems) + m.itemsPerPage - 1) / m.itemsPerPage

			var reportText strings.Builder
			for idx, item := range m.ollamaItems[start:end] {
				cursorStr := "  "
				if m.ollamaCursor == idx {
					cursorStr = cyanStyle("> ")
				}

				var row string
				if m.ollamaCursor == idx {
					row = selectedItemStyle.Render(fmt.Sprintf("%-50s %s", item.Name, scanners.FormatSize(item.Size)))
				} else {
					row = itemStyle.Render(fmt.Sprintf("%-50s %s", item.Name, scanners.FormatSize(item.Size)))
				}

				reportText.WriteString(fmt.Sprintf("%s%s\n", cursorStr, row))
			}

			s.WriteString(reportBoxStyle.Render(reportText.String()) + "\n\n")
			s.WriteString(lipgloss.NewStyle().Foreground(yellowTheme).Render(fmt.Sprintf("Page %d of %d (Items %d-%d of %d)", m.ollamaPage+1, totalPages, start+1, end, len(m.ollamaItems))) + "\n")
		}

		s.WriteString(helpStyle.Render("↑/↓: Navigate • [Enter]: Move to Trash • [Esc/b]: Back"))

	case stateBrowsers:
		s.WriteString(lipgloss.NewStyle().Foreground(cyanTheme).Bold(true).Render("🌐 BROWSER CACHES") + "\n\n")

		if len(m.browsersItems) == 0 {
			s.WriteString(statusStyle.Render("No browser caches found.") + "\n")
		} else {
			start := m.browsersPage * m.itemsPerPage
			end := start + m.itemsPerPage
			if end > len(m.browsersItems) {
				end = len(m.browsersItems)
			}

			totalPages := (len(m.browsersItems) + m.itemsPerPage - 1) / m.itemsPerPage

			var reportText strings.Builder
			for idx, item := range m.browsersItems[start:end] {
				cursorStr := "  "
				if m.browsersCursor == idx {
					cursorStr = cyanStyle("> ")
				}

				checked := " "
				if item.Selected {
					checked = "x"
				}

				var row string
				if m.browsersCursor == idx {
					row = selectedItemStyle.Render(fmt.Sprintf("[%s] %-50s %s", checked, item.Name, scanners.FormatSize(item.Size)))
				} else {
					row = itemStyle.Render(fmt.Sprintf("[%s] %-50s %s", checked, item.Name, scanners.FormatSize(item.Size)))
				}

				reportText.WriteString(fmt.Sprintf("%s%s\n", cursorStr, row))
			}

			s.WriteString(reportBoxStyle.Render(reportText.String()) + "\n\n")
			s.WriteString(lipgloss.NewStyle().Foreground(yellowTheme).Render(fmt.Sprintf("Page %d of %d (Items %d-%d of %d)", m.browsersPage+1, totalPages, start+1, end, len(m.browsersItems))) + "\n")
		}

		s.WriteString(helpStyle.Render("↑/↓: Navigate • [Space]: Select • [Enter]: Clean Selected • [Esc/b]: Back"))

	case stateDevTools:
		s.WriteString(lipgloss.NewStyle().Foreground(cyanTheme).Bold(true).Render("🛠️ DEVELOPER TOOLS CACHES") + "\n\n")

		if len(m.devItems) == 0 {
			s.WriteString(statusStyle.Render("No developer tool caches found.") + "\n")
		} else {
			start := m.devPage * m.itemsPerPage
			end := start + m.itemsPerPage
			if end > len(m.devItems) {
				end = len(m.devItems)
			}

			totalPages := (len(m.devItems) + m.itemsPerPage - 1) / m.itemsPerPage

			var reportText strings.Builder
			for idx, item := range m.devItems[start:end] {
				cursorStr := "  "
				if m.devCursor == idx {
					cursorStr = cyanStyle("> ")
				}

				var row string
				if m.devCursor == idx {
					row = selectedItemStyle.Render(fmt.Sprintf("%-50s %s", item.Name, scanners.FormatSize(item.Size)))
				} else {
					row = itemStyle.Render(fmt.Sprintf("%-50s %s", item.Name, scanners.FormatSize(item.Size)))
				}

				reportText.WriteString(fmt.Sprintf("%s%s\n", cursorStr, row))
			}

			s.WriteString(reportBoxStyle.Render(reportText.String()) + "\n\n")
			s.WriteString(lipgloss.NewStyle().Foreground(yellowTheme).Render(fmt.Sprintf("Page %d of %d (Items %d-%d of %d)", m.devPage+1, totalPages, start+1, end, len(m.devItems))) + "\n")
		}

		s.WriteString(helpStyle.Render("↑/↓: Navigate • [Enter]: Clean • [Esc/b]: Back"))

	case stateAppSupport:
		s.WriteString(lipgloss.NewStyle().Foreground(cyanTheme).Bold(true).Render("📁 APPLICATION SUPPORT DATA") + "\n\n")

		if len(m.appSupportItems) == 0 {
			s.WriteString(statusStyle.Render("No Application Support caches or data found.") + "\n")
		} else {
			start := m.appSupportPage * m.itemsPerPage
			end := start + m.itemsPerPage
			if end > len(m.appSupportItems) {
				end = len(m.appSupportItems)
			}

			totalPages := (len(m.appSupportItems) + m.itemsPerPage - 1) / m.itemsPerPage

			var reportText strings.Builder
			for idx, item := range m.appSupportItems[start:end] {
				cursorStr := "  "
				if m.appSupportCursor == idx {
					cursorStr = cyanStyle("> ")
				}

				checked := " "
				if item.Selected {
					checked = "x"
				}

				statusLabel := "Active"
				if item.Orphan {
					statusLabel = "Orphaned (Safe)"
				}

				var row string
				rowContent := fmt.Sprintf("[%s] %-25s %-15s %s", checked, item.Name, statusLabel, scanners.FormatSize(item.Size))
				if m.appSupportCursor == idx {
					row = selectedItemStyle.Render(rowContent)
				} else {
					if item.Orphan {
						row = lipgloss.NewStyle().Foreground(greenTheme).Render(rowContent)
					} else {
						row = itemStyle.Render(rowContent)
					}
				}

				reportText.WriteString(fmt.Sprintf("%s%s\n", cursorStr, row))
			}

			s.WriteString(reportBoxStyle.Render(reportText.String()) + "\n\n")
			s.WriteString(lipgloss.NewStyle().Foreground(yellowTheme).Render(fmt.Sprintf("Page %d of %d (Items %d-%d of %d)", m.appSupportPage+1, totalPages, start+1, end, len(m.appSupportItems))) + "\n")
		}

		s.WriteString(helpStyle.Render("↑/↓: Navigate • [Space]: Select • [Enter]: Clean Selected • [Esc/b]: Back"))

	case stateDocker:
		s.WriteString(lipgloss.NewStyle().Foreground(cyanTheme).Bold(true).Render("🐳 DOCKER CLEANUP & CACHES") + "\n\n")

		if len(m.dockerItems) == 0 {
			s.WriteString(statusStyle.Render("No Docker VM storage files or caches found.") + "\n")
		} else {
			start := m.dockerPage * m.itemsPerPage
			end := start + m.itemsPerPage
			if end > len(m.dockerItems) {
				end = len(m.dockerItems)
			}

			totalPages := (len(m.dockerItems) + m.itemsPerPage - 1) / m.itemsPerPage

			var reportText strings.Builder
			for idx, item := range m.dockerItems[start:end] {
				cursorStr := "  "
				if m.dockerCursor == idx {
					cursorStr = cyanStyle("> ")
				}

				var row string
				if m.dockerCursor == idx {
					row = selectedItemStyle.Render(fmt.Sprintf("%-50s %s", item.Name, scanners.FormatSize(item.Size)))
				} else {
					row = itemStyle.Render(fmt.Sprintf("%-50s %s", item.Name, scanners.FormatSize(item.Size)))
				}

				reportText.WriteString(fmt.Sprintf("%s%s\n", cursorStr, row))
			}

			s.WriteString(reportBoxStyle.Render(reportText.String()) + "\n\n")
			s.WriteString(lipgloss.NewStyle().Foreground(yellowTheme).Render(fmt.Sprintf("Page %d of %d (Items %d-%d of %d)", m.dockerPage+1, totalPages, start+1, end, len(m.dockerItems))) + "\n")
		}

		s.WriteString(helpStyle.Render("↑/↓: Navigate • [Enter]: Clean • [Esc/b]: Back"))

	case stateCleaning:
		s.WriteString(statusStyle.Render("Safely cleaning selected components...") + "\n\n")
		s.WriteString(m.progress.View() + "\n")

	case stateConfirmTrash:
		s.WriteString(lipgloss.NewStyle().Foreground(redTheme).Bold(true).Render("⚠️ DELETE CONFIRMATION") + "\n\n")
		s.WriteString(boxStyle.Render(m.confirmPrompt) + "\n\n")
		s.WriteString(helpStyle.Render("[S/Enter]: Yes, continue • [N/Esc/b]: Cancel and go back"))

	case stateFinished:
		freedSummary := fmt.Sprintf("✨ SUCCESS! CLEANUP COMPLETED!\n\nEstimated space freed: %s\n\nAll selected items have been cleaned!", scanners.FormatSize(m.freedSize))
		s.WriteString(doneBoxStyle.Render(freedSummary) + "\n\n")
		s.WriteString(helpStyle.Render("[Esc/Enter/b]: Navigation Menu • [q]: Exit"))
	}

	return s.String()
}
