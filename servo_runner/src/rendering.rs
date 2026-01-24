// MIT License - Marco Project
// Rendering context creation with OpenGL + Software fallback

use compositing_traits::rendering_context::{RenderingContext, SoftwareRenderingContext};
use dpi::PhysicalSize;
use std::rc::Rc;

#[derive(Debug, Clone, Copy)]
pub enum RenderingMode {
    OpenGL,
    Software,
}

/// Create a rendering context, trying OpenGL first, then falling back to software
pub fn create_rendering_context(
    size: PhysicalSize<u32>,
) -> Result<(Rc<dyn RenderingContext>, RenderingMode), String> {
    // Try OpenGL first
    match try_create_opengl_context(size) {
        Ok(ctx) => {
            log::info!("Using OpenGL rendering");
            return Ok((ctx, RenderingMode::OpenGL));
        }
        Err(e) => {
            log::warn!("OpenGL rendering not available: {}", e);
            log::info!("Falling back to software rendering");
        }
    }

    // Fall back to software rendering
    match try_create_software_context(size) {
        Ok(ctx) => {
            log::info!("Using software rendering");
            Ok((ctx, RenderingMode::Software))
        }
        Err(e) => Err(format!("Failed to create rendering context: {}", e)),
    }
}

fn try_create_opengl_context(_size: PhysicalSize<u32>) -> Result<Rc<dyn RenderingContext>, String> {
    // TODO: Implement OpenGL context creation using surfman
    // For now, return error to use software fallback
    Err("OpenGL not yet implemented".to_string())
}

fn try_create_software_context(
    size: PhysicalSize<u32>,
) -> Result<Rc<dyn RenderingContext>, String> {
    SoftwareRenderingContext::new(size)
        .map(|ctx| Rc::new(ctx) as Rc<dyn RenderingContext>)
        .map_err(|e| format!("Software context failed: {:?}", e))
}
