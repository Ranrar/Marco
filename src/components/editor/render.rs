use sourceview5::prelude::*;

pub fn render_editor_with_view(
    _scheme_id: &str,
    style_scheme: Option<&sourceview5::StyleScheme>,
    font_family: &str,
    font_size_pt: f64,
    show_line_numbers: bool,
) -> (
    gtk4::Box,
    sourceview5::Buffer,
    sourceview5::View,
    gtk4::CssProvider,
    gtk4::ScrolledWindow,
) {
    let container = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
    let buffer = sourceview5::Buffer::new(None);
    buffer.set_text("");
    let source_view = sourceview5::View::new();
    source_view.set_buffer(Some(&buffer));
    source_view.set_monospace(true);
    source_view.set_vexpand(true);
    source_view.set_editable(true);
    source_view.set_show_line_numbers(show_line_numbers);
    source_view.set_highlight_current_line(false);
    source_view.set_show_line_marks(false);

    // Set inner text margins (space between text and editor boundaries)
    source_view.set_left_margin(10); // Left margin inside the editor
    source_view.set_right_margin(10); // Right margin inside the editor
    source_view.set_top_margin(10); // Top margin inside the editor
    source_view.set_bottom_margin(10); // Bottom margin inside the editor

    if let Some(scheme) = style_scheme {
        buffer.set_style_scheme(Some(scheme));
    }

    use gtk4::CssProvider;
    let css = format!(
        ".sourceview {{ font-family: '{}', 'monospace'; font-size: {}pt; }}",
        font_family, font_size_pt
    );
    let font_provider = CssProvider::new();
    font_provider.connect_parsing_error(|_provider, section, error| {
        eprintln!(
            "[Theme] CSS parsing error in SourceView: {:?} at {:?}",
            error, section
        );
    });
    font_provider.load_from_data(&css);
    source_view
        .style_context()
        .add_provider(&font_provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);

    use sourceview5::BackgroundPatternType;
    source_view.set_background_pattern(BackgroundPatternType::None);

    let scrolled = gtk4::ScrolledWindow::new();
    scrolled.set_child(Some(&source_view));
    scrolled.set_vexpand(true);

    let scrolled_provider = gtk4::CssProvider::new();
    scrolled.add_css_class("editor-scrolled");
    if let Some(display) = gtk4::gdk::Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &scrolled_provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }

    container.append(&scrolled);

    (container, buffer, source_view, scrolled_provider, scrolled)
}
