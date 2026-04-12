use gtk4::prelude::*;
use gtk4::Box as GtkBox;
use log::debug;
use std::rc::Rc;

use super::helpers::{add_setting_row_i18n, SettingsI18nRegistry};
use crate::components::language::{SettingsLayoutTranslations, Translations};

pub struct PrintPreviewTabCallbacks {
    /// Called when any page view setting changes. Receives the full current state.
    pub on_page_view_changed: Option<
        std::boxed::Box<dyn Fn(crate::components::viewer::preview_types::PageViewState) + 'static>,
    >,
}

impl Default for PrintPreviewTabCallbacks {
    fn default() -> Self {
        Self {
            on_page_view_changed: None,
        }
    }
}

pub fn build_print_preview_tab(
    callbacks: PrintPreviewTabCallbacks,
    settings_path: Option<&str>,
    translations: &SettingsLayoutTranslations,
    i18n: &SettingsI18nRegistry,
) -> GtkBox {
    use gtk4::{
        Adjustment, Box as GtkBox, DropDown, Expression, Orientation, PropertyExpression,
        SpinButton, StringList, StringObject, Switch,
    };

    let container = GtkBox::new(Orientation::Vertical, 0);
    container.add_css_class("marco-settings-tab");

    let PrintPreviewTabCallbacks {
        on_page_view_changed,
    } = callbacks;

    // Initialize SettingsManager once if settings_path is available
    let settings_manager_opt = if let Some(path) = settings_path {
        match core::logic::swanson::SettingsManager::initialize(std::path::PathBuf::from(path)) {
            Ok(sm) => Some(sm),
            Err(e) => {
                debug!(
                    "Failed to initialize SettingsManager in print preview tab: {}",
                    e
                );
                None
            }
        }
    } else {
        None
    };

    // ── Shared state snapshot ───────────────────────────────────────────────
    use crate::components::viewer::preview_types::PageViewState;
    let pv_state: Rc<std::cell::RefCell<PageViewState>> = Rc::new(std::cell::RefCell::new({
        let layout = settings_manager_opt
            .as_ref()
            .map(|sm| sm.get_settings())
            .and_then(|s| s.layout.clone());
        PageViewState {
            enabled: layout
                .as_ref()
                .and_then(|l| l.page_view_enabled)
                .unwrap_or(false),
            paper: layout
                .as_ref()
                .and_then(|l| l.page_view_paper.clone())
                .unwrap_or_else(|| "A4".to_string()),
            orientation: layout
                .as_ref()
                .and_then(|l| l.page_view_orientation.clone())
                .unwrap_or_else(|| "portrait".to_string()),
            margin_mm: layout
                .as_ref()
                .and_then(|l| l.page_view_margin_mm)
                .unwrap_or(20),
            show_page_numbers: layout
                .as_ref()
                .and_then(|l| l.page_view_show_page_numbers)
                .unwrap_or(true),
            columns_per_row: layout
                .as_ref()
                .and_then(|l| l.page_view_columns)
                .unwrap_or(1)
                .clamp(1, 4),
        }
    }));

    let pv_callback: Rc<Option<std::boxed::Box<dyn Fn(PageViewState) + 'static>>> =
        Rc::new(on_page_view_changed);

    // ── Paper Size (DropDown) ───────────────────────────────────────────────
    let paper_values = ["A4", "Letter", "A3", "A5", "Legal", "B5"];
    let paper_options = StringList::new(&paper_values);
    let paper_expression =
        PropertyExpression::new(StringObject::static_type(), None::<&Expression>, "string");
    let paper_combo = DropDown::new(Some(paper_options), Some(paper_expression));
    paper_combo.add_css_class("marco-dropdown");

    let paper_index = paper_values
        .iter()
        .position(|v| v.eq_ignore_ascii_case(&pv_state.borrow().paper))
        .unwrap_or(0);
    paper_combo.set_selected(paper_index as u32);

    {
        let pv_state_c = Rc::clone(&pv_state);
        let pv_cb_c = Rc::clone(&pv_callback);
        let sm_c = settings_manager_opt.clone();
        paper_combo.connect_selected_notify(move |combo| {
            let paper = paper_values
                .get(combo.selected() as usize)
                .copied()
                .unwrap_or("A4");
            pv_state_c.borrow_mut().paper = paper.to_string();
            if let Some(ref sm) = sm_c {
                if let Err(e) = sm.update_settings(|s| {
                    s.layout
                        .get_or_insert_with(core::logic::swanson::LayoutSettings::default)
                        .page_view_paper = Some(paper.to_string());
                }) {
                    debug!("Failed to save page_view_paper: {}", e);
                }
            }
            if let Some(ref cb) = *pv_cb_c {
                cb(pv_state_c.borrow().clone());
            }
        });
    }

    let paper_row = add_setting_row_i18n(
        i18n,
        &translations.page_view_paper_label,
        &translations.page_view_paper_description,
        Rc::new(|t: &Translations| t.settings.layout.page_view_paper_label.clone()),
        Rc::new(|t: &Translations| t.settings.layout.page_view_paper_description.clone()),
        &paper_combo,
        true, // first row
    );
    container.append(&paper_row);

    // ── Orientation (DropDown) ──────────────────────────────────────────────
    let orient_labels = [
        translations.page_view_orientation_portrait.as_str(),
        translations.page_view_orientation_landscape.as_str(),
    ];
    let orient_values = ["portrait", "landscape"];
    let orient_options = StringList::new(&orient_labels);
    i18n.bind_string_list_item(
        &orient_options,
        0,
        Rc::new(|t: &Translations| t.settings.layout.page_view_orientation_portrait.clone()),
    );
    i18n.bind_string_list_item(
        &orient_options,
        1,
        Rc::new(|t: &Translations| t.settings.layout.page_view_orientation_landscape.clone()),
    );
    let orient_expression =
        PropertyExpression::new(StringObject::static_type(), None::<&Expression>, "string");
    let orient_combo = DropDown::new(Some(orient_options), Some(orient_expression));
    orient_combo.add_css_class("marco-dropdown");

    let orient_index = orient_values
        .iter()
        .position(|v| v.eq_ignore_ascii_case(&pv_state.borrow().orientation))
        .unwrap_or(0);
    orient_combo.set_selected(orient_index as u32);

    {
        let pv_state_c = Rc::clone(&pv_state);
        let pv_cb_c = Rc::clone(&pv_callback);
        let sm_c = settings_manager_opt.clone();
        orient_combo.connect_selected_notify(move |combo| {
            let orient = orient_values
                .get(combo.selected() as usize)
                .copied()
                .unwrap_or("portrait");
            pv_state_c.borrow_mut().orientation = orient.to_string();
            if let Some(ref sm) = sm_c {
                if let Err(e) = sm.update_settings(|s| {
                    s.layout
                        .get_or_insert_with(core::logic::swanson::LayoutSettings::default)
                        .page_view_orientation = Some(orient.to_string());
                }) {
                    debug!("Failed to save page_view_orientation: {}", e);
                }
            }
            if let Some(ref cb) = *pv_cb_c {
                cb(pv_state_c.borrow().clone());
            }
        });
    }

    let orient_row = add_setting_row_i18n(
        i18n,
        &translations.page_view_orientation_label,
        &translations.page_view_orientation_description,
        Rc::new(|t: &Translations| t.settings.layout.page_view_orientation_label.clone()),
        Rc::new(|t: &Translations| t.settings.layout.page_view_orientation_description.clone()),
        &orient_combo,
        false,
    );
    container.append(&orient_row);

    // ── Page Margin (SpinButton, 5-100 mm) ─────────────────────────────────
    let margin_adj = Adjustment::new(
        pv_state.borrow().margin_mm as f64,
        5.0,
        100.0,
        1.0,
        0.0,
        0.0,
    );
    let margin_spin = SpinButton::new(Some(&margin_adj), 1.0, 0);
    margin_spin.add_css_class("marco-spinbutton");

    {
        let pv_state_c = Rc::clone(&pv_state);
        let pv_cb_c = Rc::clone(&pv_callback);
        let sm_c = settings_manager_opt.clone();
        margin_adj.connect_value_changed(move |adj| {
            let mm = adj.value() as u8;
            pv_state_c.borrow_mut().margin_mm = mm;
            if let Some(ref sm) = sm_c {
                if let Err(e) = sm.update_settings(|s| {
                    s.layout
                        .get_or_insert_with(core::logic::swanson::LayoutSettings::default)
                        .page_view_margin_mm = Some(mm);
                }) {
                    debug!("Failed to save page_view_margin_mm: {}", e);
                }
            }
            if let Some(ref cb) = *pv_cb_c {
                cb(pv_state_c.borrow().clone());
            }
        });
    }

    let margin_row = add_setting_row_i18n(
        i18n,
        &translations.page_view_margin_label,
        &translations.page_view_margin_description,
        Rc::new(|t: &Translations| t.settings.layout.page_view_margin_label.clone()),
        Rc::new(|t: &Translations| t.settings.layout.page_view_margin_description.clone()),
        &margin_spin,
        false,
    );
    container.append(&margin_row);

    // ── Show Page Numbers (Toggle) ──────────────────────────────────────────
    let page_numbers_switch = Switch::new();
    page_numbers_switch.add_css_class("marco-switch");
    page_numbers_switch.set_active(pv_state.borrow().show_page_numbers);

    {
        let pv_state_c = Rc::clone(&pv_state);
        let pv_cb_c = Rc::clone(&pv_callback);
        let sm_c = settings_manager_opt.clone();
        page_numbers_switch.connect_state_set(move |_sw, active| {
            pv_state_c.borrow_mut().show_page_numbers = active;
            if let Some(ref sm) = sm_c {
                if let Err(e) = sm.update_settings(|s| {
                    s.layout
                        .get_or_insert_with(core::logic::swanson::LayoutSettings::default)
                        .page_view_show_page_numbers = Some(active);
                }) {
                    debug!("Failed to save page_view_show_page_numbers: {}", e);
                }
            }
            if let Some(ref cb) = *pv_cb_c {
                cb(pv_state_c.borrow().clone());
            }
            glib::Propagation::Proceed
        });
    }

    let page_numbers_row = add_setting_row_i18n(
        i18n,
        &translations.page_view_page_numbers_label,
        &translations.page_view_page_numbers_description,
        Rc::new(|t: &Translations| t.settings.layout.page_view_page_numbers_label.clone()),
        Rc::new(|t: &Translations| t.settings.layout.page_view_page_numbers_description.clone()),
        &page_numbers_switch,
        false,
    );
    container.append(&page_numbers_row);

    // ── Pages per Row (SpinButton, 1-4) ────────────────────────────────────
    let cols_adj = Adjustment::new(
        pv_state.borrow().columns_per_row as f64,
        1.0,
        4.0,
        1.0,
        0.0,
        0.0,
    );
    let cols_spin = SpinButton::new(Some(&cols_adj), 1.0, 0);
    cols_spin.add_css_class("marco-spinbutton");

    {
        let pv_state_c = Rc::clone(&pv_state);
        let pv_cb_c = Rc::clone(&pv_callback);
        let sm_c = settings_manager_opt.clone();
        cols_adj.connect_value_changed(move |adj| {
            let cols = adj.value().clamp(1.0, 4.0) as u8;
            pv_state_c.borrow_mut().columns_per_row = cols;
            if let Some(ref sm) = sm_c {
                if let Err(e) = sm.update_settings(|s| {
                    s.layout
                        .get_or_insert_with(core::logic::swanson::LayoutSettings::default)
                        .page_view_columns = Some(cols);
                }) {
                    debug!("Failed to save page_view_columns: {}", e);
                }
            }
            if let Some(ref cb) = *pv_cb_c {
                cb(pv_state_c.borrow().clone());
            }
        });
    }

    let cols_row = add_setting_row_i18n(
        i18n,
        "Pages per Row",
        "Number of pages to display side by side in page view (1-4).",
        Rc::new(|t: &Translations| t.settings.layout.page_view_page_numbers_label.clone()),
        Rc::new(|t: &Translations| t.settings.layout.page_view_page_numbers_description.clone()),
        &cols_spin,
        false,
    );
    container.append(&cols_row);

    container
}
