// Input validation utilities for dialog forms
// Common validation patterns and error handling

/// Validates that a string is not empty
pub fn validate_not_empty(text: &str) -> bool {
    !text.trim().is_empty()
}

/// Validates that a string is a valid HTML color
pub fn validate_html_color(color: &str) -> bool {
    if color.is_empty() {
        return false;
    }
    
    // Check if it's a valid hex color
    if color.starts_with('#') && color.len() == 7 {
        return color.chars().skip(1).all(|c| c.is_ascii_hexdigit());
    }
    
    // Check if it's a valid CSS color name (basic validation)
    matches!(color.to_lowercase().as_str(),
        "red" | "green" | "blue" | "yellow" | "orange" | "purple" | "pink" | 
        "brown" | "gray" | "grey" | "black" | "white" | "cyan" | "magenta" |
        "lime" | "navy" | "teal" | "silver" | "maroon" | "olive" | "aqua" |
        "fuchsia" | "darkred" | "darkgreen" | "darkblue" | "darkcyan" |
        "darkmagenta" | "darkyellow" | "darkgray" | "darkgrey" | "lightgray" |
        "lightgrey" | "lightred" | "lightgreen" | "lightblue" | "lightcyan" |
        "lightmagenta" | "lightyellow"
    )
}

/// Validates that a number is within a specified range
pub fn validate_number_range(value: f64, min: f64, max: f64) -> bool {
    value >= min && value <= max
}

/// Validates that a YouTube URL is valid
pub fn validate_youtube_url(url: &str) -> bool {
    if url.is_empty() {
        return false;
    }
    
    url.contains("youtube.com/watch?v=") || url.contains("youtu.be/")
}

/// Validates that a URL is valid (basic validation)
pub fn validate_url(url: &str) -> bool {
    if url.is_empty() {
        return false;
    }
    
    url.starts_with("http://") || url.starts_with("https://") || url.starts_with("ftp://")
}

/// Adds error styling to a widget
pub fn add_error_style(widget: &impl gtk4::prelude::WidgetExt) {
    widget.add_css_class("error");
}

/// Removes error styling from a widget
pub fn remove_error_style(widget: &impl gtk4::prelude::WidgetExt) {
    widget.remove_css_class("error");
}

/// Validates all required fields in a form
pub fn validate_form_fields<W: gtk4::prelude::WidgetExt>(validations: &[(bool, &W)]) -> bool {
    let mut all_valid = true;
    
    for (is_valid, widget) in validations {
        if *is_valid {
            remove_error_style(*widget);
        } else {
            add_error_style(*widget);
            all_valid = false;
        }
    }
    
    all_valid
}
