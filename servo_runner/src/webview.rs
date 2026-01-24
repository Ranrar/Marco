// MIT License - Marco Project
// GTK4 WebView widget powered by Servo

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use compositing_traits::rendering_context::RenderingContext;
use glib::{clone, Object};
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use image::RgbaImage;
use servo::{Servo, ServoBuilder};
use url::Url;

use crate::event_loop_waker::GtkEventLoopWaker;
use crate::rendering;

glib::wrapper! {
    pub struct WebView(ObjectSubclass<imp::WebView>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl WebView {
    pub fn new() -> Self {
        Object::builder().build()
    }

    pub fn load_url(&self, url: &str) {
        let imp = self.imp();

        log::info!("load_url() called with: {}", url);

        // Check if Servo is initialized
        if imp.servo_webview.borrow().is_none() {
            log::debug!("Servo not initialized yet, deferring URL load");
            let url_string = url.to_string();
            let widget_clone = self.clone();
            let tries = Cell::new(0u32);

            glib::timeout_add_local(Duration::from_millis(50), move || {
                let t = tries.get();
                if t >= 100 {
                    log::warn!("Giving up deferred URL load after {} retries", t);
                    return glib::ControlFlow::Break;
                }
                tries.set(t + 1);

                // Check if Servo is now initialized
                let imp = widget_clone.imp();
                if imp.servo_webview.borrow().is_none() {
                    return glib::ControlFlow::Continue;
                }

                // Servo is ready - actually load the URL
                log::debug!("Servo now initialized, loading URL");
                if let Ok(parsed_url) = Url::parse(&url_string) {
                    if let Some(webview) = imp.servo_webview.borrow().as_ref() {
                        log::info!("Loading URL in Servo WebView: {}", parsed_url);
                        webview.load(parsed_url);
                        imp.start_ticker_if_needed();
                    }
                }

                glib::ControlFlow::Break
            });
            return;
        }

        // Servo is already initialized - load immediately
        if let Ok(parsed_url) = Url::parse(url) {
            log::debug!("URL parsed successfully: {}", parsed_url);
            if let Some(webview) = imp.servo_webview.borrow().as_ref() {
                log::info!("Loading URL in Servo WebView: {}", parsed_url);
                webview.load(parsed_url);

                // Start ticker on first URL load (avoids race condition)
                log::debug!("Calling start_ticker_if_needed()");
                imp.start_ticker_if_needed();
                log::debug!("Ticker started (or already running)");
            } else {
                log::warn!("Servo WebView not initialized yet");
            }
        } else {
            log::error!("Invalid URL: {}", url);
        }
    }

    /// Cleanup method for compatibility with old API.
    /// The new Servo implementation handles cleanup automatically via Drop.
    pub fn cleanup(&self) {
        // Servo cleanup happens automatically when dropped
        log::info!("WebView cleanup called (handled automatically by Servo)");
    }
}

impl Default for WebView {
    fn default() -> Self {
        Self::new()
    }
}

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct WebView {
        pub servo: RefCell<Option<Servo>>,
        pub servo_webview: RefCell<Option<servo::WebView>>,
        pub rendering_context: RefCell<Option<Rc<dyn RenderingContext>>>,
        pub rendering_mode: Cell<Option<rendering::RenderingMode>>,
        pub cached_frame: Arc<Mutex<Option<RgbaImage>>>, // Shared pixel cache
        initialized: Cell<bool>,
        tick_callback: RefCell<Option<glib::SourceId>>,
        shutting_down: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for WebView {
        const NAME: &'static str = "ServoWebView";
        type Type = super::WebView;
        type ParentType = gtk::Widget;
    }

    impl ObjectImpl for WebView {
        fn constructed(&self) {
            self.parent_constructed();

            let widget = self.obj();

            log::info!("WebView widget constructed");

            // Initialize Servo when widget is realized
            widget.connect_realize(|w| {
                log::info!("WebView widget realized - initializing Servo");
                let imp = w.imp();
                imp.initialize_servo();
            });

            // Request redraw when widget becomes mapped (visible in window hierarchy)
            // This ensures the first frame is displayed when the widget is actually visible
            widget.connect_map(|w| {
                log::info!("WebView widget mapped - requesting initial redraw");
                w.queue_draw();
            });
        }

        fn dispose(&self) {
            // Stop the ticker to prevent further Servo calls
            self.shutting_down.set(true);

            if let Some(source_id) = self.tick_callback.take() {
                source_id.remove();
            }

            // WORKAROUND: Leak Servo to avoid crash on shutdown
            // The crash happens because Servo's background threads hold mutexes
            // and dropping Servo tries to destroy those mutexes while they're still in use
            if let Some(webview) = self.servo_webview.take() {
                std::mem::forget(webview);
            }
            if let Some(ctx) = self.rendering_context.take() {
                std::mem::forget(ctx);
            }
            if let Some(servo) = self.servo.take() {
                std::mem::forget(servo);
            }
        }
    }

    impl WidgetImpl for WebView {
        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            let widget = self.obj();
            let width = widget.width();
            let height = widget.height();

            log::debug!("snapshot() called: {}x{}", width, height);

            if width <= 0 || height <= 0 {
                log::debug!("Widget size invalid, skipping render");
                return;
            }

            // Try to get cached frame and render it
            if let Ok(frame_lock) = self.cached_frame.try_lock() {
                if let Some(ref image) = *frame_lock {
                    let (img_width, img_height) = image.dimensions();
                    log::debug!("Rendering cached frame: {}x{}", img_width, img_height);
                    let bytes = glib::Bytes::from(image.as_raw().as_slice());

                    let texture = gtk::gdk::MemoryTexture::new(
                        img_width as i32,
                        img_height as i32,
                        gtk::gdk::MemoryFormat::R8g8b8a8,
                        &bytes,
                        (img_width * 4) as usize, // stride: 4 bytes per pixel (RGBA)
                    );

                    use gtk::graphene;
                    // Scale texture to widget dimensions
                    let rect = graphene::Rect::new(0.0, 0.0, width as f32, height as f32);
                    snapshot.append_texture(&texture, &rect);
                    return;
                } else {
                    log::debug!("snapshot: No pixels cached yet (first render pending)");
                }
            } else {
                log::warn!("Could not lock cached_frame mutex");
            }

            // Fallback: Show color based on rendering mode while waiting for pixels
            match self.rendering_mode.get() {
                Some(rendering::RenderingMode::OpenGL) => {
                    use gtk::graphene;
                    let rect = graphene::Rect::new(0.0, 0.0, width as f32, height as f32);
                    let color = gtk::gdk::RGBA::new(0.0, 0.5, 1.0, 1.0); // Blue = OpenGL
                    snapshot.append_color(&color, &rect);
                }
                Some(rendering::RenderingMode::Software) => {
                    use gtk::graphene;
                    let rect = graphene::Rect::new(0.0, 0.0, width as f32, height as f32);
                    let color = gtk::gdk::RGBA::new(0.0, 0.7, 0.0, 1.0); // Green = Software (waiting for pixels)
                    snapshot.append_color(&color, &rect);
                }
                None => {
                    use gtk::graphene;
                    let rect = graphene::Rect::new(0.0, 0.0, width as f32, height as f32);
                    let color = gtk::gdk::RGBA::new(1.0, 0.0, 0.0, 1.0); // Red = Error
                    snapshot.append_color(&color, &rect);
                }
            }
        }

        fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
            // Call parent implementation first
            self.parent_size_allocate(width, height, baseline);

            // Resize Servo WebView when widget size changes
            if width > 0 && height > 0 {
                if let Some(webview) = self.servo_webview.borrow().as_ref() {
                    log::debug!(
                        "size_allocate: resizing Servo WebView to {}x{}",
                        width,
                        height
                    );
                    webview.resize(dpi::PhysicalSize::new(width as u32, height as u32));
                }
            }
        }
    }

    impl WebView {
        fn initialize_servo(&self) {
            if self.initialized.get() {
                return;
            }

            log::info!("Starting Servo initialization...");

            // Initialize crypto provider (required by Servo)
            match rustls::crypto::aws_lc_rs::default_provider().install_default() {
                Ok(_) => log::info!("Crypto provider installed"),
                Err(_) => log::info!("Crypto provider already installed"),
            }

            log::info!("Creating event loop waker");
            // Create event loop waker for GTK
            let event_loop_waker = Box::new(GtkEventLoopWaker::new());

            log::info!("Building Servo instance - this may take a moment");
            // Build Servo instance
            let servo = ServoBuilder::default()
                .event_loop_waker(event_loop_waker)
                .build();

            log::info!("Servo instance created successfully");

            // Note: Don't call servo.setup_logging() here - polo already initialized logging

            // Create rendering context (try OpenGL, fall back to software)
            let (rendering_context, rendering_mode) =
                match rendering::create_rendering_context(dpi::PhysicalSize {
                    width: 800,
                    height: 600,
                }) {
                    Ok((ctx, mode)) => (ctx, mode),
                    Err(e) => {
                        log::error!("Failed to create rendering context: {}", e);
                        return;
                    }
                };

            // Store rendering mode for later use in snapshot()
            self.rendering_mode.set(Some(rendering_mode));
            match rendering_mode {
                rendering::RenderingMode::OpenGL => log::info!("Using OpenGL hardware rendering"),
                rendering::RenderingMode::Software => log::info!("Using Software CPU rendering"),
            }

            // Make context current
            if let Err(e) = rendering_context.make_current() {
                log::error!("Failed to make rendering context current: {:?}", e);
                return;
            }

            // Create WebView with the rendering context
            log::info!("Creating Servo WebView");
            let widget = self.obj();
            let cached_frame_clone = self.cached_frame.clone();
            let delegate =
                WebViewDelegateImpl::new(&widget, rendering_context.clone(), cached_frame_clone);
            let webview = servo::WebViewBuilder::new(&servo, rendering_context.clone())
                .delegate(delegate)
                .build();

            log::info!("WebView created successfully");

            // Store Servo, WebView, and rendering context
            *self.servo.borrow_mut() = Some(servo);
            *self.servo_webview.borrow_mut() = Some(webview);
            *self.rendering_context.borrow_mut() = Some(rendering_context);

            self.initialized.set(true);
            log::info!("Servo initialization complete - ready for URL loading");
        }

        pub(crate) fn start_ticker_if_needed(&self) {
            // Only start ticker once
            if self.tick_callback.borrow().is_some() {
                return;
            }

            log::info!("Starting render ticker (60 FPS)");

            let widget = self.obj();
            let cached_frame = self.cached_frame.clone();
            let source_id = gtk::glib::timeout_add_local(
                std::time::Duration::from_millis(16), // ~60 FPS
                clone!(
                    #[weak]
                    widget,
                    #[strong]
                    cached_frame,
                    #[upgrade_or]
                    glib::ControlFlow::Break,
                    move || {
                        let imp = widget.imp();

                        // Don't run if shutting down
                        if imp.shutting_down.get() {
                            return glib::ControlFlow::Break;
                        }

                        if let Some(servo) = imp.servo.borrow().as_ref() {
                            servo.spin_event_loop();
                        }

                        // Paint and present the current frame
                        if let Some(webview) = imp.servo_webview.borrow().as_ref() {
                            webview.paint();

                            // CRITICAL: Present the rendered pixels to the screen
                            if let Some(ctx) = imp.rendering_context.borrow().as_ref() {
                                ctx.present();

                                // For software mode: Try to read pixels and cache them
                                if matches!(
                                    imp.rendering_mode.get(),
                                    Some(rendering::RenderingMode::Software)
                                ) {
                                    // Read pixels asynchronously (this is safe in the ticker thread)
                                    let width = widget.width();
                                    let height = widget.height();

                                    if width > 0 && height > 0 {
                                        use euclid::{Box2D, Size2D};
                                        let rect: Box2D<i32, servo::DevicePixel> =
                                            Box2D::from_size(Size2D::new(width, height));

                                        if let Some(image) = ctx.read_to_image(rect) {
                                            log::debug!(
                                                "Read pixels from software context: {}x{}",
                                                image.width(),
                                                image.height()
                                            );
                                            // Update cache (non-blocking try_lock)
                                            if let Ok(mut cache) = cached_frame.try_lock() {
                                                *cache = Some(image);
                                            }
                                        } else {
                                            log::warn!("read_to_image() returned None (software context doesn't support pixel reading?)");
                                        }
                                    }
                                }
                            }
                        }

                        // Request GTK to redraw the widget
                        widget.queue_draw();

                        glib::ControlFlow::Continue
                    }
                ),
            );

            // Store the ticker ID so we can stop it during disposal
            self.tick_callback.replace(Some(source_id));
            self.initialized.set(true);

            log::info!("Servo initialized successfully");
        }
    }

    /// Minimal WebView delegate implementation
    struct WebViewDelegateImpl {
        widget: glib::WeakRef<super::WebView>,
        rendering_context: Rc<dyn servo::RenderingContext>,
        cached_frame: Arc<Mutex<Option<image::RgbaImage>>>,
    }

    impl WebViewDelegateImpl {
        fn new(
            widget: &super::WebView,
            rendering_context: Rc<dyn servo::RenderingContext>,
            cached_frame: Arc<Mutex<Option<image::RgbaImage>>>,
        ) -> Rc<Self> {
            Rc::new(Self {
                widget: widget.downgrade(),
                rendering_context,
                cached_frame,
            })
        }
    }

    impl servo::WebViewDelegate for WebViewDelegateImpl {
        fn notify_new_frame_ready(&self, webview: servo::WebView) {
            // Paint Servo content to rendering context
            log::debug!("notify_new_frame_ready: calling paint()");
            webview.paint();

            // Present the rendered content (swap buffers for double-buffered contexts)
            log::debug!("notify_new_frame_ready: calling present()");
            self.rendering_context.present();

            // Read pixels from rendering context and cache them
            let size = self.rendering_context.size2d().to_i32();
            let viewport_rect =
                servo::DeviceIntRect::from_origin_and_size(euclid::Point2D::origin(), size);

            if let Some(rgba_image) = self.rendering_context.read_to_image(viewport_rect) {
                log::debug!(
                    "Read {}x{} image from rendering context",
                    rgba_image.width(),
                    rgba_image.height()
                );
                *self.cached_frame.lock().unwrap() = Some(rgba_image);
            } else {
                log::warn!("Failed to read image from rendering context");
            }

            // Request GTK redraw to display the cached frame
            if let Some(widget) = self.widget.upgrade() {
                log::debug!("Scheduling queue_draw via idle callback");
                // Use idle_add to ensure GTK is ready to process the draw request
                glib::idle_add_local_once(move || {
                    log::debug!("Executing queued queue_draw");
                    widget.queue_allocate(); // Force layout pass first
                    widget.queue_draw();
                });
            } else {
                log::warn!("Widget was dropped, cannot queue draw");
            }
        }
    }
}
