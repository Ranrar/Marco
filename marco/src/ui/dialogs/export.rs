//! Export dialog and file chooser (GTK4, cross-platform).
//!
//! Provides a full export dialog (format selection, paper settings) followed
//! by a native file-save dialog.  Also exposes a standalone
//! `show_export_save_dialog` for callers that only need to pick a PDF path.

use gtk4::{
    glib, prelude::*, Adjustment, Align, Box as GtkBox, Button, CheckButton, DropDown, Entry,
    Expression, Label, Orientation, PropertyExpression, SpinButton, StringList, StringObject,
    Window,
};
#[cfg(not(target_os = "windows"))]
use gtk4::{FileChooserAction, FileChooserNative, FileFilter, ResponseType};
use std::{
    cell::RefCell,
    future::Future,
    path::PathBuf,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll, Waker},
};

// ─── Public types ─────────────────────────────────────────────────────────────

/// Which output format the user selected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Pdf,
    Html,
}

/// The complete set of choices collected from the export dialog.
#[derive(Debug, Clone)]
pub struct ExportSettings {
    pub format: ExportFormat,
    /// Document title embedded in HTML `<title>` / PDF job name.
    pub title: String,
    /// CSS theme filename (e.g. `"marco.css"`).
    pub theme: String,
    /// `"light"` or `"dark"`.
    pub theme_mode: String,
    /// Paper size string (`"A4"`, `"Letter"`, etc.).  For HTML without
    /// pagination this is `"None"`.
    pub paper: String,
    /// `"portrait"` or `"landscape"`.
    pub orientation: String,
    pub margin_mm: u8,
    pub show_page_numbers: bool,
    pub output_path: PathBuf,
}

// ─── Paper lists ──────────────────────────────────────────────────────────────

const PDF_PAPERS: &[&str] = &["A4", "A3", "A5", "Letter", "Legal", "B5"];
const HTML_PAPERS: &[&str] = &["None", "A4", "A3", "A5", "Letter", "Legal", "B5"];
const CONTROL_WIDTH_PX: i32 = 220;

fn make_paper_model(include_none: bool) -> StringList {
    if include_none {
        StringList::new(HTML_PAPERS)
    } else {
        StringList::new(PDF_PAPERS)
    }
}

fn paper_index(paper: &str, include_none: bool) -> u32 {
    let list = if include_none {
        HTML_PAPERS
    } else {
        PDF_PAPERS
    };
    list.iter()
        .position(|p| p.eq_ignore_ascii_case(paper))
        .unwrap_or(0) as u32
}

// ─── Full export dialog ───────────────────────────────────────────────────────

/// Show the full export dialog (format, paper, settings) and then a native
/// file-save dialog.
///
/// * `doc_stem`      - used as the suggested save filename stem.
/// * `doc_title`     - pre-populates the title entry (embedded in the output).
/// * `themes`        - `(display_label, filename)` pairs for the theme dropdown.
/// * `current_theme` - initially selected theme filename (e.g. `"marco.css"`).
/// * `current_mode`  - initially selected mode: `"light"` or `"dark"`.
///
/// Returns `None` if the user cancels at either step.
pub async fn show_export_dialog(
    parent: &gtk4::Window,
    doc_stem: &str,
    doc_title: &str,
    themes: &[(String, String)],
    current_theme: &str,
    current_mode: &str,
    layout: Option<&marco_shared::logic::swanson::LayoutSettings>,
) -> Option<ExportSettings> {
    // ── Theme detection ───────────────────────────────────────────────────────
    let parent_widget: gtk4::Widget = parent.clone().upcast();
    let initial_theme_class = if parent_widget.has_css_class("marco-theme-dark") {
        "marco-theme-dark".to_string()
    } else {
        "marco-theme-light".to_string()
    };
    let theme_class_state = Rc::new(RefCell::new(initial_theme_class));

    // ── Defaults from settings ────────────────────────────────────────────────
    let init_paper = layout
        .and_then(|l| l.page_view_paper.as_deref())
        .unwrap_or("A4")
        .to_string();
    let init_orient = layout
        .and_then(|l| l.page_view_orientation.as_deref())
        .unwrap_or("portrait")
        .to_string();
    let init_margin: u8 = layout.and_then(|l| l.page_view_margin_mm).unwrap_or(20);
    let init_page_numbers = layout
        .and_then(|l| l.page_view_show_page_numbers)
        .unwrap_or(true);

    // ── Theme data ────────────────────────────────────────────────────────────
    // Collect (label, filename) pairs; fall back to a built-in list if empty.
    let theme_pairs: Vec<(String, String)> = if themes.is_empty() {
        vec![
            ("Academic".to_string(), "academic.css".to_string()),
            ("Github".to_string(), "github.css".to_string()),
            ("Marco".to_string(), "marco.css".to_string()),
            ("Minimal".to_string(), "minimal.css".to_string()),
            ("Neutral".to_string(), "neutral.css".to_string()),
        ]
    } else {
        themes.to_vec()
    };
    let theme_labels: Vec<&str> = theme_pairs.iter().map(|(l, _)| l.as_str()).collect();
    let init_theme_idx: u32 = theme_pairs
        .iter()
        .position(|(_, f)| f.eq_ignore_ascii_case(current_theme))
        .unwrap_or(0) as u32;
    let init_mode_idx: u32 = if current_mode.eq_ignore_ascii_case("dark") {
        1
    } else {
        0
    };

    // ── Shared result state ───────────────────────────────────────────────────
    let result: Rc<RefCell<Option<ExportSettings>>> = Rc::new(RefCell::new(None));

    // ── Dialog window ─────────────────────────────────────────────────────────
    let dialog = Window::builder()
        .modal(true)
        .transient_for(parent)
        .default_width(460)
        .resizable(false)
        .build();
    dialog.add_css_class("marco-dialog");
    dialog.add_css_class(&theme_class_state.borrow());

    // ── Custom titlebar ───────────────────────────────────────────────────────
    let titlebar = crate::ui::titlebar::create_custom_titlebar_with_buttons(
        &dialog,
        "Export Document",
        crate::ui::titlebar::TitlebarButtons {
            close: true,
            minimize: false,
            maximize: false,
        },
    );
    let btn_close_titlebar = titlebar
        .close_button
        .as_ref()
        .expect("export dialog requires a close button");
    dialog.set_titlebar(Some(&titlebar.headerbar));

    // ── ESC key ───────────────────────────────────────────────────────────────
    let key_ctrl = gtk4::EventControllerKey::new();
    dialog.add_controller(key_ctrl.clone());

    // ── Main content area ─────────────────────────────────────────────────────
    let vbox = GtkBox::new(Orientation::Vertical, 10);
    vbox.add_css_class("marco-dialog-content");

    // Title row
    let title_entry = Entry::new();
    title_entry.add_css_class("marco-entry");
    title_entry.set_text(doc_title);
    title_entry.set_hexpand(true);
    title_entry.set_width_request(CONTROL_WIDTH_PX);
    vbox.append(&build_setting_row("Title:", &title_entry));

    // Format row
    let format_controls = GtkBox::new(Orientation::Horizontal, 12);
    format_controls.set_width_request(CONTROL_WIDTH_PX);
    format_controls.set_halign(Align::Start);

    let pdf_radio = CheckButton::with_label("PDF");
    pdf_radio.add_css_class("marco-radio");
    pdf_radio.set_active(true);
    let html_radio = CheckButton::with_label("HTML");
    html_radio.add_css_class("marco-radio");
    html_radio.set_group(Some(&pdf_radio));
    format_controls.append(&pdf_radio);
    format_controls.append(&html_radio);
    vbox.append(&build_setting_row("Format:", &format_controls));

    // Theme row
    let theme_expr =
        PropertyExpression::new(StringObject::static_type(), None::<&Expression>, "string");
    let theme_dropdown = DropDown::new(Some(StringList::new(&theme_labels)), Some(theme_expr));
    theme_dropdown.add_css_class("marco-dropdown");
    theme_dropdown.add_css_class(&theme_class_state.borrow());
    theme_dropdown.set_selected(init_theme_idx);
    theme_dropdown.set_hexpand(true);
    theme_dropdown.set_width_request(CONTROL_WIDTH_PX);
    vbox.append(&build_setting_row("Theme:", &theme_dropdown));

    // Mode row (Light / Dark)
    let mode_expr =
        PropertyExpression::new(StringObject::static_type(), None::<&Expression>, "string");
    let mode_dropdown = DropDown::new(Some(StringList::new(&["Light", "Dark"])), Some(mode_expr));
    mode_dropdown.add_css_class("marco-dropdown");
    mode_dropdown.add_css_class(&theme_class_state.borrow());
    mode_dropdown.set_selected(init_mode_idx);
    mode_dropdown.set_hexpand(true);
    mode_dropdown.set_width_request(CONTROL_WIDTH_PX);
    vbox.append(&build_setting_row("Mode:", &mode_dropdown));

    // Paper size row
    let paper_expr =
        PropertyExpression::new(StringObject::static_type(), None::<&Expression>, "string");
    let paper_dropdown = DropDown::new(Some(make_paper_model(false)), Some(paper_expr));
    paper_dropdown.add_css_class("marco-dropdown");
    paper_dropdown.add_css_class(&theme_class_state.borrow());
    paper_dropdown.set_selected(paper_index(&init_paper, false));
    paper_dropdown.set_hexpand(true);
    paper_dropdown.set_width_request(CONTROL_WIDTH_PX);
    vbox.append(&build_setting_row("Paper size:", &paper_dropdown));

    // Keep dialog and dropdown theme class in sync with the parent window so
    // light/dark CSS rules apply consistently, including popup content.
    {
        let dialog_weak = dialog.downgrade();
        let theme_class_state = theme_class_state.clone();
        let theme_dropdown = theme_dropdown.clone();
        let mode_dropdown = mode_dropdown.clone();
        let paper_dropdown = paper_dropdown.clone();
        parent_widget.connect_notify_local(Some("css-classes"), move |pw, _| {
            let next_theme = if pw.has_css_class("marco-theme-dark") {
                "marco-theme-dark"
            } else {
                "marco-theme-light"
            };

            {
                let mut state = theme_class_state.borrow_mut();
                if state.as_str() == next_theme {
                    return;
                }
                *state = next_theme.to_string();
            }

            let Some(dlg) = dialog_weak.upgrade() else {
                return;
            };
            dlg.remove_css_class("marco-theme-dark");
            dlg.remove_css_class("marco-theme-light");
            dlg.add_css_class(next_theme);

            for dd in [&theme_dropdown, &mode_dropdown, &paper_dropdown] {
                dd.remove_css_class("marco-theme-dark");
                dd.remove_css_class("marco-theme-light");
                dd.add_css_class(next_theme);
            }
        });
    }

    // Orientation row
    let orient_expr =
        PropertyExpression::new(StringObject::static_type(), None::<&Expression>, "string");
    let orient_dropdown = DropDown::new(
        Some(StringList::new(&["Portrait", "Landscape"])),
        Some(orient_expr),
    );
    orient_dropdown.add_css_class("marco-dropdown");
    orient_dropdown.add_css_class(&theme_class_state.borrow());
    orient_dropdown.set_selected(if init_orient == "landscape" { 1 } else { 0 });
    orient_dropdown.set_hexpand(true);
    orient_dropdown.set_width_request(CONTROL_WIDTH_PX);
    let orient_row = build_setting_row("Orientation:", &orient_dropdown);
    vbox.append(&orient_row);

    // Page margin row
    let margin_adj = Adjustment::new(init_margin as f64, 0.0, 60.0, 1.0, 5.0, 0.0);
    let margin_spin = SpinButton::new(Some(&margin_adj), 1.0, 0);
    margin_spin.add_css_class("marco-spinbutton");
    margin_spin.set_hexpand(true);
    margin_spin.set_width_request(CONTROL_WIDTH_PX);
    let margin_row = build_setting_row("Page margin:", &margin_spin);
    vbox.append(&margin_row);

    // Page numbers row
    let page_num_check = CheckButton::new();
    page_num_check.add_css_class("marco-checkbutton");
    page_num_check.set_active(init_page_numbers);
    page_num_check.set_halign(Align::Start);
    let page_num_holder = GtkBox::new(Orientation::Horizontal, 0);
    page_num_holder.set_width_request(CONTROL_WIDTH_PX);
    page_num_holder.set_halign(Align::Start);
    page_num_holder.append(&page_num_check);
    let pagenum_row = build_setting_row("Page numbers:", &page_num_holder);
    vbox.append(&pagenum_row);

    // ── Sensitivity logic ─────────────────────────────────────────────────────
    // Orientation, margin, page numbers are only meaningful when a paper size
    // is selected (i.e. not "None").  "None" is only available in HTML mode.
    let refresh_sensitivity = {
        let od = orient_dropdown.clone();
        let ms = margin_spin.clone();
        let pn = page_num_check.clone();
        move |is_pdf: bool, paper: &str| {
            let active = is_pdf || !paper.eq_ignore_ascii_case("none");
            od.set_sensitive(active);
            ms.set_sensitive(active);
            pn.set_sensitive(active);

            if active {
                od.remove_css_class("marco-control-unavailable");
                ms.remove_css_class("marco-control-unavailable");
                pn.remove_css_class("marco-control-unavailable");
                pn.set_tooltip_text(None);
            } else {
                od.add_css_class("marco-control-unavailable");
                ms.add_css_class("marco-control-unavailable");
                pn.add_css_class("marco-control-unavailable");
                pn.set_tooltip_text(Some("Locked when HTML paper size is set to None"));
            }
        }
    };

    // Paper change → re-evaluate
    {
        let pdf_radio_c = pdf_radio.clone();
        let rs = refresh_sensitivity.clone();
        paper_dropdown.connect_selected_notify(move |dd| {
            let paper = dd
                .selected_item()
                .and_then(|o| o.downcast::<StringObject>().ok())
                .map(|s| s.string().to_string())
                .unwrap_or_default();
            rs(pdf_radio_c.is_active(), &paper);
        });
    }

    // Format toggle → swap paper model, re-evaluate
    {
        let pd = paper_dropdown.clone();
        let rs = refresh_sensitivity.clone();
        // Remember last non-None paper across format switches.
        let last_paper: Rc<RefCell<String>> = Rc::new(RefCell::new(init_paper.clone()));
        let lp = last_paper.clone();
        pdf_radio.connect_toggled(move |radio| {
            let is_pdf = radio.is_active();

            // Save current paper if it's not "None".
            if let Some(current) = pd
                .selected_item()
                .and_then(|o| o.downcast::<StringObject>().ok())
                .map(|s| s.string().to_string())
            {
                if !current.eq_ignore_ascii_case("none") {
                    *lp.borrow_mut() = current;
                }
            }

            if is_pdf {
                let paper = {
                    let p = lp.borrow().clone();
                    if p.eq_ignore_ascii_case("none") {
                        "A4".to_string()
                    } else {
                        p
                    }
                };
                pd.set_model(Some(&make_paper_model(false)));
                pd.set_selected(paper_index(&paper, false));
                rs(true, &paper);
            } else {
                pd.set_model(Some(&make_paper_model(true)));
                // Default to "None" when switching to HTML.
                pd.set_selected(0);
                rs(false, "None");
            }
        });
    }

    // ── Button row ────────────────────────────────────────────────────────────
    let button_box = GtkBox::new(Orientation::Horizontal, 8);
    button_box.set_halign(Align::End);

    let btn_cancel = Button::with_label("Cancel");
    btn_cancel.add_css_class("marco-btn");
    btn_cancel.add_css_class("marco-btn-yellow");

    let btn_export = Button::with_label("Export\u{2026}");
    btn_export.add_css_class("marco-btn");
    btn_export.add_css_class("marco-btn-blue");

    button_box.append(&btn_cancel);
    button_box.append(&btn_export);

    let bottom_frame = gtk4::Frame::new(None);
    bottom_frame.add_css_class("marco-dialog-bottom-frame");
    bottom_frame.set_height_request(48);
    bottom_frame.set_vexpand(false);
    bottom_frame.set_margin_top(2);

    let bottom_inner = GtkBox::new(Orientation::Horizontal, 0);
    bottom_inner.set_margin_start(8);
    bottom_inner.set_margin_end(8);
    bottom_inner.set_margin_top(4);
    bottom_inner.set_margin_bottom(4);
    bottom_inner.set_halign(Align::Fill);
    bottom_inner.set_valign(Align::Center);

    let spacer = GtkBox::new(Orientation::Horizontal, 0);
    spacer.set_hexpand(true);
    bottom_inner.append(&spacer);
    bottom_inner.append(&button_box);
    bottom_frame.set_child(Some(&bottom_inner));

    let root = GtkBox::new(Orientation::Vertical, 0);
    root.append(&vbox);
    root.append(&bottom_frame);
    dialog.set_child(Some(&root));

    // ── Cancel / close wiring ─────────────────────────────────────────────────
    {
        let dw = dialog.downgrade();
        btn_cancel.connect_clicked(move |_| {
            if let Some(d) = dw.upgrade() {
                d.close();
            }
        });
    }
    {
        let dw = dialog.downgrade();
        btn_close_titlebar.connect_clicked(move |_| {
            if let Some(d) = dw.upgrade() {
                d.close();
            }
        });
    }
    {
        let dw = dialog.downgrade();
        key_ctrl.connect_key_pressed(move |_, key, _, _| {
            if key == gtk4::gdk::Key::Escape {
                if let Some(d) = dw.upgrade() {
                    d.close();
                }
                glib::Propagation::Stop
            } else {
                glib::Propagation::Proceed
            }
        });
    }

    // ── Export button: collect settings, open file chooser ────────────────────
    {
        let result_c = result.clone();
        let dw = dialog.downgrade();
        let pdf_radio_c = pdf_radio.clone();
        let paper_dd = paper_dropdown.clone();
        let orient_dd = orient_dropdown.clone();
        let margin_sp = margin_spin.clone();
        let pagenum_ch = page_num_check.clone();
        let title_ent = title_entry.clone();
        let theme_dd = theme_dropdown.clone();
        let mode_dd = mode_dropdown.clone();
        let theme_pairs_c = theme_pairs.clone();
        let doc_stem = doc_stem.to_string();

        btn_export.connect_clicked(move |_| {
            let format = if pdf_radio_c.is_active() {
                ExportFormat::Pdf
            } else {
                ExportFormat::Html
            };
            let paper = paper_dd
                .selected_item()
                .and_then(|o| o.downcast::<StringObject>().ok())
                .map(|s| s.string().to_string())
                .unwrap_or_else(|| "A4".to_string());
            let orientation = if orient_dd.selected() == 1 {
                "landscape".to_string()
            } else {
                "portrait".to_string()
            };
            let margin_mm = margin_sp.value() as u8;
            let show_page_numbers = pagenum_ch.is_active();
            let title = title_ent.text().to_string();
            let theme = theme_pairs_c
                .get(theme_dd.selected() as usize)
                .map(|(_, f)| f.clone())
                .unwrap_or_else(|| "marco.css".to_string());
            let theme_mode = if mode_dd.selected() == 1 {
                "dark".to_string()
            } else {
                "light".to_string()
            };

            let result_cc = result_c.clone();
            let dw_c = dw.clone();
            let stem = doc_stem.clone();

            glib::MainContext::default().spawn_local(async move {
                // Upgrade before opening the file chooser. Parent the file
                // chooser to the export dialog (not the main window) so it
                // appears on top of — not behind — the modal export dialog.
                let Some(dlg) = dw_c.upgrade() else { return };
                let ext = match format {
                    ExportFormat::Pdf => "pdf",
                    ExportFormat::Html => "html",
                };
                let suggested = format!("{}.{}", stem, ext);
                let path = show_save_dialog_for_format(&dlg, &suggested, format).await;

                if let Some(output_path) = path {
                    *result_cc.borrow_mut() = Some(ExportSettings {
                        format,
                        title,
                        theme,
                        theme_mode,
                        paper,
                        orientation,
                        margin_mm,
                        show_page_numbers,
                        output_path,
                    });
                    dlg.close();
                }
                // If file chooser was cancelled, keep the dialog open.
            });
        });
    }

    // ── Present and wait ──────────────────────────────────────────────────────
    dialog.present();

    struct DialogFuture {
        completed: Rc<RefCell<bool>>,
        waker: Rc<RefCell<Option<Waker>>>,
    }
    impl Future for DialogFuture {
        type Output = ();
        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
            if *self.completed.borrow() {
                Poll::Ready(())
            } else {
                *self.waker.borrow_mut() = Some(cx.waker().clone());
                Poll::Pending
            }
        }
    }

    let completed = Rc::new(RefCell::new(false));
    let waker: Rc<RefCell<Option<Waker>>> = Rc::new(RefCell::new(None));
    let completed_c = completed.clone();
    let waker_c = waker.clone();
    dialog.connect_close_request(move |_| {
        *completed_c.borrow_mut() = true;
        if let Some(w) = waker_c.borrow_mut().take() {
            w.wake();
        }
        glib::Propagation::Proceed
    });

    DialogFuture { completed, waker }.await;

    // Clone while both the Rc and the inner borrow are still in scope, then
    // return the owned value so the borrow does not outlive `result`.
    let choice = result.borrow().clone();
    choice
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Two-column settings row: fixed-width label on the left, widget on the right.
fn build_setting_row<W: IsA<gtk4::Widget>>(label_text: &str, widget: &W) -> GtkBox {
    let row = GtkBox::new(Orientation::Horizontal, 10);
    row.add_css_class("marco-dialog-row");
    row.set_halign(Align::Fill);
    let label = Label::new(Some(label_text));
    label.add_css_class("marco-dialog-option-title");
    label.set_halign(Align::Start);
    label.set_xalign(0.0);
    label.set_width_chars(14);
    row.append(&label);
    row.append(widget);
    row
}

/// Native file-save dialog filtered for the given export format.
async fn show_save_dialog_for_format(
    _parent: &gtk4::Window,
    suggested: &str,
    format: ExportFormat,
) -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        use rfd::AsyncFileDialog;

        let mut dialog = AsyncFileDialog::new().set_file_name(suggested);
        dialog = match format {
            ExportFormat::Pdf => dialog.add_filter("PDF Documents", &["pdf"]),
            ExportFormat::Html => dialog.add_filter("HTML Files", &["html", "htm"]),
        };

        let picked = dialog.save_file().await?;
        let mut p = picked.path().to_path_buf();
        let ext = match format {
            ExportFormat::Pdf => "pdf",
            ExportFormat::Html => "html",
        };
        if !p
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| e.eq_ignore_ascii_case(ext))
        {
            p.set_extension(ext);
        }
        return Some(p);
    }

    #[cfg(not(target_os = "windows"))]
    {
        let title = match format {
            ExportFormat::Pdf => "Save PDF As\u{2026}",
            ExportFormat::Html => "Save HTML As\u{2026}",
        };
        let native = FileChooserNative::new(
            Some(title),
            Some(_parent),
            FileChooserAction::Save,
            Some("Save"),
            Some("Cancel"),
        );
        native.set_current_name(suggested);

        let main_filter = FileFilter::new();
        match format {
            ExportFormat::Pdf => {
                main_filter.set_name(Some("PDF Documents (*.pdf)"));
                main_filter.add_pattern("*.pdf");
            }
            ExportFormat::Html => {
                main_filter.set_name(Some("HTML Files (*.html, *.htm)"));
                main_filter.add_pattern("*.html");
                main_filter.add_pattern("*.htm");
            }
        }
        native.add_filter(&main_filter);

        let all = FileFilter::new();
        all.set_name(Some("All Files"));
        all.add_pattern("*");
        native.add_filter(&all);

        if native.run_future().await == ResponseType::Accept {
            native.file().and_then(|f| f.path()).map(|mut p| {
                let ext = match format {
                    ExportFormat::Pdf => "pdf",
                    ExportFormat::Html => "html",
                };
                if p.extension().and_then(|e| e.to_str()) != Some(ext) {
                    let stem = p
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("document")
                        .to_owned();
                    p.set_file_name(format!("{}.{}", stem, ext));
                }
                p
            })
        } else {
            None
        }
    }
}

// ─── Standalone PDF save dialog (kept for callers that skip the full dialog) ──

/// Show a minimal "Save As PDF" file chooser without the settings dialog.
pub async fn show_export_save_dialog(
    parent: &gtk4::Window,
    suggested_name: Option<&str>,
) -> Option<PathBuf> {
    let suggested = format!("{}.pdf", suggested_name.unwrap_or("document"));
    show_save_dialog_for_format(parent, &suggested, ExportFormat::Pdf).await
}
