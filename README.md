# seonbi-rs

Rust port of `seonbi`.

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
