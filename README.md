# seonbi-rs

Rust port of `seonbi`.

## Language Bindings

This repository now includes multi-language bindings around `crates/seonbi`:

- `crates/seonbi-node`: Node.js / Deno / Bun binding via `napi-rs`
- `crates/seonbi-wasm`: browser/Web bundler binding via `wasm-bindgen`
- `crates/seonbi-python`: Python binding via `PyO3` + `maturin`

### Build Commands

- Node: `cd crates/seonbi-node && npm install && npx napi build --release --platform`
- WASM: `wasm-pack build crates/seonbi-wasm --release --target web --scope seonbi`
- Python: `maturin build --release --manifest-path crates/seonbi-python/Cargo.toml`

### Dictionary Embedding

`crates/seonbi` enables `freeze-dict` by default. This embeds `ko-kr-stdict.tsv`
into the binary and also guarantees WASM compatibility where filesystem access
is unavailable.

To build without embedding the default dictionary:

```bash
cargo build -p seonbi --no-default-features
```

## CLI Compatibility Note

The original `seonbi` 0.5.0 CLI exposes `-D` for both `--no-em-dash` and
`--dict`.

In this Rust port, `clap` cannot safely keep that duplicate short option, so
the mapping is:

- `-D, --dict FILE`
- `-M, --no-em-dash`

Long options are unchanged.

### Why This Diff Exists

- The original help output (`.tools/original/seonbi-0.5.0/seonbi -h`) lists
  both `-D,--no-em-dash` and `-D,--dict FILE`.
- With that duplication, short-option parsing can be context-sensitive and
  surprising.
- To keep parsing deterministic in `clap`, this port reserves `-D` for
  dictionary input and moves only `--no-em-dash` to `-M`.
