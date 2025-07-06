// Custom Shortcuts Dialog - Modern GTK4 ShortcutsWindow-style design
// Built with CSS styling and custom widgets for a polished look

use gtk4::prelude::*;
use gtk4::{Orientation, Align, Grid, Label, Dialog, ScrolledWindow, Box, Button, Stack, StackSidebar, Separator};
use crate::language;

/// Show a modern shortcuts dialog with section navigation
pub fn show_shortcuts_dialog(parent: &gtk4::Window) {
    let dialog = Dialog::new();
    dialog.set_transient_for(Some(parent));
    dialog.set_modal(true);
    dialog.set_title(Some(&language::tr("shortcuts.title")));
    dialog.set_default_size(900, 650);
    dialog.set_resizable(true);
    
    // Apply custom CSS styling
    apply_shortcuts_dialog_css();
    
    // Create main container
    let main_box = Box::new(Orientation::Vertical, 0);
    main_box.add_css_class("shortcuts-window");
    
    // Create header with title and close button
    let header = create_header_bar(&dialog);
    main_box.append(&header);
    
    // Create content area with sidebar navigation
    let content_box = Box::new(Orientation::Horizontal, 0);
    content_box.set_vexpand(true);
    content_box.add_css_class("shortcuts-content");
    
    // Create stack for different sections
    let stack = Stack::new();
    stack.set_transition_type(gtk4::StackTransitionType::SlideLeftRight);
    stack.set_transition_duration(200);
    
    // Create sidebar for navigation
    let sidebar = StackSidebar::new();
    sidebar.set_stack(&stack);
    sidebar.set_width_request(200);
    sidebar.add_css_class("shortcuts-sidebar");
    
    // Add sections to the stack
    add_editing_section(&stack);
    add_formatting_section(&stack);
    add_insert_section(&stack);
    add_view_section(&stack);
    add_advanced_section(&stack);
    
    // Add sidebar and stack to content
    content_box.append(&sidebar);
    content_box.append(&stack);
    
    main_box.append(&content_box);
    
    // Set up dialog
    dialog.set_child(Some(&main_box));
    
    // Connect close button
    dialog.connect_response(|dialog, _| {
        dialog.close();
    });
    
    dialog.present();
}

/// Create a modern header bar for the shortcuts dialog
fn create_header_bar(dialog: &Dialog) -> Box {
    let header = Box::new(Orientation::Horizontal, 0);
    header.set_height_request(48);
    header.add_css_class("shortcuts-header");
    
    // Title
    let title_label = Label::new(Some(&language::tr("shortcuts.title")));
    title_label.add_css_class("shortcuts-title");
    title_label.set_halign(Align::Start);
    title_label.set_hexpand(true);
    header.append(&title_label);
    
    // Close button
    let close_button = Button::with_label("✕");
    close_button.add_css_class("shortcuts-close");
    close_button.set_halign(Align::End);
    
    {
        let dialog_weak = dialog.downgrade();
        close_button.connect_clicked(move |_| {
            if let Some(dialog) = dialog_weak.upgrade() {
                dialog.close();
            }
        });
    }
    
    header.append(&close_button);
    header
}

/// Add editing section (File, Edit operations)
fn add_editing_section(stack: &Stack) {
    let section = create_section_container();
    
    // File Operations Group
    let file_group = create_shortcut_group(
        "File Operations",
        &[
            ("Ctrl+N", "New Document"),
            ("Ctrl+O", "Open Document"),
            ("Ctrl+S", "Save Document"),
            ("Ctrl+Shift+S", "Save As..."),
            ("Ctrl+Q", "Quit Application"),
        ]
    );
    section.append(&file_group);
    
    // Edit Operations Group
    let edit_group = create_shortcut_group(
        "Edit Operations",
        &[
            ("Ctrl+Z", "Undo"),
            ("Ctrl+Y", "Redo"),
            ("Ctrl+X", "Cut"),
            ("Ctrl+C", "Copy"),
            ("Ctrl+V", "Paste"),
            ("Ctrl+A", "Select All"),
            ("Ctrl+F", "Find"),
            ("Ctrl+H", "Replace"),
        ]
    );
    section.append(&edit_group);
    
    // Create scrolled window for this section
    let section_scroll = create_scrolled_section(&section);
    stack.add_titled(&section_scroll, Some("editing"), "Editing");
}

/// Add formatting section
fn add_formatting_section(stack: &Stack) {
    let section = create_section_container();
    
    // Basic Formatting Group
    let basic_group = create_shortcut_group(
        &language::tr("shortcuts.basic_formatting"),
        &[
            (&language::tr("shortcuts.ctrl_b"), &language::tr("shortcuts.bold_text")),
            (&language::tr("shortcuts.ctrl_i"), &language::tr("shortcuts.italic_text")),
            (&language::tr("shortcuts.ctrl_u"), &language::tr("shortcuts.strikethrough_text")),
            (&language::tr("shortcuts.ctrl_backtick"), &language::tr("shortcuts.inline_code")),
        ]
    );
    section.append(&basic_group);
    
    // Headings Group
    let headings_group = create_shortcut_group(
        &language::tr("shortcuts.headings"),
        &[
            (&language::tr("shortcuts.ctrl_1"), &language::tr("shortcuts.heading_1")),
            (&language::tr("shortcuts.ctrl_2"), &language::tr("shortcuts.heading_2")),
            (&language::tr("shortcuts.ctrl_3"), &language::tr("shortcuts.heading_3")),
            (&language::tr("shortcuts.ctrl_4"), &language::tr("shortcuts.heading_4")),
            (&language::tr("shortcuts.ctrl_5"), &language::tr("shortcuts.heading_5")),
            (&language::tr("shortcuts.ctrl_6"), &language::tr("shortcuts.heading_6")),
        ]
    );
    section.append(&headings_group);
    
    // Lists and Quotes Group
    let lists_group = create_shortcut_group(
        &language::tr("shortcuts.lists_and_quotes"),
        &[
            (&language::tr("shortcuts.ctrl_shift_8"), &language::tr("shortcuts.bullet_list")),
            (&language::tr("shortcuts.ctrl_shift_7"), &language::tr("shortcuts.numbered_list")),
            (&language::tr("shortcuts.ctrl_shift_period"), &language::tr("shortcuts.blockquote")),
        ]
    );
    section.append(&lists_group);
    
    // Create scrolled window for this section
    let section_scroll = create_scrolled_section(&section);
    stack.add_titled(&section_scroll, Some("formatting"), "Formatting");
}

/// Add insert section
fn add_insert_section(stack: &Stack) {
    let section = create_section_container();
    
    // Insert Elements Group
    let insert_group = create_shortcut_group(
        "Insert Elements",
        &[
            (&language::tr("shortcuts.ctrl_k"), &language::tr("shortcuts.insert_link")),
            ("Ctrl+Shift+I", "Insert Image"),
            ("Ctrl+Shift+T", "Insert Table"),
            ("Ctrl+Shift+H", "Insert Horizontal Rule"),
            ("Ctrl+.", "Insert Emoji"),
        ]
    );
    section.append(&insert_group);
    
    // Code Blocks Group
    let code_group = create_shortcut_group(
        "Code Blocks",
        &[
            ("Ctrl+Shift+C", "Insert Code Block"),
            ("Ctrl+Shift+F", "Insert Fenced Code Block"),
            ("Ctrl+Shift+`", "Toggle Inline Code"),
        ]
    );
    section.append(&code_group);
    
    // Create scrolled window for this section
    let section_scroll = create_scrolled_section(&section);
    stack.add_titled(&section_scroll, Some("insert"), "Insert");
}

/// Add view section
fn add_view_section(stack: &Stack) {
    let section = create_section_container();
    
    // View Operations Group
    let view_group = create_shortcut_group(
        "View Operations",
        &[
            ("F5", "Switch to HTML Preview"),
            ("F6", "Switch to Code Preview"),
            ("F4", "Show Context Menu"),
            ("Shift+F10", "Show Context Menu at Cursor"),
            ("Ctrl+Plus", "Zoom In"),
            ("Ctrl+Minus", "Zoom Out"),
            ("Ctrl+0", "Reset Zoom"),
        ]
    );
    section.append(&view_group);
    
    // Navigation Group
    let nav_group = create_shortcut_group(
        "Navigation",
        &[
            ("Ctrl+G", "Go to Line"),
            ("Ctrl+Home", "Go to Beginning"),
            ("Ctrl+End", "Go to End"),
            ("Ctrl+Left", "Word Left"),
            ("Ctrl+Right", "Word Right"),
        ]
    );
    section.append(&nav_group);
    
    // Create scrolled window for this section
    let section_scroll = create_scrolled_section(&section);
    stack.add_titled(&section_scroll, Some("view"), "View");
}

/// Add advanced section
fn add_advanced_section(stack: &Stack) {
    let section = create_section_container();
    
    // Advanced Formatting Group
    let advanced_group = create_shortcut_group(
        "Advanced Formatting",
        &[
            ("Ctrl+Shift+U", "Underline"),
            ("Ctrl+Shift+E", "Center Text"),
            ("Ctrl+Shift+L", "Colored Text"),
            ("Ctrl+Shift+M", "Highlight"),
            ("Ctrl+Shift+Plus", "Superscript"),
            ("Ctrl+Shift+Minus", "Subscript"),
        ]
    );
    section.append(&advanced_group);
    
    // Help Group
    let help_group = create_shortcut_group(
        "Help",
        &[
            ("Ctrl+?", "Show Keyboard Shortcuts"),
            ("F1", "Show Markdown Guide"),
            ("Ctrl+Shift+A", "About Marco"),
        ]
    );
    section.append(&help_group);
    
    // Create scrolled window for this section
    let section_scroll = create_scrolled_section(&section);
    stack.add_titled(&section_scroll, Some("advanced"), "Advanced");
}

/// Create a section container with proper styling
fn create_section_container() -> Box {
    let section = Box::new(Orientation::Vertical, 24);
    section.set_margin_top(32);
    section.set_margin_bottom(32);
    section.set_margin_start(32);
    section.set_margin_end(32);
    section.add_css_class("shortcuts-section");
    section
}

/// Create a scrolled window containing the section
fn create_scrolled_section(section: &Box) -> ScrolledWindow {
    let scrolled = ScrolledWindow::new();
    scrolled.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);
    scrolled.set_vexpand(true);
    scrolled.add_css_class("shortcuts-section-scroll");
    scrolled.set_child(Some(section));
    scrolled
}

/// Create a group of shortcuts with a title
fn create_shortcut_group(title: &str, shortcuts: &[(&str, &str)]) -> Box {
    let group = Box::new(Orientation::Vertical, 12);
    group.add_css_class("shortcuts-group");
    
    // Group title
    let title_label = Label::new(Some(title));
    title_label.set_halign(Align::Start);
    title_label.add_css_class("shortcuts-group-title");
    group.append(&title_label);
    
    // Shortcuts grid
    let grid = Grid::new();
    grid.set_row_spacing(8);
    grid.set_column_spacing(32);
    grid.set_margin_start(16);
    grid.add_css_class("shortcuts-grid");
    
    for (row, (shortcut, description)) in shortcuts.iter().enumerate() {
        // Keyboard shortcut display
        let shortcut_box = create_shortcut_key_display(shortcut);
        grid.attach(&shortcut_box, 0, row as i32, 1, 1);
        
        // Description
        let desc_label = Label::new(Some(description));
        desc_label.set_halign(Align::Start);
        desc_label.add_css_class("shortcuts-description");
        grid.attach(&desc_label, 1, row as i32, 1, 1);
    }
    
    group.append(&grid);
    
    // Add separator after group (except for last group)
    let separator = Separator::new(Orientation::Horizontal);
    separator.add_css_class("shortcuts-separator");
    group.append(&separator);
    
    group
}

/// Create a keyboard shortcut key display with proper styling
fn create_shortcut_key_display(shortcut: &str) -> Box {
    let key_box = Box::new(Orientation::Horizontal, 4);
    key_box.set_halign(Align::Start);
    key_box.add_css_class("shortcuts-key-container");
    
    // Split shortcut into individual keys
    let keys: Vec<&str> = shortcut.split('+').collect();
    
    for (i, key) in keys.iter().enumerate() {
        if i > 0 {
            // Add "+" separator
            let plus_label = Label::new(Some("+"));
            plus_label.add_css_class("shortcuts-key-separator");
            key_box.append(&plus_label);
        }
        
        // Create key badge
        let key_label = Label::new(Some(key));
        key_label.add_css_class("shortcuts-key");
        key_box.append(&key_label);
    }
    
    key_box
}

/// Apply custom CSS styling to make the dialog look like GTK4 ShortcutsWindow
fn apply_shortcuts_dialog_css() {
    let css_provider = gtk4::CssProvider::new();
    css_provider.load_from_data(
        "
        /* Main shortcuts window styling */
        .shortcuts-window {
            background-color: @window_bg_color;
            border-radius: 12px;
        }
        
        /* Header styling */
        .shortcuts-header {
            background: linear-gradient(to bottom, @headerbar_bg_color, alpha(@headerbar_bg_color, 0.95));
            border-bottom: 1px solid @borders;
            padding: 12px 16px;
            border-radius: 12px 12px 0 0;
        }
        
        .shortcuts-title {
            font-size: 16px;
            font-weight: bold;
            color: @headerbar_fg_color;
        }
        
        .shortcuts-close {
            background: none;
            border: none;
            padding: 8px 12px;
            border-radius: 6px;
            color: @headerbar_fg_color;
            font-size: 14px;
            font-weight: bold;
        }
        
        /* Note: :hover and :active pseudo-classes removed to avoid GTK state accounting issues */
        
        /* Content area styling */
        .shortcuts-content {
            background-color: @window_bg_color;
        }
        
        /* Sidebar styling */
        .shortcuts-sidebar {
            background-color: @sidebar_bg_color;
            border-right: 1px solid @borders;
            padding: 0;
        }
        
        .shortcuts-sidebar row {
            padding: 12px 16px;
            border-radius: 0;
            margin: 0;
        }
        
        /* Note: :hover and :selected pseudo-classes removed to avoid GTK state accounting issues */
        
        .shortcuts-sidebar row label {
            font-size: 14px;
            font-weight: 500;
        }
        
        /* Section styling */
        .shortcuts-section {
            background-color: @view_bg_color;
        }
        
        .shortcuts-section-scroll {
            background-color: @view_bg_color;
        }
        
        /* Group styling */
        .shortcuts-group {
            margin-bottom: 24px;
        }
        
        .shortcuts-group-title {
            font-size: 18px;
            font-weight: bold;
            color: @view_fg_color;
            margin-bottom: 8px;
        }
        
        /* Grid styling */
        .shortcuts-grid {
            background-color: transparent;
        }
        
        /* Key display styling */
        .shortcuts-key-container {
            min-width: 140px;
        }
        
        .shortcuts-key {
            background: linear-gradient(to bottom, @card_bg_color, alpha(@card_bg_color, 0.9));
            border: 1px solid @borders;
            border-radius: 6px;
            padding: 4px 8px;
            font-family: monospace;
            font-size: 12px;
            font-weight: bold;
            color: @card_fg_color;
            box-shadow: 0 1px 3px alpha(@card_shade_color, 0.2);
            min-width: 24px;
        }
        
        .shortcuts-key-separator {
            color: alpha(@view_fg_color, 0.6);
            font-size: 12px;
            margin: 0 2px;
        }
        
        /* Description styling */
        .shortcuts-description {
            font-size: 14px;
            color: @view_fg_color;
        }
        
        /* Separator styling */
        .shortcuts-separator {
            background-color: alpha(@borders, 0.5);
            margin: 16px 0;
        }
        "
    );
    
    gtk4::style_context_add_provider_for_display(
        &gtk4::gdk::Display::default().unwrap(),
        &css_provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION + 1,
    );
}