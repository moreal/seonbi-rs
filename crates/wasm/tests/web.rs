#![cfg(target_arch = "wasm32")]

use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn smoke_test() {
    let config = wasm::ko_kr();
    let output = wasm::transform(config, "<p>\"漢字\"</p>").expect("transform should succeed");
    assert_eq!(output, "<p>&ldquo;한자&rdquo;</p>");
}

#[wasm_bindgen_test]
fn quote_and_content_type_matrix_for_ko_kr() {
    struct Case {
        quote: wasm::QuoteOption,
        content_type: &'static str,
        input: &'static str,
        expected: &'static str,
    }

    let cases = [
        Case {
            quote: wasm::QuoteOption::CurvedQuotes,
            content_type: "text/html",
            input: "<p>\"abc\"</p>",
            expected: "<p>&ldquo;abc&rdquo;</p>",
        },
        Case {
            quote: wasm::QuoteOption::CurvedQuotes,
            content_type: "application/xhtml+xml",
            input: "<p>\"abc\"</p>",
            expected: "<p>&ldquo;abc&rdquo;</p>",
        },
        Case {
            quote: wasm::QuoteOption::CurvedQuotes,
            content_type: "text/markdown",
            input: "\"abc\"",
            expected: "“abc”\n",
        },
        Case {
            quote: wasm::QuoteOption::CurvedQuotes,
            content_type: "text/plain",
            input: "\"abc\"",
            expected: "“abc”",
        },
        Case {
            quote: wasm::QuoteOption::Guillemets,
            content_type: "text/html",
            input: "<p>\"abc\"</p>",
            expected: "<p>&#x300a;abc&#x300b;</p>",
        },
        Case {
            quote: wasm::QuoteOption::Guillemets,
            content_type: "application/xhtml+xml",
            input: "<p>\"abc\"</p>",
            expected: "<p>&#x300a;abc&#x300b;</p>",
        },
        Case {
            quote: wasm::QuoteOption::Guillemets,
            content_type: "text/markdown",
            input: "\"abc\"",
            expected: "《abc》\n",
        },
        Case {
            quote: wasm::QuoteOption::Guillemets,
            content_type: "text/plain",
            input: "\"abc\"",
            expected: "《abc》",
        },
    ];

    for case in cases {
        let mut config = wasm::ko_kr();
        config.quote = Some(case.quote);
        config.content_type = Some(case.content_type.to_string());
        let output = wasm::transform(config, case.input).expect("matrix transform should succeed");
        assert_eq!(output, case.expected);
    }
}
