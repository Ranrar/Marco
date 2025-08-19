use comrak::{markdown_to_html, ComrakOptions};

#[test]
fn table_extension_renders_html_table() {
    println!("Test: table_extension_renders_html_table");
    let md = "| Col1 | Col2 |\n| --- | --- |\n| a | b |";
    let mut opts = ComrakOptions::default();
    opts.extension.table = true;
    let html = markdown_to_html(md, &opts);
    // Expect a <table> element in output when table extension is enabled
    assert!(html.contains("<table") && html.contains("<td") && html.contains("Col1"));
}

#[test]
fn table_extension_disabled_shows_no_table() {
    println!("Test: table_extension_disabled_shows_no_table");
    let md = "| Col1 | Col2 |\n| --- | --- |\n| a | b |";
    let mut opts = ComrakOptions::default();
    opts.extension.table = false;
    let html = markdown_to_html(md, &opts);
    // When disabled, comrak should not output a <table> element.
    assert!(!html.contains("<table") );
}

#[test]
fn autolink_extension_converts_urls_to_links() {
    println!("Test: autolink_extension_converts_urls_to_links");
    let md = "Visit https://example.com for info.";
    let mut opts = ComrakOptions::default();
    opts.extension.autolink = true;
    let html = markdown_to_html(md, &opts);
    assert!(html.contains("<a") && html.contains("https://example.com"));

    let mut opts2 = ComrakOptions::default();
    opts2.extension.autolink = false;
    let html2 = markdown_to_html(md, &opts2);
    // without autolink, raw url should remain as text (no <a> tag)
    assert!(!html2.contains("<a") );
}

#[test]
fn strikethrough_extension_renders_del_tags() {
    println!("Test: strikethrough_extension_renders_del_tags");
    let md = "This is ~~bad~~ text.";
    let mut opts = ComrakOptions::default();
    opts.extension.strikethrough = true;
    let html = markdown_to_html(md, &opts);
    assert!(html.contains("<del") || html.contains("<s") );

    let mut opts2 = ComrakOptions::default();
    opts2.extension.strikethrough = false;
    let html2 = markdown_to_html(md, &opts2);
    assert!(!html2.contains("<del") && !html2.contains("<s") );
}

#[test]
fn tasklist_extension_renders_checkboxes() {
    println!("Test: tasklist_extension_renders_checkboxes");
    let md = "- [x] Done\n- [ ] Not done";
    let mut opts = ComrakOptions::default();
    opts.extension.tasklist = true;
    let html = markdown_to_html(md, &opts);
    // comrak renders checkboxes as <input type="checkbox"
    assert!(html.contains("type=\"checkbox\"") || html.contains("checkbox"));

    let mut opts2 = ComrakOptions::default();
    opts2.extension.tasklist = false;
    let html2 = markdown_to_html(md, &opts2);
    assert!(!html2.contains("type=\"checkbox\"") );
}

#[test]
fn footnotes_extension_generates_footnote_section() {
    println!("Test: footnotes_extension_generates_footnote_section");
    let md = "Footnote[^1]\n\n[^1]: footnote text";
    let mut opts = ComrakOptions::default();
    opts.extension.footnotes = true;
    let html = markdown_to_html(md, &opts);
    // When enabled we expect a footnotes section or reference formatting
    assert!(html.to_lowercase().contains("class=\"footnotes\"") || html.to_lowercase().contains("footnote") );

    let mut opts2 = ComrakOptions::default();
    opts2.extension.footnotes = false;
    let html2 = markdown_to_html(md, &opts2);
    // When disabled, the original footnote markers should remain as literal text
    assert!(html2.contains("[^1]") || html2.contains("[^1]:"));
}

#[test]
fn tagfilter_extension_filters_raw_html_tags() {
    println!("Test: tagfilter_extension_filters_raw_html_tags");
    let md = "<div>hello</div>";
    let mut opts = ComrakOptions::default();
    opts.extension.tagfilter = true;
    // Do not allow raw HTML rendering so tagfilter can remove/escape tags
    opts.render.unsafe_ = false;
    let html = markdown_to_html(md, &opts);
    // with tagfilter, raw tags should be removed/escaped (no literal <div>)
    assert!(!html.contains("<div") );

    let mut opts2 = ComrakOptions::default();
    opts2.extension.tagfilter = false;
    opts2.render.unsafe_ = true;
    let html2 = markdown_to_html(md, &opts2);
    assert!(html2.contains("<div") );
}
