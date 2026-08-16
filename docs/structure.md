## 📂 Project Architecture

The code is structured modularly in the following directories:

* **`cmd/cinder/`**:
  * [main.go](file:///Users/ricardo/Project/k-wrk/cinder/cmd/cinder/main.go): Simplified entry point of the program.
* **`tui/`**:
  * [tui.go](file:///Users/ricardo/Project/k-wrk/cinder/tui/tui.go): Coordinates the Bubble Tea event loop initialization.
  * **`navigation/`**: Handles view states, navigation inputs, paging, and rendering layouts.
    * [types.go](file:///Users/ricardo/Project/k-wrk/cinder/tui/navigation/types.go): State models, enums, and Bubble Tea async message signatures.
    * [styles.go](file:///Users/ricardo/Project/k-wrk/cinder/tui/navigation/styles.go): Dracula color palette and visual styling definitions using Lip Gloss.
    * [views.go](file:///Users/ricardo/Project/k-wrk/cinder/tui/navigation/views.go): Interactive terminal renderers for each screen.
    * Feature controllers (e.g., [apps.go](file:///Users/ricardo/Project/k-wrk/cinder/tui/navigation/apps.go), [devtools.go](file:///Users/ricardo/Project/k-wrk/cinder/tui/navigation/devtools.go), [docker.go](file:///Users/ricardo/Project/k-wrk/cinder/tui/navigation/docker.go)): Event handlers for key presses and selection.
  * **`scanners/`**: Self-contained background filesystem scanning engines and clean actions (apps, browsers, devtools, docker, etc.).