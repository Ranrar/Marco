//! Layout CSS Module
//!
//! Provides CSS styling for different layout modes, particularly the
//! document-style editor view in EditorOnly and EditorAndViewSeparate modes.

/// Generate CSS for layout modes
pub fn generate_css() -> String {
    r#"
/* ========================================
   Layout Mode Styling
   ======================================== */

/* Document mode for editor (EditorOnly and EditorAndViewSeparate) */
.editor-document-mode {
    background: #f5f5f5;
}

/* Document-style editor container with shadow */
.editor-document-mode .editor-container {
    background: #ffffff;
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.1), 
                0 2px 4px rgba(0, 0, 0, 0.08);
    margin: 24px;
    padding: 48px 72px;
    border-radius: 8px;
}

/* Dark theme document styling */
.marco-theme-dark .editor-document-mode {
    background: #1e1e1e;
}

.marco-theme-dark .editor-document-mode .editor-container {
    background: #2d2d2d;
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.3), 
                0 2px 4px rgba(0, 0, 0, 0.2);
}

/* Light theme document styling */
.marco-theme-light .editor-document-mode {
    background: #f5f5f5;
}

.marco-theme-light .editor-document-mode .editor-container {
    background: #ffffff;
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.1), 
                0 2px 4px rgba(0, 0, 0, 0.08);
}

/* ScrolledWindow in document mode - let it inherit */
.editor-document-mode scrolledwindow {
    background: transparent;
}

/* A4-like appearance for printed document feel */
.editor-document-mode .document-paper {
    background: white;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
    margin: 24px;
    padding: 96px 72px;
    min-height: 11.69in; /* A4 height */
}

.marco-theme-dark .editor-document-mode .document-paper {
    background: #353535;
}

"#.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_layout_css() {
        let css = generate_css();
        
        // Verify not empty
        assert!(!css.is_empty());
        
        // Verify document mode classes present
        assert!(css.contains(".editor-document-mode"));
        assert!(css.contains(".editor-container"));
        assert!(css.contains(".document-paper"));
        
        // Verify theme-specific rules
        assert!(css.contains(".marco-theme-dark"));
        assert!(css.contains(".marco-theme-light"));
        
        // Verify shadow styling present
        assert!(css.contains("box-shadow"));
    }
}
