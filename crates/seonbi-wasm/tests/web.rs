#![cfg(target_arch = "wasm32")]

use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn smoke_test() {
    let config = seonbi_wasm::ko_kr();
    let output =
        seonbi_wasm::transform(config, "<p>\"漢字\"</p>").expect("transform should succeed");
    assert!(!output.is_empty());
}
