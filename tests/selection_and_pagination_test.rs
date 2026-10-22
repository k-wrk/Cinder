use cinder::scanners::devtools::DevItem;
use std::path::PathBuf;

#[test]
fn test_multi_select_toggle_and_size_aggregation() {
    let mut items = vec![
        DevItem {
            id: "brew".to_string(),
            name: "Homebrew".to_string(),
            path: PathBuf::from("/tmp/brew"),
            size: 1024 * 1024 * 500, // 500 MB
            selected: false,
            action_type: "CLI: brew",
            custom_clean: None,
        },
        DevItem {
            id: "xcode".to_string(),
            name: "Xcode DerivedData".to_string(),
            path: PathBuf::from("/tmp/xcode"),
            size: 1024 * 1024 * 1024, // 1 GB
            selected: false,
            action_type: "TRASH",
            custom_clean: None,
        },
        DevItem {
            id: "deno".to_string(),
            name: "Deno Cache".to_string(),
            path: PathBuf::from("/tmp/deno"),
            size: 1024 * 1024 * 200, // 200 MB
            selected: false,
            action_type: "CLI: deno",
            custom_clean: None,
        },
    ];

    // Initial state: nothing selected
    let selected_size: u64 = items.iter().filter(|i| i.selected).map(|i| i.size).sum();
    assert_eq!(selected_size, 0);

    // Simulate toggle first item with [Space]
    items[0].selected = !items[0].selected;
    assert!(items[0].selected);
    let selected_size: u64 = items.iter().filter(|i| i.selected).map(|i| i.size).sum();
    assert_eq!(selected_size, 500 * 1024 * 1024);

    // Simulate toggle all with [a]
    let all_selected = items.iter().all(|i| i.selected);
    assert!(!all_selected);
    for item in items.iter_mut() {
        item.selected = true;
    }
    let selected_size: u64 = items.iter().filter(|i| i.selected).map(|i| i.size).sum();
    let expected_total: u64 = (500 + 1024 + 200) * 1024 * 1024;
    assert_eq!(selected_size, expected_total);

    // Toggle all off with [a]
    for item in items.iter_mut() {
        item.selected = false;
    }
    let selected_size: u64 = items.iter().filter(|i| i.selected).map(|i| i.size).sum();
    assert_eq!(selected_size, 0);
}

#[test]
fn test_pagination_and_cursor_bounds() {
    let items_count: usize = 25;
    let items_per_page: usize = 10;
    let total_pages = items_count.div_ceil(items_per_page);
    assert_eq!(total_pages, 3); // pages 0 (10), 1 (10), 2 (5)

    // Test page navigation
    let mut current_page = 0;
    let mut cursor = 0;

    // Move next page
    if current_page < total_pages - 1 {
        current_page += 1;
        cursor = 0;
    }
    assert_eq!(current_page, 1);
    assert_eq!(cursor, 0);

    // Move next page again (to last page)
    if current_page < total_pages - 1 {
        current_page += 1;
        cursor = 0;
    }
    assert_eq!(current_page, 2);

    // Last page size is 5 items
    let start = current_page * items_per_page;
    let end = (start + items_per_page).min(items_count);
    let page_size = end - start;
    assert_eq!(page_size, 5);

    // Moving down should stop at page_size - 1 (index 4)
    for _ in 0..10 {
        if page_size > 0 && cursor < page_size - 1 {
            cursor += 1;
        }
    }
    assert_eq!(cursor, 4);

    // Attempting next page beyond bounds must be blocked
    if current_page < total_pages - 1 {
        current_page += 1;
    }
    assert_eq!(current_page, 2);
}
