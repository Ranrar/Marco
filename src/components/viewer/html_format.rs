pub fn pretty_print_html(input: &str) -> String {
    let with_newlines = input.replace(
        "><",
        ">
<",
    );
    let mut out = String::new();
    let mut indent: usize = 0;
    for raw_line in with_newlines.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with("</") {
            indent = indent.saturating_sub(1);
        }
        out.push_str(&"    ".repeat(indent));
        out.push_str(line);
        out.push('\n');
        let has_closing_after = line.find("</").is_some_and(|i| i > 0);
        if line.starts_with('<')
            && !line.starts_with("</")
            && !line.ends_with("/>")
            && !line.starts_with("<!")
            && !has_closing_after
        {
            indent += 1;
        }
    }
    out
}
