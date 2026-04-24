# Wrapper Documentation

## Overview

The Rust wrapper is a process-level integration layer around the upstream NIST SP 800-90B C++ reference implementation.

The architecture is intentionally simple:

1. `build.rs` compiles the upstream C++ entry points into local helper binaries.
2. The Rust API constructs argument lists for those binaries.
3. Each binary writes a JSON report to a temporary file.
4. The wrapper deserializes that JSON into `AssessmentReport`.

This keeps the wrapper close to the behavior of the original implementation while giving Rust callers a typed interface.

## Upstream Dependency Model

The upstream NIST source is expected at:

```text
vendor/SP800-90B_EntropyAssessment
```

That directory is tracked as a git submodule in the parent repository. The wrapper should avoid modifying upstream C++ source files and instead keep wrapper-specific code inside `rust-wrapper/`.

## Public API

Main entry points in [`src/lib.rs`](./src/lib.rs):

- `iid(input, &CommonOptions)`
- `non_iid(input, &CommonOptions)`
- `restart(input, &RestartOptions)`
- `conditioning(ConditioningOptions)`

Important supporting types:

- `CommonOptions`
- `RestartOptions`
- `ConditioningOptions`
- `DatasetMode`
- `BitstringMode`
- `RestartMode`
- `ConditioningMode`
- `AssessmentReport`
- `Error`

## Error Model

The wrapper distinguishes between:

- `InvalidBitsPerSymbol`: invalid wrapper input rejected before launching a tool
- `MissingArgument`: required wrapper input missing
- `MissingBinary`: expected compiled helper binary is absent
- `Io`: process launch or temp-file IO failure
- `Json`: upstream tool ran but produced JSON the wrapper could not decode
- `CommandFailed`: upstream process returned a non-zero exit code
- `AssessmentFailed`: upstream tool returned a report with non-zero `errorLevel`

The wrapper also tolerates upstream error JSON where `testCases` is `null`, converting that into an empty vector rather than failing deserialization.

## Build Flow

[`build.rs`](./build.rs) compiles these upstream C++ entry points:

- `iid_main.cpp` -> `ea_iid`
- `non_iid_main.cpp` -> `ea_non_iid`
- `restart_main.cpp` -> `ea_restart`
- `conditioning_main.cpp` -> `ea_conditioning`

Produced binaries are placed under Cargo build output:

```text
$OUT_DIR/bin
```

The wrapper locates them at runtime using `env!("OUT_DIR")`.

### Environment Variables

- `SP80090B_CPP_DIR`: override upstream C++ source directory
- `SP80090B_CXXFLAGS`: append custom compiler flags
- `SP80090B_LDFLAGS`: append custom linker flags
- `CXX`: override the selected C++ compiler

## Test Suite

Integration tests live under [`tests/`](./tests).

### `tests/correctness.rs`

Validates that the wrapper successfully drives the upstream tools and returns expected high-level report properties for known sample inputs.

Coverage includes:

- `iid`
- `non_iid`
- conditioned dataset mode
- `restart`
- vetted `conditioning`

### `tests/security.rs`

Checks wrapper hardening and failure behavior.

Coverage includes:

- invalid argument rejection before process launch
- missing required conditioning input
- missing input-file failure handling
- shell-metacharacter-heavy filenames
- safe option constructor defaults

### `tests/performance.rs`

Ignored by default. These are speed smoke tests rather than strict benchmarks.

Run them with:

```bash
cargo test --test performance -- --ignored --nocapture
```

These tests are useful for regression detection but should not be treated as rigorous benchmarking without repeated controlled runs.

## Performance Notes

The wrapper still spends almost all of its time in the upstream C++ binaries. Extra wrapper overhead is limited to:

- process creation
- writing the JSON output path
- reading the generated JSON report
- JSON deserialization

For long-running assessments like `iid` and `non_iid`, wrapper overhead is usually small compared to the estimator runtime.

## Maintenance Guidance

- Keep wrapper code inside `rust-wrapper/`.
- Avoid changing files inside `vendor/SP800-90B_EntropyAssessment` unless intentionally updating or patching upstream.
- If upstream JSON output changes, update `AssessmentReport` and the tests together.
- Prefer adding integration tests when changing CLI argument construction or error handling.

## Suggested Workflow

Build and test locally:

```bash
cd rust-wrapper
cargo test
```

Run performance smoke tests:

```bash
cd rust-wrapper
cargo test --test performance -- --ignored --nocapture
```
