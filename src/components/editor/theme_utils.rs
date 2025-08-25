/// Crude helper to extract a color value from a minimal editor XML file. Looks
/// for: <color name="<key>" value="#RRGGBB"/> and returns the hex string.
pub fn extract_xml_color_value(contents: &str, key: &str) -> Option<String> {
    let needle = format!("name=\"{}\"", key);
    if let Some(pos) = contents.find(&needle) {
        if let Some(val_pos) = contents[pos..].find("value=\"") {
            let start = pos + val_pos + "value=\"".len();
            if let Some(end_rel) = contents[start..].find('"') {
                let val = contents[start..start + end_rel].trim().to_string();
                return Some(val);
            }
        }
    }
    None
}
