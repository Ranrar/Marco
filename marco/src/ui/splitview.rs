use gtk4::{prelude::*, CssProvider, EventControllerMotion, Label, Orientation, Overlay, Paned};
use std::cell::RefCell;
use std::rc::Rc;

/// Create a basic split view structure
pub fn create_split_view() -> Paned {
    let paned = Paned::new(Orientation::Horizontal);
    paned.set_position(400); // Initial position
    paned.set_resize_start_child(true);
    paned.set_resize_end_child(true);
    paned.set_shrink_start_child(false);
    paned.set_shrink_end_child(false);
    paned
}

/// Split percentage indicator that shows the current split percentage
/// while the user is dragging the paned separator
pub struct SplitPercentageIndicator {
    /// The overlay widget that contains the paned and the percentage label
    pub overlay: Overlay,
    /// The percentage label widget
    percentage_label: Label,
    /// Flag to track if the indicator is currently visible
    is_showing: Rc<RefCell<bool>>,
}

impl SplitPercentageIndicator {
    /// Create a new split percentage indicator that wraps the given paned widget
    pub fn new(paned: &Paned) -> Self {
        let overlay = Overlay::new();

        // Create the percentage label
        let percentage_label = Label::new(Some("50%"));
        percentage_label.set_halign(gtk4::Align::Center);
        percentage_label.set_valign(gtk4::Align::Center);
        percentage_label.set_visible(false);
        percentage_label.add_css_class("split-percentage-indicator");

        // Setup CSS styling
        let css_provider = CssProvider::new();
        css_provider.load_from_data(
            ".split-percentage-indicator {
                background-color: rgba(0, 0, 0, 0.7);
                color: white;
                padding: 10px 20px;
                border-radius: 8px;
                font-size: 16pt;
                font-weight: bold;
                margin: 10px;
            }",
        );

        // Apply CSS to the label
        let style_context = percentage_label.style_context();
        style_context.add_provider(&css_provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);

        // Set up the overlay structure
        overlay.set_child(Some(paned));
        overlay.add_overlay(&percentage_label);

        Self {
            overlay,
            percentage_label,
            is_showing: Rc::new(RefCell::new(false)),
        }
    }

    /// Show the percentage indicator with the current split percentage
    pub fn show(&self, percentage: i32) {
        self.percentage_label.set_text(&format!("{}%", percentage));
        self.percentage_label.set_visible(true);
        *self.is_showing.borrow_mut() = true;
    }

    /// Hide the percentage indicator
    pub fn hide(&self) {
        self.percentage_label.set_visible(false);
        *self.is_showing.borrow_mut() = false;
    }

    /// Update the percentage text (used during dragging)
    pub fn update_percentage(&self, percentage: i32) {
        if *self.is_showing.borrow() {
            self.percentage_label.set_text(&format!("{}%", percentage));
        }
    }

    /// Check if the indicator is currently showing
    pub fn is_showing(&self) -> bool {
        *self.is_showing.borrow()
    }

    /// Get a reference to the overlay widget (for adding to the UI)
    pub fn widget(&self) -> &Overlay {
        &self.overlay
    }
}

/// Set up drag detection and percentage indicator for a paned widget
/// Returns the split percentage indicator instance wrapped in an Rc for shared ownership
pub fn setup_split_percentage_indicator(paned: &Paned) -> Rc<SplitPercentageIndicator> {
    setup_split_percentage_indicator_with_cascade_prevention(paned, None)
}

/// Set up drag detection and percentage indicator for a paned widget with cascade prevention
/// Returns the split percentage indicator instance wrapped in an Rc for shared ownership
pub fn setup_split_percentage_indicator_with_cascade_prevention(
    paned: &Paned,
    position_being_set: Option<Rc<RefCell<bool>>>,
) -> Rc<SplitPercentageIndicator> {
    let indicator = Rc::new(SplitPercentageIndicator::new(paned));

    // Track drag state and position history
    let is_dragging = Rc::new(RefCell::new(false));
    let last_position = Rc::new(RefCell::new(-1i32));
    let last_change_time = Rc::new(RefCell::new(std::time::Instant::now()));

    // Create motion controller to detect when mouse leaves for cleanup
    let motion_controller = EventControllerMotion::new();

    // Mouse leaves the paned area - stop dragging immediately
    {
        let is_dragging_clone = Rc::clone(&is_dragging);
        let indicator_clone = Rc::clone(&indicator);

        motion_controller.connect_leave(move |_controller| {
            if *is_dragging_clone.borrow() {
                *is_dragging_clone.borrow_mut() = false;
                indicator_clone.hide();
            }
        });
    }

    // Add motion controller to the paned
    paned.add_controller(motion_controller);

    // Connect to position changes to detect and handle dragging
    {
        let indicator_clone = Rc::clone(&indicator);
        let is_dragging_clone = Rc::clone(&is_dragging);
        let last_position_clone = Rc::clone(&last_position);
        let last_change_time_clone = Rc::clone(&last_change_time);

        paned.connect_notify_local(Some("position"), move |paned, _| {
            // Prevent updates during programmatic position changes
            if let Some(ref flag) = position_being_set {
                if *flag.borrow() {
                    return;
                }
            }

            let width = paned.allocated_width();
            if width <= 0 {
                return;
            }

            let current_position = paned.position();
            let last_pos = *last_position_clone.borrow();

            // Check if position actually changed
            if last_pos == current_position {
                return;
            }

            *last_position_clone.borrow_mut() = current_position;

            let now = std::time::Instant::now();
            let time_since_last_change = now.duration_since(*last_change_time_clone.borrow());
            *last_change_time_clone.borrow_mut() = now;

            let percentage = ((current_position as f64 / width as f64) * 100.0).round() as i32;

            // If position changes are rapid (< 50ms apart), we're likely dragging
            let is_rapid_change = time_since_last_change.as_millis() < 50;

            if !*is_dragging_clone.borrow() && is_rapid_change {
                // Start dragging on rapid position changes
                *is_dragging_clone.borrow_mut() = true;
                indicator_clone.show(percentage);
            } else if *is_dragging_clone.borrow() && is_rapid_change {
                // Update percentage during rapid changes
                indicator_clone.update_percentage(percentage);
            } else if *is_dragging_clone.borrow() && !is_rapid_change {
                // Stop dragging on slow/single position changes
                *is_dragging_clone.borrow_mut() = false;
                indicator_clone.hide();
            }
        });
    }

    indicator
}
