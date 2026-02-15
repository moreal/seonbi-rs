#![cfg(target_arch = "wasm32")]

use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn smoke_test() {
    let config = seonbi_wasm::ko_kr();
    let output =
        seonbi_wasm::transform(config, "<p>\"漢字\"</p>").expect("transform should succeed");
    assert_eq!(output, "<p>&ldquo;한자&rdquo;</p>");
}

#[wasm_bindgen_test]
fn transform_output_differs_by_content_type_for_ko_kr() {
    let html_output =
        seonbi_wasm::transform(seonbi_wasm::ko_kr(), "<p>\"abc\"</p>").expect("html transform");
    assert_eq!(html_output, "<p>&ldquo;abc&rdquo;</p>");

    let mut markdown_config = seonbi_wasm::ko_kr();
    markdown_config.content_type = Some("text/markdown".to_string());
    let markdown_output =
        seonbi_wasm::transform(markdown_config, "\"abc\"").expect("markdown transform");
    assert_eq!(markdown_output, "“abc”\n");
}
