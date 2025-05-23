# Detect fast hardware support for PDEP/PEXT.

[<img alt="github" src="https://img.shields.io/badge/github-seancroach/has__fast__pdep-c9b9ff?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/seancroach/has_fast_pdep)
[<img alt="crates.io" src="https://img.shields.io/crates/v/has_fast_pdep.svg?style=for-the-badge&color=ffb488&logo=rust" height="20">](https://crates.io/crates/has_fast_pdep)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-has__fast__pdep-a6ebf4?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/has_fast_pdep)
[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/seancroach/has_fast_pdep/ci.yml?style=for-the-badge&logo=github&color=c6f0a0" height="20">](https://github.com/seancroach/has_fast_pdep/actions?query=branch%main)

A single-function, `no-std` library that returns `true` if the current CPU implements PDEP and PEXT with fast, non-microcoded hardware.

```toml
[dependencies]
has_fast_pdep = "0.1"
```

*Compiler support: requires rustc 1.85+*

## Rationale

Zen, Zen+, Zen 2, and Hygon Dhyana CPUs implement PDEP and PEXT using microcode in a way that makes them slower than well-optimized, non-intrinsic fallbacks. In performance-critical code, checking for BMI2 support isn't enough—you could end up hurting performance on said CPUs where these instructions exist but are slow. This crate helps you avoid that by detecting speed, not just support.

## Examples

Basic usage:

```rust
use has_fast_pdep::has_fast_pdep;

#[must_use]
pub fn exposed_fn(value: u64) -> u64 {
    if has_fast_pdep() {
        // SAFETY: The CPU has BMI2 and fast PDEP/PEXT instructions.
        unsafe { uses_pdep(value) }
    } else {
        fallback(value)
    }
}

#[must_use]
#[target_feature(enable = "bmi2")]
fn uses_pdep(value: u64) -> u64 {
    // TODO: implement PDEP/PEXT algorithm
    value
}

#[must_use]
fn fallback(value: u64) -> u64 {
    // TODO: implement fallback algorithm
    value
}
```

You can view the documentation on docs.rs [here](https://docs.rs/has_fast_pdep).

## Implementation Details

The result of the hardware check is determined once at runtime. After the initial check, all future
calls to `has_fast_pdep` becomes a simple `true` or `false` with zero branching or logic.

On x86 targets, CPUID is used directly without probing for its existence. This is intentional. For every tier 1 Rust target CPUID is guaranteed to be present. If you're targeting old hardware, such as an i486, this crate might not be for you. If you happen to be that individual, make an issue, and I'll reimplement the probing logic via inline assembly.

## License

Licensed under either of:

- Apache License, Version 2.0
  ([LICENSE-APACHE](https://github.com/seancroach/has_fast_pdep/blob/main/LICENSE-APACHE)
  or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license
  ([LICENSE-MIT](https://github.com/seancroach/has_fast_pdep/blob/main/LICENSE-MIT)
  or <http://opensource.org/licenses/MIT>)

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
