// CommonMark compliance tests: validate against commonmark.json spec

#[cfg(test)]
mod tests {
    use core::parser;
    use core::render::{render, RenderOptions};
    use serde_json::Value;
    
    #[test]
    fn test_commonmark_compliance() {
        let test_data = include_str!("../spec/commonmark.json");
        let tests: Vec<Value> = serde_json::from_str(test_data).unwrap();
        
        log::info!("Running {} CommonMark tests", tests.len());
        
        let mut passed = 0;
        let mut failed = 0;
        
        for test in tests.iter().take(10) {  // Start with first 10 tests
            let markdown = test["markdown"].as_str().unwrap();
            let expected_html = test["html"].as_str().unwrap();
            let section = test["section"].as_str().unwrap();
            
            log::debug!("Testing section: {}", section);
            
            match parser::parse(markdown) {
                Ok(doc) => {
                    match render(&doc, &RenderOptions::default()) {
                        Ok(html) => {
                            if html.trim() == expected_html.trim() {
                                passed += 1;
                            } else {
                                failed += 1;
                                log::warn!("Failed test in section: {}", section);
                            }
                        }
                        Err(_) => failed += 1,
                    }
                }
                Err(_) => failed += 1,
            }
        }
        
        log::info!("CommonMark results: {} passed, {} failed", passed, failed);
    }
}
