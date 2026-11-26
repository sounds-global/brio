# Brio

Brio is an open-source systems programming language for software where failure is not acceptable. It aims to run as fast as C and Rust, build much faster during development, and prove the safety properties that matter, with gradual migration from Rust and continued use of Rust crates.

> **Status: early design.** The language is fully specified in `docs/` (version 0.1, edition 2026). The toolchain is just starting, and nothing here is usable yet.

> **Safety notice.** The Brio toolchain is not qualified under any safety standard. It must not be the sole means of assurance for safety-critical software, and until it is qualified it marks everything it builds as *unqualified*.

## A taste of Brio

```rust
unit ml, h

distinct type PatientId = U64

record Infusion {
    patient: PatientId
    rate: F64<ml/h>
    invariant rate <= 1_200.0<ml/h>
}

/// The rate that delivers `volume` over `hours`, capped at `max`.
fn rate_for(volume: F64<ml>, hours: F64<h>, max: F64<ml/h>) -> F64<ml/h>
    requires hours > 0.0<h>
    ensures result <= max
{
    let rate = volume / hours        // ml ÷ h is ml/h; `volume + hours` would not compile
    return if rate > max { max } else { rate }
}

test "never exceeds the maximum" {
    assert rate_for(500.0<ml>, 1.0<h>, max: 200.0<ml/h>) == 200.0<ml/h>
}
```

Units of measure are part of the type, the compiler proves the `requires` and `ensures` contracts and record invariants before the program runs, and tests live next to the code.

## Why Brio

1. **Runtime speed equal to C and Rust.** No garbage collector, no runtime, no hidden allocations.
2. **Much faster development builds.** Edit to run in under a second, with a fast development back end and LLVM for release.
3. **Safety for life-critical systems**, through ownership, contracts, range types, units of measure and a provable `critical` profile.
4. **Gradual migration from Rust.** Brio calls Rust crates and Rust calls Brio, so a codebase can move one module at a time.
5. **Private-source libraries.** Consumers compile against interface files (`.brioh`), so a library's implementation source can stay private.
6. **Capabilities, not ambient authority.** Code reaches the network, files or other resources only through capabilities it is given, and can narrow them for a block.

## Brio Lens

The first tool being built is **Brio Lens**. Every compiler phase records what it did to each module and why, and every run can be recorded and replayed, so any result can be traced back to its source and explained. A phase cannot be released without its Lens records (specification, Clause 49).

## Documentation

- [Brio Language Specification](docs/Brio%20Language%20Specification.md): the normative definition of the language and toolchain.
- [Brio Language Design Spec](docs/Brio%20Language%20Design%20Spec.md): the design and the reasons behind it.
- [Brio Syntax Guide: Brio vs Rust vs TypeScript](docs/Brio%20Syntax%20Guide%20Brio%20vs%20Rust%20vs%20TypeScript.md): Brio side by side with the languages you already know.

## Roadmap

1. **Now:** the specification (version 0.1) and Brio Lens, the recorder every compiler phase writes to.
2. **Next:** the front end (lexer, error-recovering parser, name resolution and types), each phase with its Lens records.
3. **Then:** the ownership checker, the verifier, Brio Core and the interpreter, and the development and release back ends.

Progress is tracked in [GitHub Issues](https://github.com/sounds-global/brio/issues).

## Repository layout

| Path | Crate | What it is |
| --- | --- | --- |
| `cli/` | `brio_cli` | The `brio` command; the only binary in the workspace |
| `crates/lens/` | `brio_lens` | The Lens record envelope shared by every phase |
| `docs/` | | The specifications, decision records and contributor conventions |

The workspace is split into domain contexts, each with its own glossary and decision records. `GLOSSARY-MAP.md` lists them.

## Building

The toolchain is written in Rust 1.96 or later.

```sh
cargo build
cargo test
cargo clippy --workspace
```

The binary is `target/debug/brio`.

## Contributing

Contributions are welcome: bug reports, spec fixes, language proposals and code. Read [CONTRIBUTING.md](CONTRIBUTING.md) first. Every commit needs a [DCO](DCO) sign-off (`git commit -s`). See also [GOVERNANCE.md](GOVERNANCE.md) and the [Code of Conduct](CODE_OF_CONDUCT.md). Report security problems privately, as described in [SECURITY.md](SECURITY.md).

## License

- **Code** is licensed under the Apache License 2.0 with LLVM Exceptions (`Apache-2.0 WITH LLVM-exception`). See [LICENSE](LICENSE) and [NOTICE](NOTICE).
- **The language specification and documentation** in `docs/` are licensed under [Creative Commons Attribution 4.0 International](docs/LICENSE) (CC BY 4.0).

The "Brio" name and logo are not covered by these licenses.

Copyright 2026 Sounds Global B.V.
