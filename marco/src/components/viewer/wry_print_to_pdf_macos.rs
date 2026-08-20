//! Silent PDF export backend for macOS / WKWebView.
//!
//! Windows uses `ICoreWebView2::PrintToPdf`, which is synchronous and pumps
//! the Win32 message queue while it runs. macOS has
//! `-[WKWebView createPDFWithConfiguration:completionHandler:]`, which is
//! asynchronous: the completion block is delivered through the main run loop,
//! so blocking the main thread while waiting for it would deadlock. Instead
//! we hand the write half of an mpsc channel to the completion block and let
//! the caller poll the receiver while the GTK main loop keeps pumping.

#![cfg(target_os = "macos")]

use std::path::Path;
use std::sync::mpsc;

use block2::RcBlock;
use objc2::rc::Retained;
use objc2_foundation::{NSData, NSError};
use objc2_web_kit::WKWebView;
use wry::WebViewExtMacOS;

/// Start an asynchronous WKWebView PDF capture of the page currently shown
/// in `view`.
///
/// Returns immediately with an mpsc `Receiver`. The caller must keep the GTK
/// main loop pumping (e.g. poll the receiver from a glib timeout) until the
/// receiver yields `Ok(())` or `Err(String)`. Dropping the receiver does not
/// cancel the capture; the completion block will simply fail to send.
pub fn start_print_to_pdf(
    view: &wry::WebView,
    output_path: &Path,
) -> Result<mpsc::Receiver<Result<(), String>>, String> {
    let (tx, rx) = mpsc::channel();
    let output_path = output_path.to_path_buf();

    let block = RcBlock::new(move |data_ptr: *mut NSData, err_ptr: *mut NSError| {
        // SAFETY: per ARC conventions the block receives a +1-retained object;
        // `Retained::retain` takes ownership of that reference. Null pointers
        // map to `None` so the error path never dereferences garbage.
        if !err_ptr.is_null() {
            let msg = match unsafe { Retained::retain(err_ptr) } {
                Some(err) => format!(
                    "WKWebView PDF capture failed: {}",
                    err.localizedDescription()
                ),
                None => "WKWebView PDF capture failed (unknown error)".to_string(),
            };
            let _ = tx.send(Err(msg));
            return;
        }

        let data = match unsafe { Retained::retain(data_ptr) } {
            Some(data) => data,
            None => {
                let _ = tx.send(Err("WKWebView PDF capture returned no data".to_string()));
                return;
            }
        };

        let result = std::fs::write(&output_path, data.to_vec()).map_err(|e| {
            format!(
                "Failed to write PDF to {}: {}",
                output_path.display(),
                e
            )
        });
        let _ = tx.send(result);
    });

    let webview: Retained<WKWebView> = view.webview().into_super();
    // WebKit retains the completion block until it fires, so the RcBlock can
    // be dropped as soon as the call returns.
    unsafe {
        webview.createPDFWithConfiguration_completionHandler(None, &block);
    }

    Ok(rx)
}
