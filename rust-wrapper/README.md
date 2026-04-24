# `sp800-90b-entropy-assessment`

Rust wrapper for the NIST SP 800-90B entropy assessment reference implementation.

This crate does not reimplement the entropy estimators in Rust. Instead, it builds the upstream NIST C++ command-line tools during `cargo build`, runs them with JSON output enabled, and deserializes the results into Rust types.

## What This Wrapper Provides

- Rust functions for the main NIST assessment flows:
  - `iid`
  - `non_iid`
  - `restart`
  - `conditioning`
- Typed option structs for each tool
- Structured `AssessmentReport` parsing
- Error handling for invalid arguments, process failures, and assessment failures

## Repository Layout

- `src/lib.rs`: public Rust API and JSON report handling
- `build.rs`: compiles the upstream C++ tools into Cargo build output
- `tests/`: correctness, security, and performance smoke tests
- `../vendor/SP800-90B_EntropyAssessment`: upstream NIST source tracked as a git submodule

## Requirements

The wrapper depends on the upstream NIST C++ code and its native dependencies.

Typical native requirements:

- `bz2`
- `divsufsort`
- `jsoncpp`
- `openssl` / `libcrypto`
- `gmp`
- `mpfr`
- OpenMP-capable C++ toolchain

On macOS with Homebrew, a typical setup is:

```bash
brew install gcc jsoncpp libdivsufsort openssl@3 gmp mpfr bzip2 libomp
```

## Build

From the wrapper directory:

```bash
cargo build
```

By default, `build.rs` reads the upstream C++ source from:

```text
../vendor/SP800-90B_EntropyAssessment/cpp
```

You can override that with:

```bash
SP80090B_CPP_DIR=/path/to/SP800-90B_EntropyAssessment/cpp cargo build
```

If headers or libraries are installed in nonstandard locations, you can also use:

```bash
SP80090B_CXXFLAGS="..." SP80090B_LDFLAGS="..." cargo build
```

## Example

```rust
use sp800_90b_entropy_assessment::{iid, CommonOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let report = iid(
        "../vendor/SP800-90B_EntropyAssessment/bin/normal.bin",
        &CommonOptions {
            bits_per_symbol: Some(8),
            quiet: true,
            ..CommonOptions::default()
        },
    )?;

    println!("error level: {}", report.error_level);
    println!("iid result: {:?}", report.iid);
    Ok(())
}
```

## Testing

Run the standard suite:

```bash
cargo test
```

Run the speed smoke tests:

```bash
cargo test --test performance -- --ignored --nocapture
```

## Notes

- The heavy work still happens in the upstream C++ implementation.
- Wrapper overhead is mostly process launch, temp-file IO, and JSON parsing.
- File-based assessments usually include `filename` and `sha256` in the parsed report.

More detailed project notes are in [`DOCUMENTATION.md`](./DOCUMENTATION.md).
