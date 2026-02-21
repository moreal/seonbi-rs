# seonbi

Core Rust library for Korean typographic adjustments and Hanja-to-Hangul
transformation.

## Example

```rust
use seonbi::{ko_kr, transform_html_text};

let input = "<p>平壤 冷麵...</p>";
let output = transform_html_text(&ko_kr(), input).unwrap();
println!("{output}");
```

## Compatibility Goal

For the same input and configuration, this crate aims to produce output
identical to the original Haskell `seonbi`.
