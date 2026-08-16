package main

import (
	"fmt"
	"os"

	"github.com/k-wrk/cinder/tui"

	tea "github.com/charmbracelet/bubbletea"
)

func main() {
	p := tea.NewProgram(tui.InitialModel())
	if _, err := p.Run(); err != nil {
		fmt.Printf("Error running Cinder CLI: %v\n", err)
		os.Exit(1)
	}
}
