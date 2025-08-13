The reason you only see a red border around the SourceView5 widget (and not the yellow background or blue text) is because **GTK SourceView5 does not allow the background color to be set via CSS**. The border and text color can be styled, but the background is controlled by the style scheme, which is not accessible via Rust bindings.

**Summary:**
- The CSS rule for `.sourceview { background: #ffeb3b; color: #1a237e; border: 3px solid red; ... }` only applies the border and text color.
- The background color is ignored by SourceView5 due to its internal style scheme handling.
- This is a known limitation in GTK4/SourceView5 and the Rust bindings.

**What you can do:**
- You cannot set the SourceView5 background color via CSS in Rust.
- You must wait for the Rust bindings to expose style scheme APIs, or use C/FFI.
- The red border is visible because borders are handled by the widget's style context, not the style scheme.

**Conclusion:**  
Your code is correct for what is currently possible. The red border is the only part of your CSS that SourceView5 will honor. Background color and font color are not supported via CSS for SourceView5 in Rust.

If you want to pursue a workaround, you would need to:
- Request or contribute style scheme support to the Rust bindings.
- Use a different widget for code editing if you need full CSS control.

Let me know if you want to file an upstream issue or need help with a workaround!