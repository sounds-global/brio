# Brio Language Design Spec, v0.1

Sep 28, 2026 · Angelo Fornerino

## Overview and goals

Brio is a new systems language that can replace Rust in an existing codebase gradually, runs at least as fast as Rust with the same safety checks and faster where its proofs pay off, builds much faster during development, and is designed for systems where failure is not acceptable. Its syntax is its own, drawing on Rust, Ada, SPARK, Swift, Go and TypeScript. Version 0.1 incorporates an adversarial security and soundness review and a performance review, both from September 2026.

Five goals drive every decision in this spec:

1. **Runtime speed equal to C and Rust.** No garbage collector, no runtime, no hidden allocations.
2. **Much faster development builds.** Build time is the biggest pain point of large Rust codebases today.
3. **Safety strong enough for life-critical systems**, through ownership, contracts, range types, units of measure and a provable `critical` profile.
4. **Gradual migration from Rust**, with continued use of Rust crates (Tauri among them) and existing Rust macros.
5. **Private-source libraries.** Consumers compile against interface files; implementation source can stay private.

| Item | Value |
| --- | --- |
| Language name | Brio |
| Source files | `.brio` |
| Interface (header) files | `.brioh` |
| Command-line tool | `brio` |
| Searchable name | brio-lang |

Brio is meant for production use in well-tested, non-life-critical areas first. It must not be used for life-critical functions until the toolchain is qualified.

## &#32;Compiler architecture

Brio uses a hybrid model: its own compiler builds all Brio code, and only the thin boundary layer to Rust is emitted as Rust and built by rustc. Transpiling everything to Rust would inherit Rust's build times; a fully separate compiler with no Rust output could not use Rust generics or proc macros.

&#91;embedded content: Brio compiler architecture · hybrid model\]

The bridge generator reads a Rust crate's public API from rustdoc JSON and produces two things: `.brioh` interface files that Brio code compiles against, and a small Rust shim crate that rustc builds. Everything meets at the linker.

| Build | Backend | Generics | Purpose |
| --- | --- | --- | --- |
| Development | Cranelift | Dictionary passing (compiled once) | Edit-to-run in under a second |
| Release | LLVM | Monomorphized | Runtime speed equal to C and Rust |

### How Brio is implemented

Two different things are called "speed", and they are decided in different places:

| Speed of | Decided by | Brio's choice |
| --- | --- | --- |
| Programs Brio produces | The release code generator, and the facts Brio gives it | LLVM, fed with Brio's proven facts (see Runtime performance) |
| Development builds | The development code generator and the linker | TPDE on Linux x86-64 and AArch64, Cranelift everywhere else; fast incremental linkers |
| The compiler itself | Its architecture far more than its language | Rust, with a data-oriented, query-based, parallel design |

Writing the compiler in assembly would not make Brio programs one instruction faster: they are produced by LLVM either way. It would not make compiling faster either. When Zig rewrote its compiler from C++ into Zig, the new compiler was only 7% faster at building itself; the large gains came from its own back ends and its data layout. No production compiler today is written in assembly.

**Code generators.** Release builds use LLVM, the optimizer Rust and Clang use, at a pinned version. Development builds use one of two fast back ends. TPDE compiles LLVM's intermediate representation 10 to 20 times faster than LLVM's unoptimized mode with similar code quality, so dev and release builds share one lowering path; it currently supports only Linux on x86-64 and AArch64. Cranelift covers every other target, including macOS, Windows and embedded processors. The default for each target is chosen by measurement, and every dev back end is cross-checked against release builds and the interpreter; one that fails is switched off for that release. TPDE's code is slower than optimized code, so it is never used for release or critical builds.

**The compiler.** The compiler is written in Rust, which is memory-safe, already qualified for safety standards in the form of Ferrocene, and the language of Cranelift and of the fastest current linkers. Its design follows the fastest compilers of today: tokens, syntax and semantic data in flat arrays indexed by small integers; every piece of work a memoized query with early cutoff, shared by the command line, the editor and the assistant server; interfaces checked first so dependents start early; and bodies compiled in parallel. Budgets on a one-million-line corpus are enforced in CI: parsing at 200 ns per line or less, checking at 2 µs per line or less, completion within 50 ms.

**One compiler, with its trust rooted outside Brio.** The compiler stays in Rust. The danger a second compiler would guard against is a compiler binary that secretly inserts code into programs and into future builds of itself; that is only a risk when a compiler is used to build itself. Brio's compiler is built by Rust's toolchain, and Rust's compiler can be rebuilt from a full-source bootstrap, a few hundred bytes of machine code, through an independent Rust compiler, so Brio inherits that root of trust without maintaining a second compiler. Builds are deterministic, byte for byte. If the compiler is ever rewritten in Brio, the specification prescribes the next step: a small bootstrap compiler that only has to translate the compiler's own source, with no checking, optimizing, diagnostics or editor support, used to verify every release by diverse double-compiling.

**Where assembly is used.** Only where no language can avoid it: startup code, switching between tasks, interrupt and trap entry, stack probing, fallbacks for atomic operations on small processors, and single-instruction wrappers. Each piece is listed in the assurance ledger with its reviewer and tests. Cryptography uses formally verified assembly where hand-optimized code is needed.

**Libraries and proofs.** The core, alloc and std libraries are written in Brio. Hosted programs use a fast general allocator chosen by measurement; embedded and critical programs use fixed pools and an allocator with a bounded worst-case time. SMT solvers are treated as untrusted: their results are cached by content hash, limited by solver steps rather than time so results are deterministic, and accepted for critical code only when a small, independent proof checker confirms them.

### Brio Lens: seeing everything the language does

Brio Lens is the toolchain's built-in visualizer, and the first tool we build. From the first compiler phase onward, every phase writes down what it produced and why, in one linked format, and `brio lens` shows all of it side by side. It is how we develop the language, how we test it, and later how we teach it.

**One thread through the whole pipeline.** Every element at every stage carries a stable identifier and links back to the source text and to what it came from. Click a line of source and Lens highlights everything it became:

| Stage | What you see |
| --- | --- |
| Tokens and syntax | The tokens and the syntax tree, including how the parser recovered from errors |
| Names and types | What each name resolved to and which candidates were considered; each inferred type and the constraints that decided it |
| Ownership | Every borrow as a span over time; for a rejected borrow, both conflicting accesses and the rule |
| Contracts and proofs | Each proof obligation, what the solver was asked and answered; each safety check removed (with its proof) or kept (with the goal that failed) |
| Termination | The call graph, the measure that proves each function finishes, or the cycle that makes it unbounded |
| Capabilities | Where each capability came from and every place it flows |
| Compile-time code | Every compile-time evaluation and embedded-language expansion, with its origin |
| Brio Core and machine code | The core form, the intermediate code before and after each optimization pass of Cranelift, TPDE and LLVM, and the final machine code |

**Every decision has a reason.** Lens records not just what the compiler did but why, so questions that normally take an afternoon of guessing take a click:

- *Why was this borrow rejected?* Both accesses, highlighted, with the rule.
- *Why didn't this loop vectorize?* The pass that tried, and the check or alias it could not rule out.
- *Why did this edit rebuild 200 modules?* The query engine's record of what ran, what came from the cache, and where early cutoff stopped.
- *Why is this function unbounded?* The cycle, drawn.

**Running code, recorded.** The interpreter and the simulator record every step: values, memory with its aliasing tags, tasks and the order they ran in, capability uses and traps. A recording replays exactly and can be stepped backward. When the dev back end, the release back end and the interpreter disagree, Lens jumps to the first point where they differ, with the code of the functions involved.

**Built to test the language itself.** `brio lens test` compares each stage's output for test programs against reviewed reference records, so a change to the compiler shows exactly which stages changed and how. The conformance suite uses it to test each rule of the specification by its identifier. Tests, scripts and AI assistants can query the same records programmatically.

**Safe and free.** Recording never changes the build's output, which is tested, and costs nothing when off. Lens never shows decrypted licensed code to anyone not entitled to it, and it masks secret and sensitive values in execution records.

**From day one, by rule.** A compiler phase is not finished until it emits its Lens records, with provenance and reasons, in the same change that introduces it. That rule is what keeps Lens complete as the compiler grows, instead of becoming an afterthought nobody maintains.

The Brio compiler is also the language server. The editor, command line and CI run the same query-based engine, so they can never disagree about an error.

## Core syntax

Brio's syntax favours words over symbols, types after names, and contracts as part of every signature.

### Bindings

```brio
let speed: MetersPerSec = 12.5<m/s>   // immutable
var retries = 0                        // mutable
const MAX_DOSE: Mg = 500<mg>           // compile-time constant
```

### Functions and contracts

Contract clauses sit between the signature and the body. They are part of the signature, so they appear in `.brioh` interface files where library users see them.

```brio
fn dose_for(weight: Kg, rate: MgPerKg) -> Mg
    requires weight in 1<kg> ..= 300<kg>
    requires rate in 0.1<mg/kg> ..= 5.0<mg/kg>
    ensures  result <= MAX_DOSE
{
    return min(weight * rate, MAX_DOSE)
}
```

Each contract is handled in one of three modes, set per module or build profile:

| Mode | Meaning |
| --- | --- |
| `prove` | Proven statically with an SMT solver; zero runtime cost |
| `check` | Checked at runtime |
| `assume` | Trusted without checking; for boundaries such as Rust calls |

### Range types, units and distinct types

```brio
type Percent   = Int in 0 ..= 100
type HeartRate = U16 in 20 ..= 300
distinct type PatientId = U64          // no implicit conversion to or from U64

unit m, s, kg, g, ml
unit mg = g / 1000
unit h  = 3600 * s                     // derived units such as m/s need no declaration

type Altitude = F64<m> in -500.0<m> ..= 20_000.0<m>
```

Units are erased after type checking and cost nothing at runtime.

### Records, enums and match

```brio
record Reading {
    patient: PatientId
    rate:    HeartRate
    at:      Instant
}

enum Alarm {
    None
    Low(HeartRate)
    High(HeartRate)
    SensorFault(code: U16, since: Instant)
}

match alarm {
    .None                     => {}
    .Low(r) if r < 30         => page_team(.Urgent)
    .Low(_) | .High(_)        => page_team(.Normal)
    .SensorFault(code, ..)    => log.error("sensor fault {code}")
}
```

Matches must be exhaustive. The leading dot (`.None`, `.Urgent`) infers the enum type from context, as in Swift. Records are built with labels, `Reading(patient: p, rate: r, at: now)`, so a `{` after `if` or `while` always starts a block.

### Readability conveniences

Brio also has string interpolation (`"rate {r}"`), argument labels (`pump.set_rate(to: 5<ml/h>)`), optional chaining (`patient?.ward?.name`) and expression-bodied functions (`fn area(self) -> F64 = self.w * self.h`).

## Ownership and parameter modes

Brio keeps Rust's ownership and borrowing guarantees but expresses them as parameter modes, as in Ada and Swift, instead of reference types and lifetime annotations. Every mode maps one-to-one onto Rust, which is what makes automatic bridging and gradual migration possible.

```brio
fn checksum(buf: Buffer) -> U32              // default: shared borrow
fn normalize(buf: inout Buffer)              // exclusive borrow
fn archive(buf: take Buffer)                 // ownership moves in
fn init(buf: out Buffer)                     // must be written before return
fn header(src: Buffer) -> Header from src    // returned view borrows src
```

| Brio mode | Rust equivalent | Meaning |
| --- | --- | --- |
| (default) | `&T` | Shared, read-only borrow |
| `inout` | `&mut T` | Exclusive, mutable borrow |
| `take` | `T` | Ownership transferred to the callee |
| `out` | `&mut MaybeUninit<T>` | Callee must initialise before returning |
| `from a` | lifetime `'a` | Returned value borrows from `a` |

A mode describes intent, not mechanism: it says what the callee may do with the argument, and the compiler chooses the fastest safe way to pass it. Small values, such as a `U32` or, where the target's calling convention allows it, a `Vec3`, travel in registers even in the default mode, and an `inout` value may be copied in and back instead of passed by address, because no other code can see it during the call. The compiler always passes by address when it matters: when the result borrows from the parameter (`from`), when the type has interior mutability such as a `Cell` or `Mutex`, when the type's size is not known at the call, and at foreign boundaries that require it. The choice never changes what a program does; the only trace is a parameter's address, which is unspecified and which safe code cannot rely on.

### Call-site markers

Mutation and moves are visible where the call happens, so reviewers never have to open a function to know what it does to its arguments:

```brio
let sum = checksum(buf)
normalize(mut buf)
archive(move buf)
```

### Origins instead of lifetimes

`from` replaces Rust's `'a` lifetime syntax and is inferred wherever the answer is unambiguous. When a result may borrow from several inputs, they are joined with `|`:

```brio
fn longest(a: Str, b: Str) -> Str from a | b
```

### Value semantics in the critical profile

Inside `@profile(critical)` modules, borrowed references cannot be stored in record fields; they exist only as parameters. This is the mutable value semantics model from Hylo. It makes lifetimes almost disappear and removes a whole class of aliasing reasoning from safety reviews.

## Error handling

Errors are ordinary values: `T ! E` in a signature compiles to the same representation as Rust's `Result<T, E>`, with no exceptions and no unwinding tables.

A prefix `try` propagates an error to the caller. It is easier to spot in code review than Rust's trailing `?`:

```brio
fn read_sensor(port: inout Port) -> Reading ! SensorError {
    let raw = try port.read_frame()
    let reading = try Reading.decode(raw)
    return reading
}
```

For local recovery, a Swift-style `else` block handles the error in place and must leave the scope:

```brio
let frame = port.read_frame() else |e| {
    log.warn("retrying after {e}")
    return .retry
}
```

Brio has no panics. A violated contract or failed runtime check is a *trap*: the trap handler brings the system to a safe state, then the process stops without unwinding. In the `critical` profile, every condition that could trap is proven impossible at compile time.

## Generics and traits

Generics use square brackets, which removes Rust's turbofish and its parsing ambiguities:

```brio
fn max[T: Ord](a: T, b: T) -> T
let m = max[I32](a, b)
```

Traits and their implementations:

```brio
trait Sensor {
    type Output
    fn sample(inout self) -> Self.Output ! SensorError
}

impl Sensor for PulseOx { ... }
```

A trait's name used as a type means *one kind, fixed at compile time*: the compiler generates code specialized for the exact type, so calls are direct and can be inlined. `any` means *the kind may vary at runtime*: each value carries its method table, and calls are dispatched at runtime. The fast option is the one without a keyword, and the cost of flexibility is always visible:

```brio
fn poll(s: Sensor)                 // one sensor, kind fixed at compile time
fn poll_all(s: [Sensor])           // many sensors, all the same kind
fn log_all(s: [any Sensor])        // many sensors, kinds may differ
fn open_default() -> Sensor        // one concrete kind, hidden from the caller

fn compare[S: Sensor](a: S, b: S)  // name the type to require the same kind twice
```

Each `Sensor` in a parameter list is its own fixed kind, so `fn f(a: Sensor, b: Sensor)` accepts two different kinds; naming the type, as in `compare`, requires them to match. Record fields cannot use a bare trait name, because a hidden type parameter there would surprise readers: a field is written `sensor: S` with a named type parameter, or `sensor: any Sensor`.

`any Sensor` is an ordinary sized value, so it works in lists, fields and return types without a `Box`. Small implementations are stored inline; larger ones are placed on the heap when converted, which the compiler knows at the conversion. Such conversions are errors in `@no_alloc` code, are listed by `brio perf`, and never happen when a value is only borrowed, as when it is passed to a parameter of type `any Sensor`. `critical` code uses only the static form.

### How generics compile

| Build | Strategy | Why |
| --- | --- | --- |
| Development | Dictionary passing (witness tables); each generic function compiles once | Monomorphization bloat is a major cause of slow Rust builds |
| Release | Full monomorphization, as in Rust | Lets LLVM inline, vectorize and specialize |

Where a library author wants fast versions of a generic function without exposing its body, `@specialize` precompiles them:

```brio
@specialize(F32, F64, I32)
public fn dot[T: Num](a: [T], b: [T]) -> T
```

The compiler warns whenever a release build falls back to dictionary dispatch, so this cost never goes unnoticed.

## Modules, interfaces and private source

Developers write one `.brio` file per module with `public` markers, and nothing else: a source tree contains only `.brio` files, `Brio.toml` and `Brio.lock`. The compiler generates each module's `.brioh` interface file into the `.brio/` build folder. API changes are still guarded: `Brio.lock` records a hash of every public item, so a build fails when the public API changes unless the change is accepted with `--accept-interface-change`, and the diagnostic lists every added, removed and changed item, with a textual difference when the previous version is available from version control or the registry.

A generated interface looks like this:

```brio
interface pump.controller

use units.{Ml, MlPerHour}

public opaque record Controller               // layout hidden
public frozen record Config {                 // layout exported
    public max_rate: MlPerHour
}
public enum PumpError { Occlusion, AirInLine, DoorOpen }

public fn Controller.new(cfg: Config) -> Controller
public fn Controller.set_rate(inout self, rate: MlPerHour) -> () ! PumpError
    requires rate <= self.max_rate()
    ensures  self.rate() == rate

@inlinable
public fn Controller.is_running(self) -> Bool = self.rate() > 0.0<ml/h>
```

### Why interfaces speed up builds

Dependents recompile only when an interface file's hash changes, as in Go and Ada. Editing a function body rebuilds one module, not everything downstream of it.

### The build folder

Everything the compiler generates lives in `.brio/` at the workspace root: interfaces, compiled code, caches, the assurance ledger and reports. `brio new` adds it to `.gitignore`, and it can be deleted at any time: the next build recreates it, bit for bit.

This design is both sound and fast:

- **Nothing can go stale.** Every cache entry is keyed by a hash of all its inputs (the module's source, the interfaces it depends on, the toolchain, the target and the settings) and is verified before use. There are no timestamps to go wrong, and a corrupted entry is simply regenerated.
- **Interfaces first.** Because every signature is written out, the compiler produces a module's interface before checking any function body. Modules that depend on it start compiling at once, in parallel with the dependency's bodies.
- **Early cutoff.** Changing a function body leaves the interface hash unchanged, so nothing that depends on the module is recompiled or re-verified.
- **Shared caches.** Teams and CI can share a remote cache, but only trusted builders, such as CI on protected branches, may write to it; everyone else reads. Keys cover an enumerated list of every input, including the toolchain itself, and every entry is signed and verified, so a poisoned or mismatched entry is impossible and a build using the cache is identical to one that does not.
- **One source of truth for editors.** The language server uses the same interfaces and cache, which is why completion stays fast in large workspaces.

Dependencies' interfaces, including those of private-source packages, are unpacked into the build folder too. Schema history for `@schema` types is written in the source itself, through `@removed`, `@renamed` and `@migrate`, and is guarded by hashes in `Brio.lock`, so no schema files are needed either.

### Private-source packages

A package ships `.brioh` files plus compiled code per target. Source is optional:

```toml
[package]
name   = "infusion-core"
source = "private"    # "public" | "private" | "escrow"
```

| Mechanism | What is exposed | Speed |
| --- | --- | --- |
| Plain `public fn` | Signature and contracts only | Normal call |
| `@inlinable` / `inline` | Body shipped in the interface | Fully optimizable |
| `@specialize(...)` | Precompiled versions for listed types | Full speed for those types |
| Generic, no annotation | Signature only | Dictionary dispatch across the boundary |
| LLVM bitcode shipping | Optimizable IR, not source | Full cross-library optimization |

Types follow the same trade-off. Within a package every type is laid out directly, inline and on the stack. Only types exported from a binary-only library are opaque, and marking them `frozen` exports their layout and removes the indirection.

Bitcode is much harder to read than source but is not encryption; it suits clients who want convenience rather than strong secrecy.

### Licensed packages

A publisher can make a package readable only by its licensees. Encryption keeps it confidential from everyone else, including registries, mirrors and anyone intercepting downloads, and guarantees its integrity to all. It cannot hide the code from a licensee whose toolchain decrypts it, and Brio says so plainly instead of relying on obfuscation; contracts cover the rest.

```toml
[distribution]
interfaces = "public"      # anyone can browse the API and audit the package
code       = "licensed"    # compiled code for licensees only
bitcode    = "licensed"
# source follows the package's `source` key: here "escrow"
```

**How it works.**

1. **Encrypt once, unlock per licensee.** Each non-public layer of a version is encrypted once with a fresh key (AES-256-GCM or XChaCha20-Poly1305) in a key-committing construction, so a file decrypts under exactly one key. That key is wrapped for each licensee's public key with HPKE (RFC 9180), optionally with a hybrid post-quantum suite (X25519 with ML-KEM-768) for code that must stay confidential for decades. Adding a licensee only wraps one more small key.
2. **Bound to its context.** Every ciphertext and wrapped key is bound to its package, version, layer and target, so nothing can be moved to unlock something else.
3. **Signed and verified twice.** The publisher signs the hashes of every layer, encrypted and decrypted. Every wrapped key is signed too. The toolchain checks all of it before and after decrypting, before any tool, even the editor, sees the code, and uses nothing that fails.
4. **Registries hold nothing secret.** They store only ciphertext, wrapped keys and public metadata (name, version, dependencies, declared permissions, advisories), so resolution, `brio audit` and permission policies work without any key, and a breached registry leaks nothing.
5. **Keys stay out of the repository.** Licensee keys live in the OS key store, a hardware key, a key-management service or a CI secret, managed with `brio license`.
6. **Offline license tokens.** A signed token states the licensee, packages, versions, layers, targets and expiry, and is checked offline, so air-gapped builds work. A missing or expired license is a clear build error.

**Guarantees that matter.**

- **No kill switch.** Licenses are checked only when building. A program already shipped, such as a device in a hospital, never stops working because a license expired.
- **No leaks through caches.** Decrypted layers are cached per machine and readable only by the building user. Anything built from licensed code is shared through a remote cache only with holders of the same license.
- **Fast.** Decryption runs at gigabytes per second, and each layer is decrypted once per machine, then cached by content hash like everything else.
- **Stable and reproducible.** `Brio.lock` pins both the encrypted and decrypted hashes, and builds by the same licensee are bit-for-bit reproducible.
- **Honest revocation.** Removing a licensee applies to future versions; what was already downloaded cannot be recalled, and the tool says so. After a key compromise, a new version gets new keys and the old ones can be marked exposed with an advisory.
- **Leak tracing, optional.** A per-licensee identifier can be embedded without changing behavior, and `brio license trace` reads it from a leaked binary.
- **Visible.** The ledger lists every licensed dependency, its license and expiry, and the toolchain checks the decrypted code's actual permissions against those the publisher declared.
- **Future-proof.** Encryption suites are versioned; weakened suites are retired on a published date with advance warnings, while old artifacts stay decryptable for reproducible rebuilds.

## Rust interop

Brio code uses Rust crates through generated bridges, and Rust code can call Brio the same way, so migration can start at either end of the dependency graph.

### Using Rust crates from Brio

```toml
[rust-dependencies]
tauri = { version = "2", features = ["tray-icon"] }
serde = "1"
```

```brio
use rust tauri.{Builder, AppHandle}
use rust serde.{Serialize, Deserialize}
```

Rust generics only exist once rustc monomorphizes them, so heavily generic APIs need concrete instantiations declared in a bridge block. Tauri, which is generic over its runtime, is the main case:

```brio
rust bridge app_shell {
    instantiate tauri.AppHandle[tauri.Wry]
    instantiate tauri.Window[tauri.Wry]
}
```

### Tauri commands and Rust proc macros

Brio attaches Rust attributes and derives to generated shim code, so Rust proc macros run inside the Rust shim and Brio never has to understand them:

```brio
@rust.derive(Serialize, Deserialize)
public record Vitals { hr: HeartRate, spo2: Percent }

@rust.attr(tauri::command)
public fn get_vitals(id: PatientId) -> Vitals ! AppError {
    ...
}
```

The existing Rust `main.rs` stays unchanged: `tauri::generate_handler![brio_app::get_vitals]`.

### Calling Brio from Rust

A `brio-build` crate used in `build.rs` generates Rust bindings from `.brioh` files, much as the `cxx` crate does for C++. A Rust crate can depend on a Brio package like any other crate.

### Existing Rust macros and comptime

Existing Rust macros keep working on the Rust side of the bridge. New metaprogramming in Brio uses `comptime`, code run by the compiler's built-in interpreter. There is no separate proc-macro crate to compile, and errors always point at the user's code rather than generated code:

```brio
comptime fn derive_eq(T: type) -> Impl {
    quote {
        impl Eq for $T {
            fn eq(self, other: Self) -> Bool =
                $(for f in T.fields join " and ") { self.$f == other.$f }
        }
    }
}

@derive(Eq, Debug)
record Reading { ... }
```

### Keeping the bridge fast

1. **Cross-language LTO.** The Brio toolchain pins the same LLVM version as the project's rustc and uses linker-plugin LTO, so release builds can inline across the Brio–Rust boundary.
2. **Shared layouts.** Slices and strings use Rust's pointer-and-length layout and cross the boundary without conversion. Other Rust types cross by pointer.
3. **A boundary lint.** The compiler warns when a bridge call sits inside a loop marked `@hot`.

## Interop with every language

Rust and Python get deep, dedicated bridges. Every other language gets one uniform mechanism built on three ideas:

1. **The interface file is the contract.** A `.brioh` file already states types, parameter modes, units, ranges and contracts, so Brio needs no separate interface language. UniFFI, by contrast, needs its own UDL files. Every binding is generated from the interface file, so it can never drift from the code.
2. **Three channels, chosen by how close the other code runs.** Each is built on an open standard.
3. **Bindings are extensions.** Each target language's binding generator is a tool extension. The community can add a language without a Brio release, just as with embedded languages.

### Choosing a channel

| Channel | Use it when | Standard | Languages |
| --- | --- | --- | --- |
| Native, in the same process | Speed matters and you trust the other code | A stable, versioned Brio C ABI, with generated idiomatic wrappers | C, C++, Swift, Kotlin, Java (FFM API), C#, Go, Dart, Zig, Node.js, Ruby, … |
| Components, sandboxed in the same process | You need isolation, plugins, or code you don't fully trust | WebAssembly Component Model and WIT, with WASI 0.3 native async | Any language that builds components: Go, JavaScript, Python, C#, C/C++, Rust, … |
| Services, in another process or machine | Separate deployments and teams | gRPC and Protocol Buffers, OpenAPI, GraphQL, AsyncAPI, through embedded languages | Every language |

### Exporting Brio to another language

Mark the public API and name the languages; `brio bind` produces a ready-to-publish package for each.

```brio
@foreign
public frozen record Vitals { public hr: HeartRate, public spo2: Percent }

@foreign
public fn latest_vitals(id: PatientId) -> Vitals ! MonitorError
    requires id != 0 as PatientId
```

```toml
[foreign]
languages = ["kotlin", "swift", "csharp", "typescript"]
```

The generated bindings are idiomatic in each language, not thin C wrappers:

```kotlin
val v = Monitor.latestVitals(PatientId(42u))   // Kotlin: throws MonitorException
```

```swift
let v = try Monitor.latestVitals(id: PatientId(42))   // Swift: throws
```

```typescript
const v = await monitor.latestVitals(42n)   // Node.js: a Promise, or throws MonitorError
```

| Brio | Kotlin | Swift | C# | TypeScript |
| --- | --- | --- | --- | --- |
| `T ! E` | Exception `E` | `throws E` | Exception `E` | Rejected promise or thrown `E` |
| `T?` | `T?` | `T?` | `T?` | `T \| undefined` |
| Frozen record | `data class` | `struct` | `record` | Readonly object type |
| Opaque record | Class, `AutoCloseable` | `final class` with `deinit` | Class, `IDisposable` | Class with `dispose()` |
| Enum with payloads | `sealed class` | `enum` with associated values | Record hierarchy | Discriminated union |
| Trait | Interface, implementable in Kotlin | Protocol | Interface | Interface |
| `async fn` | `suspend fun` | `async` | `Task` | `Promise` |
| Distinct type | `value class` | `struct` wrapper | `readonly record struct` | Branded type |
| Units and ranges | Named wrapper types, checked on entry | Same | Same | Same |

### Importing C and C++

```brio
use c "vendor/pump_hal.h" as hal         // parsed with Clang
use cpp "vendor/imaging.hpp" as img      // classes, methods, std::string, std::vector
```

Imported C functions are `unsafe` until they are described. The description is an *API notes* block written directly on the import, in ordinary Brio:

```brio
use c "vendor/pump_hal.h" as hal {
    fn hal_read(buf: out [U8; 64]) -> I32 ! HalError
        ensures result <= 64
    fn hal_set_rate(rate: MlPerHour) -> () ! HalError
        requires rate <= 1_200.0<ml/h>
}
```

It adds what C cannot say: parameter modes, ownership, units, ranges, nullability and contracts. With those notes, the functions become safe Brio functions, and their contracts are checked at every call. C++ templates are used at explicit instantiations, as with Rust generics. C++ exceptions are caught at the boundary and become errors or traps.

### Why the boundary stays sound

- **Nothing from outside is trusted.** Every value entering Brio is validated: UTF-8 in strings, enum tags, ranges, units and non-null pointers.
- **Ownership is explicit.** Foreign code receives owned handles or borrows that last only for the call. A borrow can never outlive its call, because origins are enforced by the generated wrapper.
- **Callers are checked.** Preconditions of exported functions are checked in Brio on every call. A precondition violated by the foreign caller raises that language's own exception instead of stopping the process.
- **Isolation when needed.** A trap inside a native binding stops the process, as in Brio itself. A trap inside a *component* stops only that component instance, and the host keeps running. For code that must not take its host down, the component channel is the right choice.
- **Foreign failures are contained.** Exceptions thrown by foreign callbacks become Brio errors when the callback's type allows them, and traps otherwise. They never unwind through Brio.
- **Threads are respected.** Objects that are not `Send` are confined to one thread, and the bindings enforce it.
- **Everything is on the ledger.** Every foreign channel, API notes block and binding appears in the assurance ledger and the `brio ship` report.

## Embedded languages

Rust handles GraphQL, SQL, JSON and UI markup with procedural macros such as `view!` and `query!`. Those macros see only tokens, not types. They run arbitrary code at build time, give editors almost nothing to work with, and report errors poorly. Brio replaces them with *embedded languages*: small languages defined in ordinary Brio packages, which plug into the compiler **and** the editor through a stable interface. Teams can add or improve a language, with full editor support, without waiting for a Brio release.

### Using an embedded language

A package provides a language; a module configures it with a `language` item and then writes it with `#name`:

```brio
use gql.graphql
use weave.view
use pg.sql

language api = graphql.Schema(file: "schema/hospital.graphql")
language db  = sql.Postgres(migrations: "db/migrations")

// GraphQL, checked against the schema; the result type is generated from the query
let q = #api{
    patient(id: ${id}) { name  ward { name }  vitals(last: 10) { hr spo2 at } }
}
let result = try await client.run(q)
print(result.patient.ward.name)

// SQL, checked against the migrations; rows are typed records
let rows = try await conn.query(#db"SELECT name, ward FROM patients WHERE id = ${id}")

// JSON built at compile time; every hole must implement ToJson
let body = #json{ "patient": ${id}, "rates": ${rates}, "urgent": true }

// UI markup in the style of Leptos
fn counter(start: Int) -> View {
    var count = signal(start)
    return #view{
        <button on:click={|_| { count += 1 }}>"Clicked " {count} " times"</button>
    }
}
```

Embedded text is written as `#name{ … }`, with balanced braces, or as `#name"…"` and `#name"""…"""` when the text contains unbalanced braces. *Holes* such as `${id}` contain ordinary Brio expressions, which the compiler type-checks. Whole files work too: `language api = graphql.Schema(…, files: "queries/*.graphql")` turns each `.graphql` file into a typed Brio module.

### Defining a language

A language is a type that implements the core trait `Language`. Its methods run at compile time, in the same sandbox as `comptime` code:

```brio
public record Schema { file: Path, files: Str = "" }

impl Language for Schema {
    const lexical = Lexical(holes: .Dollar, strings: .DoubleQuoted, comments: .Hash)

    comptime fn parse(self, src: Source) -> Ast ! Diagnostics { … }
    comptime fn check(self, ast: Ast, cx: LangContext) -> Diagnostics { … }
    comptime fn expand(self, ast: Ast, cx: LangContext) -> Code { … }

    comptime fn complete(self, src: Source, at: Offset, cx: LangContext) -> [Completion] { … }
    comptime fn hover(self, src: Source, at: Offset, cx: LangContext) -> Markdown? { … }
}
```

| Service | Required | Used by |
| --- | --- | --- |
| `parse`, `check`, `expand` | Yes | Compiler and editor |
| `lint` | No | Compiler warnings, `brio lint`, editor |
| `format` | No | `brio fmt`, format-on-save |
| `highlight`, `complete`, `hover`, `signature_help`, `inlay_hints` | No | Editor |
| `definition`, `references`, `rename`, `symbols`, `folding` | No | Editor navigation and refactoring |
| `code_actions`, `fixes` | No | Editor quick fixes, `brio fix` |
| `explain` | No | `brio explain` |

`check` can ask for the Brio type of each hole and read the files the configuration names, such as the GraphQL schema or the SQL migrations. That is how a query is validated against the real schema and gets a precise result type, all at compile time and without a database connection.

### Why this is safer and nicer than proc macros

|  | Rust procedural macros | Brio embedded languages |
| --- | --- | --- |
| What the macro sees | Tokens, no types | A parsed tree, plus the types of the holes |
| Build-time power | Arbitrary code, network, files | Sandboxed: only declared files, no network, deterministic |
| Opening an untrusted repo in an editor | Can run arbitrary code | Runs nothing outside the sandbox |
| Editor support | Mostly none inside the macro | Highlighting, completion, hover, go-to-definition, formatting |
| Errors | Often at the whole macro | At the exact spot in the embedded text, in Brio's style |
| Generated code | Trusted as is | Type-checked like hand-written code; may not contain `unsafe` |
| Debugging | Steps through generated code | Source maps: the debugger and traps point into the embedded text |
| Build speed | Separate crate compile | Cached per literal; re-run only when the text, hole types or inputs change |

The `Language` interface is versioned separately from the compiler, and old versions stay supported, so languages keep working across Brio releases. A language that fails or runs over its time limit produces a diagnostic, never a compiler crash. Rust's function-like macros can still be used from the Rust companion crate while a Brio language is written.

### Linting and formatting embedded languages

A language can ship its own lints and formatter, and they behave exactly like Brio's:

```brio
impl Language for Postgres {
    const lints = [
        Lint(id: "select_star", level: .Warn, summary: "SELECT * breaks when columns change"),
        Lint(id: "unbounded_delete", level: .Deny, summary: "DELETE without WHERE removes every row"),
    ]
    comptime fn lint(self, ast: Ast, cx: LangContext) -> Diagnostics { … }
    comptime fn format(self, src: Source, style: FormatStyle) -> String? { … }
}
```

```toml
[lints.db]                 # in Brio.toml, per language item
select_star = "deny"
```

- **Lints** are named and documented, have a default level, and are configured in the manifest or on a single item with `@lint.allow(db:select_star)`. Lints at level `deny` fail the build. Their diagnostics follow Brio's what / why it matters / how to fix style, with fixes that `brio fix` can apply.
- **Formatting** runs on `brio fmt` and on save. The language formats the text and Brio re-indents it to fit the surrounding code, leaving holes untouched. Brio re-parses the result and rejects any formatting that changes the parsed tree, so a formatter bug can never change what the code does.

## Extending the developer experience

Embedded languages are one use of a broader idea: **the Brio compiler is a platform for developer tooling**. Any package can provide a *tool extension* that adds lints, refactorings, editor features and commands for Brio code itself. The team installs it as a development dependency and it works immediately in the terminal, the editor and CI, without a compiler release.

```toml
[extensions]
hospital-rules = { path = "tools/hospital-rules" }
brio-tauri-kit = "1.2"
```

| Extension point | Example for a codebase |
| --- | --- |
| Lints | "Database calls must run inside a transaction"; "`Sensitive.expose` only in the audit module"; "every Tauri command must be `@traced`" |
| Code actions and refactorings | "Turn this function into a Tauri command"; "Add `@req` from the requirements file"; "Convert this Python-style loop" |
| Editor features | Inlay hints showing units and dose ranges; code lenses such as "run benchmark" or "3 flags read here"; hovers; snippets; extra highlighting |
| Explanations | Team-specific pages for `brio explain` |
| Documentation | Extra sections and diagrams in `brio doc` |
| Commands | New subcommands such as `brio openapi` or `brio release-notes` |
| Test support | Generators for property tests, and fixtures |

An extension implements the core trait `Extension` and receives a read-only view of the compiled program: typed syntax trees, symbols, types, contracts, the call graph, flags and the assurance ledger.

```brio
impl Extension for HospitalRules {
    const lints = [Lint(id: "db_outside_tx", level: .Deny,
                        summary: "Database writes must run inside a transaction")]

    comptime fn lint(self, m: ModuleView, cx: ToolContext) -> Diagnostics {
        var out = Diagnostics()
        for call in m.calls() {
            if call.target.has_attr("db.write") and not call.inside_call_to("db.transaction") {
                out.add(cx.diagnostic("db_outside_tx", at: call.span)
                    .why("a failure halfway through would leave records partly written")
                    .fix("wrap it in db.transaction", cx.wrap(call, "db.transaction(|tx| { … })")))
            }
        }
        return out
    }
}
```

### Why extensions can't break anything

- **Analysis only.** Extensions report diagnostics, offer edits and run commands. They cannot change how any code compiles, so a buggy extension can never make a program unsound. Code generation stays with `comptime` and embedded languages, which are fully checked.
- **Edits are checked first.** Every edit an extension offers is compiled before it is shown; an edit that would break the build is dropped.
- **Sandboxed and deterministic.** Extensions run in the same sandbox as `comptime` code, with time limits. Commands can write only to the paths they declare. Results are identical in the editor, the terminal and CI.
- **One style.** Extension diagnostics use Brio's diagnostic style, with codes such as `hospital_rules:db_outside_tx`.
- **Stable.** The extension API is versioned and stays compatible within an edition. Results are cached per module, so extensions stay fast in large codebases.
- **Visible.** The extensions used, and their lint settings, are recorded in the `brio ship` report.

## Safety profiles

Every module runs under a profile. `standard` is the default; `critical` is a stricter subset for code where failure is not acceptable, similar to Ada's Ravenscar profile or MISRA C.

```brio
@profile(critical, level: "IEC62304-C")
module pump.dosing
```

| Rule | `standard` | `critical` |
| --- | --- | --- |
| Heap allocation | Allowed | Only during initialization |
| Recursion | Allowed | Forbidden |
| Traps (failed checks) | Stop the process after the trap handler runs | Proven impossible |
| `unsafe` | Allowed with justification | Forbidden |
| Contracts | Checked or proven per module | Must be proven |
| Loops | Bounded by default; `unbounded fn` where needed | Must be provably finite |
| References in record fields | Allowed | Forbidden (value semantics) |
| Concurrency | Structured tasks | Static thread pool created at startup |

Until the toolchain is qualified, the compiler marks `critical` builds as unqualified in their build metadata and in `brio ship` reports. Teams can adopt the discipline immediately without implying certification.

### Finite loops

```brio
for i in 0 ..< n
    invariant total <= MAX_DOSE
{ ... }

while pressure.rising()
    bounded 1_000
{ ... }
```

### Justified unsafe code

Every `unsafe` block must state why it is sound. The justification appears in audit reports:

```brio
unsafe("DMA buffer pinned for the transfer lifetime; see HAZ-112") {
    ...
}
```

### Requirement traceability

Functions can be tagged with the requirements and hazards they implement. The compiler emits a traceability matrix linking requirements, code and tests:

```brio
@req("SRS-4.2.1", "HAZ-007")
fn stop_on_occlusion(pump: inout Controller) { ... }
```

### Structured concurrency

No task outlives its scope, and cancellation is cooperative: a cancelled task sees a `Cancelled` error at its next wait and runs its own cleanup. Tasks compile to state machines, as Rust's async does, with no mandatory runtime:

```brio
tasks {
    spawn monitor(sensor)
    spawn alarm_loop(bus)
}   // all tasks are joined here
```

## Runtime performance

Brio release builds must run within 3% of equivalent idiomatic Rust by geometric mean, with no benchmark more than 10% slower, measured against Rust with overflow checks enabled and enforced by a benchmark suite in CI. That is the floor. Release builds use the same LLVM optimizer as Rust and C, so release compile times will be similar to Rust's; the big build-time gains are in development builds.

### Faster where it matters, without new risk

Safety checks rarely cost much by themselves: a predicted branch is cheap. What costs is what they block. A bounds check in a loop can stop the compiler from vectorizing it, which can halve its speed, and a possible aliasing between two buffers stops the same optimizations in C unless the programmer remembers `restrict`. Brio knows things C and Rust compilers do not, and gives them to the optimizer, but only when they are proven or checked, never assumed:

| What Brio knows | What the optimizer gets | Where it pays |
| --- | --- | --- |
| A check can never fail, proven from ranges, loop invariants or contracts | The check is deleted, and the loop can vectorize | Numeric kernels, parsers, buffer processing |
| `inout` and `take` parameters cannot overlap anything, by the exclusivity rules and a machine-checked aliasing model | The equivalent of C's `restrict`, on every such parameter automatically | Code with several buffers, like DSP and packet processing |
| Nothing unwinds | No landing pads or cleanup paths; traps are cold branches | Smaller binaries and simpler control flow everywhere |
| A function is proven to terminate, or proven pure | It always returns, or touches no memory, so it can be hoisted and combined | Hot loops calling small helpers |
| Results and `out` parameters are built in place | No hidden copies of large values | Code that builds large records |

Three rules keep this sound. Facts from `assume`, from foreign code or from bridge contracts never reach the optimizer; every value entering from C, Rust, Python or device memory is checked before the optimizer may rely on its type; and every toolchain release is checked with translation validation and by comparing the two code generators and the interpreter, because Rust's history with its own non-aliasing information showed that optimizers have bugs exactly here.

Performance is also guaranteed, not hoped for. `return tail f(x)` must be a real tail call, `@vectorize` must vectorize, `@inline(always)` must inline, and `@max_stack(n)` and `@no_copy_larger_than(n)` must hold, or the build fails and says why. `brio build --report checks` lists which checks were removed and why the others could not be, which is the guide to where a loop invariant or range type would pay off.

**What we promise, honestly.** Never measurably slower than checked Rust. Faster than checked Rust and ordinary C on named workloads, each backed by a benchmark anyone can rerun. Not "faster than C" in general, and never "safety for free". Benchmarks are measured over many randomized code layouts, because code placement alone can move results by several percent, and CI gates on deterministic measurements such as instruction counts and code size on every change.

**Critical code.** Safety-critical builds use a restricted set of optimizations validated function by function, and no profile-guided or post-link optimization, which regulators have not accepted. Even so, the precedent is encouraging: avionics and nuclear projects that moved from unoptimized qualified builds to a verified optimizing compiler reported about 10% lower worst-case execution time and up to 28% lower worst-case processor load.

### Design rules that keep Brio fast

- **No hidden costs.** No garbage collector, no runtime, no hidden allocations. Ownership gives deterministic destruction.
- **Monomorphized release generics.** Dictionary passing is a development-build speedup only.
- **Direct layout.** Records are laid out as in Rust, with field reordering and `@repr(C)` for FFI. Enums get niche optimization, so `Option[ref T]` is one pointer wide.
- **Aliasing information.** `inout` guarantees exclusive access, so the compiler marks it `noalias` for LLVM. C compilers only get this with `restrict`.
- **Range facts.** Range types become LLVM range metadata, letting the optimizer delete impossible branches and prove indices valid.
- **Compile-time work.** Units are erased, and `comptime` moves table generation and parsing into the compiler.

### Runtime check policy

| Check | `standard` release | `critical` |
| --- | --- | --- |
| Bounds checks | On (mostly removed by iterators) | Must be proven |
| Integer overflow | On | Must be proven |
| Preconditions on public functions | Checked | Must be proven |
| Internal contracts | Off, or proven where possible | Must be proven |
| Range types | Checked on conversion only | Must be proven |

### Enforced performance properties

Attributes turn performance expectations into compile errors:

```brio
@hot @no_alloc @no_dyn
fn filter_ecg(samples: inout [F32; 512], coeffs: [F32; 32]) {
    for i in 32 ..< 512 {               // bounds proven from the range
        samples[i] = dot(samples[i-32 ..< i], coeffs)
    }
}
```

The build fails if this function allocates, uses dynamic dispatch, or keeps an unproven bounds check. Explicit SIMD types, `@inline(always)`, explicitly passed allocators and justified `unsafe` blocks are available for the last few percent.

## Observability and reliability

Brio treats observability, testing under failure and controlled rollout as part of the language, not as vendor SDKs added later. Two rules apply throughout:

- **No lock-in.** Every signal uses an open standard, and every feature works with no external service at all. Where the data goes is configuration, chosen at deploy time.
- **Nothing happens in the dark.** Every flag, fault point, metric and telemetry export is declared in code, so the compiler can list, check and test it.

| Need | Brio feature | Open standard |
| --- | --- | --- |
| Traces, metrics, logs | `@traced`, `metric` items, `log` | OpenTelemetry (OTLP), W3C Trace Context |
| Continuous profiling | Built-in sampling profiler | OpenTelemetry profiles, pprof |
| Crash reporting | Crash reports from the trap handler | OTLP log events and a documented JSON format |
| Feature flags and A/B tests | `flag` and `experiment` items | OpenFeature provider interface |
| Benchmarks | `bench` items with statistical baselines | Plain files in the repository |
| Chaos and failure testing | Fault points and deterministic simulation | None needed: part of the toolchain |
| Provenance | Build attestations and bill of materials | SLSA, SPDX or CycloneDX |
| Local viewing | `brio observe`, a local viewer for all of the above | Reads OTLP files |

*Note:* The OpenTelemetry profiles signal entered public alpha in March 2026 and is not yet recommended for critical production use, so Brio also writes pprof files until it matures.

### Telemetry in the language

```brio
metric infusions: Counter[(model: PumpModel, outcome: Outcome)]
metric rate_error: Histogram[F64<ml/h>](buckets: [0.1<ml/h>, 0.5<ml/h>, 1.0<ml/h>])

@traced(patient, rate)
fn start_infusion(patient: PatientId, rate: MlPerHour, pump: inout Pump) -> () ! PumpError {
    log.info("starting infusion", patient, rate)
    try pump.set_rate(to: rate)
    infusions.add(1, model: pump.model, outcome: .Started)
}
```

- **Typed metrics.** Metric labels are typed, and their types must have a fixed, small set of values, such as enums or `Bool`. This catches the classic failure where a metric labelled by user ID overwhelms the monitoring system, at compile time.
- **Context flows automatically.** Trace context follows `await`, `spawn` and `tasks` blocks, and crosses the Rust bridge into Rust's `tracing` spans, so Rust libraries nest correctly in Brio traces.
- **Sensitive data cannot leak.** Values of type `Sensitive[T]`, such as patient names, print as `‹redacted›`, and passing one to `log`, `@traced` or a metric is a compile error unless it is explicitly released with a reason.
- **Telemetry never breaks the program.** Recording is non-blocking and uses bounded memory; if an exporter fails, data is dropped and counted, never retried forever. In `critical` code, telemetry writes to a fixed ring buffer and never allocates.

### Crash reports

When a trap occurs, the trap handler writes a crash report before the process stops. It contains the trap information, the stack trace, the last spans and log events from the ring buffer, the active flag values and the build identifier. Reports are symbolicated offline from the debug information `brio ship` archives, and sent over OTLP when the program next starts. This is Crashlytics-style reporting, with no third-party service required.

### Contracts in production

A new verification mode, `observe`, evaluates a contract at runtime and reports a violation as a telemetry event instead of trapping. It lets a team roll out a new contract, watch whether production ever violates it, and then switch it to `check` or `prove`. `observe` is not allowed in `critical` modules.

### Feature flags and experiments

```brio
flag new_alarm_tones: Bool = false

experiment dashboard_layout: Layout by ClinicianId {
    control    = .Classic
    metrics    = [task_seconds, correction_rate]      // the first is the primary metric
    guardrails = [alarm_ack_seconds, crash_rate]      // must not get worse
    hypothesis = "The compact layout shortens charting without slowing alarm response"
}

if new_alarm_tones.value() { alarms.use_tones(.V2) }
let layout = dashboard_layout.variant(for: clinician.id)   // records the exposure
```

Flags and experiments are items, so the compiler knows all of them. `brio flags` lists every flag with its uses; flags that are always on or never read are reported for removal; and `brio test --flags all` runs the tests under every flag combination, or all pairs when there are many. Values come from any OpenFeature-compatible provider; the default provider reads a local file. In `critical` modules, flag values are fixed during initialization, and proofs must hold for every value.

### Experiments under full control

The code only declares what can vary and how success is measured. Everything else, meaning who sees what, when and how much, is controlled at run time, with three levels:

| Level | What it is |
| --- | --- |
| Experiment | The item in source: its variants, control, metrics, guardrails and hypothesis |
| Run | One execution of an experiment with a fixed plan: allocation, targeting, sample size, statistical method, ramp stages and stopping rules |
| Campaign | A group of runs, across experiments and products, serving one goal with one owner, optionally with a holdout group to measure their combined effect |

**Results you can trust.**

- **Planned before started.** A run cannot start without a plan: hypothesis, primary metric, the smallest effect worth detecting, significance and power. The toolchain computes the sample size and duration. Changing the plan means starting a new run, so goals cannot drift after the data arrives.
- **Safe to watch.** Results in progress use sequential methods that stay valid however often you look, so checking the dashboard daily never produces false winners.
- **Many comparisons accounted for.** With several variants or metrics, error rates are controlled across all of them, and everything but the pre-registered primary comparison is labelled secondary.
- **Broken experiments are caught.** The service constantly checks that each variant receives the share of units it should. A mismatch means assignment or logging is broken; results are withheld and the run can pause itself.
- **Stable assignment.** A unit's variant comes from a keyed hash, so it is the same in every process and on every machine, needs no database, and changing the allocation moves as few units as possible. Runs in the same layer never share a unit, so experiments cannot contaminate each other.
- **Reproducible.** Exposures and metric events are exported over OpenTelemetry, and every result can be recomputed from them, the plan and the analysis version.

**Power over every run.**

- **Ramping** through stages such as 1%, 5% and 50%, advancing automatically only when guardrails pass.
- **Guardrails that act:** a variant that measurably worsens a guardrail is stopped and its users returned to control automatically, and the owners are notified.
- **A kill switch** on every run that returns everyone to control instantly.
- **Targeting** by rules on user attributes, never on secret or sensitive data, and **offline plans** exported as signed files for sites without a connection.

**A complete overview.** The experiment service, part of the registry server or standalone, shows every campaign, experiment and run with its status, plan, allocation history, exposures, sample-ratio check, results with their uncertainty, guardrails and decisions. The web portal and the API can do exactly the same things; the API has versioning, idempotency keys, scoped tokens and signed webhooks for every event, from run started to guardrail stopped. Planning, starting and concluding are separate roles, an organization can require a second person's approval, every change needs a passkey, and every change is in the tamper-evident audit log with who, when and why. It is part of `brio registry dev`, so experiments can be tested locally end to end.

### Build your own experiments

Developers decide what an experiment assigns, whom it targets, how it balances its groups and what it measures, using any data their program can reach, such as a database, an API or a computed value:

```brio
@audience(refresh: 24<h>, units: all_hospitals)
fn hospital_traits(ids: [HospitalId]) -> Map[HospitalId, HospitalTraits] ! DbError
    uses db
{
    // one query for the whole batch, not one per hospital
    let rows = try #db"SELECT hospital_id, count(DISTINCT user_id) AS mau, region
                       FROM sessions JOIN hospitals USING (hospital_id)
                       WHERE hospital_id = ANY(${ids})
                         AND started_at > now() - interval '30 days'
                       GROUP BY hospital_id, region"
    return rows.to_map(|r| (r.hospital_id, HospitalTraits(monthly_active_users: r.mau, region: r.region)))
}

@experiment_outcome(window: 14 * 24<h>)
fn median_wait_minutes(id: HospitalId, from: Instant, until: Instant) -> F64 ! DbError
    uses db
{ … }

experiment triage_queue: QueueMode by HospitalId {
    control    = .Fifo
    audience   = hospital_traits
    eligible   = |h| h.monthly_active_users >= 10_000
    balance_by = |h| h.region
    metrics    = [median_wait_minutes]
}

flag new_triage_screen: Bool {
    default  = false
    audience = hospital_traits         // flags use the same audience for their rules
}
```

| Construct | What it is |
| --- | --- |
| `by HospitalId` | Any unit: users, devices, hospitals, tenants, regions, or your own type |
| `@audience(refresh: …, units: …)` | A function that computes typed attributes for units from any source, one at a time or in batches; shared by flags and experiments. `units` lists every unit, so the audience is complete |
| `audience`, `eligible` | Which audience to use, and who in it may take part |
| `balance_by` | Keeps every group in the allocated proportions within each value, such as each region |
| `@experiment_outcome(window: …)` | A function that measures a unit's result from any source, over a window after exposure |

**Why custom stays sound.** Custom data is where A/B tests usually go wrong, so these rules are built in:

- **Assignment never queries anything.** Audience functions run in the background, in batches, on exactly one designated runner (one instance of your program or a worker of the experiment service), and publish signed, versioned snapshots that every other instance receives. `variant(for:)` reads only the snapshot, so it never blocks and never fails; a unit without attributes gets the control variant and is counted separately in the overview.
- **Attributes are fixed before treatment.** A unit's eligibility and group are set at its first assignment, or at run start for a frozen cohort, and never re-evaluated. Otherwise an experiment that raises monthly active users would move hospitals in and out of its own sample. The compiler also warns when an attribute reads the same data as an outcome.
- **Outcomes count only what happened after exposure,** inside the stated window.
- **Groups are analysed as groups.** When hospitals are assigned but patients are measured, sample size and confidence account for patients within one hospital resembling each other, a common cause of false positives.
- **Plans are type-checked.** Eligibility rules added in the portal are checked against the attribute types published by the program, so a mistyped rule is refused before the run starts.
- **The randomization is never custom.** Units, attributes, eligibility, balance and outcomes are yours; the mapping from units to variants is always the proven keyed hash, which is what keeps every result valid.
- **Checked before it runs.** Tests can supply attribute snapshots and outcome data as fixtures, and `brio sim` runs a whole experiment on synthetic units with a planted effect, reporting whether the analysis finds it and how often it would report an effect that is not there.

**Finishing cleanly.** Concluding a run records the decision and its evidence. The compiler then points at the experiment's code and offers a fix that replaces every `variant(for:)` with the chosen variant, and experiments without an active run are flagged for removal, so old experiments never linger in the code.

**Patient safety.** An experiment's variant can never influence critical code, which the compiler checks. In products with critical modules, every experiment must name the ethics or clinical approval it runs under, and that approval is recorded with every run and in the release's ledger.

### Benchmarks

```brio
bench "ECG filter, 512 samples" {
    var samples = fixtures.ecg_512()
    measure { filter_ecg(mut samples, COEFFS) }
}
```

`brio bench` warms up, takes repeated samples and reports confidence intervals, allocation counts and, where the hardware supports it, instruction counts, which do not suffer from machine noise. Baselines live in the repository, and `brio ship` fails when a benchmark regresses beyond a configured threshold.

### Chaos and deterministic simulation

- **Fault points.** Every standard-library I/O call, every bridge call and every `@fault_point("name")` can be made to fail, slow down or be cancelled. `brio test --faults` runs the tests with faults injected, checking that every error path works.
- **Deterministic simulation.** `brio sim --seed 42` runs the whole program with a simulated clock, network, disk and task scheduler, and injects faults from the seed. A failure found this way replays exactly from its seed, which makes rare concurrency bugs reproducible.
- **Testing the tests.** `brio mutate` changes the program in small ways and reports changes that no test or contract notices.
- **Chaos in production** is available only in builds that explicitly enable it, and never in `critical` builds.

### Inspection and provenance

`brio inspect` attaches read-only to a running program and shows its tasks, locks, channels and flag values; it is disabled in release builds unless configured. `brio ship` records build provenance and a software bill of materials in open formats, so anyone can check what went into a release.

## Accelerators

Kernels for GPUs and other accelerators are ordinary Brio functions marked `@kernel`, compiled for the accelerator and, with identical meaning, for the host processor.

```brio
@kernel
fn scale(item: WorkItem, input: DeviceSlice[F32], out: inout Partitioned[F32, 1], k: F32) {
    let i = item.global_index            // proven in range for buffers of the launch size
    out.mine[0] = input[i] * k            // each work item writes only its own element
}

let result = try await accelerator.launch(scale, dims: n, input, mut out, 2.5)
```

- **No data races by construction.** Input buffers are read-only; each work item owns its own slice of each output buffer; anything else shared must be atomic. Work-group memory is checked the same way between barriers.
- **No stuck barriers.** The compiler proves every barrier is reached by all work items of a group or by none.
- **Ownership of device memory.** Buffers are moved to the device and back with `take`, so the host cannot touch them while a kernel runs.
- **Safe failure.** A failed check ends the launch with an ordinary `KernelTrap` error, and the buffers it could have written are discarded.
- **Tested against the host.** `brio test` runs every kernel test on the host and on each available accelerator and compares the results, which catches driver and compiler bugs. Accelerator drivers are listed in the trusted base, launching needs the `Accelerator` capability, and kernels are not allowed in critical code.

## Software updates

`brio ship --update` produces signed update packages, and the runtime installs them safely:

- **Verified first.** Signatures, hash and version constraints are checked before installing, and a monotonic, hardware-backed security version prevents downgrades to vulnerable releases. Update metadata uses an offline root key and a threshold of signatures; fleet frameworks such as Uptane fit this model.
- **Atomic, with automatic rollback.** Updates install into a second slot. If the new release fails its health check or traps before confirming, the device returns to the previous release by itself.
- **Data survives rollback.** Persistent data migrated by a new release stays readable by the old one until the new release is confirmed.
- **Never during therapy.** On critical devices, an update is installed only in a state the safety state machine declares safe for updating, never interrupts a function in progress, and completes or rolls back within a stated time that is part of the timing analysis.
- **On the record.** Every installed, refused and rolled-back update is logged on the device and reported through telemetry, and `brio ship` keeps each product's update history with the ledger of every release.

## Proofs, measurements and what remains to be done

The normative spec now contains everything that was previously left for later: the complete aliasing model (Annex E), the soundness theorems the core must satisfy (Annex F), the named benchmark suite and performance claims, and the procedure for qualifying cross-package optimization in critical builds (Annex G). No design question is open.

What remains is doing the work, which no document can replace: machine-checking the theorems, building the toolchain and running the benchmarks, obtaining the independent assessment, and registering the trademark. The ledger keeps us honest about it: a theorem is shown as *proven* only once its proof has been checked, and a performance claim is published only once the benchmark supports it.

## Package manager and registries

The package manager is part of the `brio` binary, and so is a registry server. A single developer, a five-person team and a large organization use the same commands and manifests; only the registry configuration differs.

### Everyday use

```text
$ brio add ward-registry
  Added ward-registry 2.4.1 (compatible with "2.4")
  Permissions: fs.read · no unsafe · no foreign code
  Advisories: none

$ brio why xmlparse
  xmlparse 2.3.4  ←  hl7 1.8.0  ←  your package

$ brio update
  hl7       1.8.0 → 1.9.2
  xmlparse  2.3.4 → 2.3.5     new permission requested: none
```

| Command | What it does |
| --- | --- |
| `brio add`, `brio remove` | Add or remove a dependency at the newest compatible version |
| `brio update`, `brio outdated` | Update within the manifest's ranges, or list newer versions, showing new permissions and advisories |
| `brio tree`, `brio why` | Show the dependency graph, and why a package is in it |
| `brio search`, `brio info` | Find packages and see their permissions and advisories before adding them |
| `brio publish --dry-run` | Run every publishing check without publishing |

The lockfile is never changed behind your back: only `add`, `remove` and `update` touch it, and CI runs with `--locked`.

### Resolution you can understand

Brio resolves versions with PubGrub, the algorithm behind Dart's, Swift's and uv's package managers. It always finds a solution when one exists, and when none does, it explains why in your own package names:

```text
── VERSIONS CANNOT BE RESOLVED ────────────────────────────── BI0301

ward-registry 3.x needs hl7 2.6 or later,
but every hl7 from 2.6 on requires Brio toolchain 1.6,
and this project uses toolchain 1.5.

How to fix, pick one:
  1. Upgrade the toolchain to 1.6                  [recommended · brio upgrade-check 1.6]
  2. Stay on ward-registry 2.4, which works with hl7 2.5
```

Features are additive by construction: the compiler checks that enabling a feature only adds to a package's interface, so features can never break each other.

### Fast by design

- **Dependencies are not compiled.** Brio packages ship compiled code and interfaces per target, so dependencies are downloaded and linked. This removes most of the build time that Rust spends compiling crates. Registries rebuild every open-source package independently, bit for bit, before it can be used, so a downloaded binary always matches its source; packages without public source are marked unverifiable.
- **One shared store per machine.** Every artifact is downloaded once per machine, stored by content hash, and linked into projects rather than copied.
- **Sparse, cached index.** Resolution fetches metadata only for packages in your graph, downloads run in parallel, and everything is verified while it streams.

### Secure by default

- **No dependency confusion.** Each dependency comes from exactly one registry and is never looked up elsewhere after a miss, so a public package can never impersonate a private one. Organization scopes such as `hospital/ward-registry` belong to one organization.
- **Tamper-proof metadata.** Registry metadata is protected with The Update Framework, which blocks rollback and freeze attacks, on top of per-package signatures and provenance.
- **No long-lived tokens.** CI publishes through short-lived OpenID Connect credentials (trusted publishing); people publish after signing in with a passkey.
- **Policies at the registry.** A registry can refuse packages with disallowed licenses, missing provenance, serious advisories, unreviewed code or unwanted permissions, for the whole organization at once.

### Signing in

Registries have no passwords. People sign in with a **passkey** or an **email code**, or through the organization's single sign-on.

```text
$ brio login
  Opening https://packages.hospital.local/login in your browser…
  Signed in as j.devries (read, publish:hospital/*), valid for 12 h, bound to this machine.
```

| Method | Good for | Why |
| --- | --- | --- |
| Passkey | Everything, including publishing | Phishing-resistant: it only works on the real registry's domain |
| Email code | Signing in, browsing and downloading | Easy, but anyone who can read the mailbox, or trick you into typing the code elsewhere, can use it |
| Single sign-on | Everything, when the identity provider uses phishing-resistant sign-in | Uses the organization's existing accounts |

- **Publishing needs a passkey.** Publishing, yanking, changing owners, creating publish tokens and changing sign-in methods require strong sign-in, because a malicious publish reaches every downstream user. A registry operator can relax this deliberately, and every such action is marked in the audit log.
- **Email codes are hardened.** They are single-use, expire within ten minutes, are rate-limited, and never reveal whether an account exists.
- **The command line signs in through the browser.** `brio login` opens the registry's sign-in page in your browser and receives the result directly on your machine, so an attacker cannot trick you into signing in on their behalf. The CLI gets a short-lived token limited to the scopes it asked for, bound to this machine and kept in the OS key store. The code-entry flow some tools use is available only for machines without a browser, and never for publishing. CI never signs in; it uses trusted publishing.
- **Recovery is safe.** A session opened with only an email code cannot add or remove sign-in methods or gain publishing rights. A new passkey needs an existing one, or an organization administrator's confirmation or the registry's support process followed by a 72-hour waiting period, during which every sign-in method is notified and can cancel and publishing is suspended. Publishers must register two passkeys, and recovery codes are offered at sign-up.
- **Self-hosted too.** `brio registry serve` includes all of this: configure the domain for passkeys and a mail server for email codes.

### Managing licenses and licensees

The registry server includes full license management, in a web portal and through an API with exactly the same capabilities, so it can be run by hand or wired into billing, CRM and support systems.

| Concept | What it is |
| --- | --- |
| Product | One or more licensed packages sold together |
| Plan | What a license grants: layers, targets, version range, seats, and build rights after expiry |
| Licensee | A customer organization, with its own administrators and members |
| Licensee key | A public key enrolled by the licensee for a developer, build machine or CI system |
| License | A licensee's right to a product under a plan, for a period |

**The key custodian keeps it secure.** Adding a licensee to a version that already exists needs that version's content key. Rather than giving the registry every content key, which a breach would expose, keys are protected by a *key custodian*: a small, isolated service backed by **OpenBao's Transit engine**, the open-source encryption-as-a-service engine of the Vault fork.

| Transit feature | How Brio uses it |
| --- | --- |
| Non-exportable keys | The publisher's key-encryption key never leaves OpenBao |
| Derived keys with a context | Every content key is bound to its package, version, layer and target; the custodian computes the context from the signed manifest, never from a caller |
| Encrypt-only access for publishing | The publishing job generates and wraps content keys itself and may only encrypt them into OpenBao, never decrypt |
| Versioned rotation, rewrap, minimum decryption version | Key-encryption keys rotate on schedule; stored content keys are re-wrapped without ever appearing in plaintext, and old versions are retired |
| Policies and audit devices | Returning data keys in plaintext and convergent encryption are forbidden in production; every operation is logged and matched to the registry's own audit log |

One step cannot happen inside Transit: wrapping a content key for a new licensee's public key (HPKE). Where a hardware security module or an attested enclave is available, the custodian does it there, so the key never touches ordinary memory. Otherwise it holds the key as a `Secret` for milliseconds, zeroes it, and the affected licensees' ledgers record this exposure. Every wrapped key is signed, and every layer uses key-committing encryption, so no one, not even a compromised partner, can give two licensees different code for the same signed release. Storage, mirrors and the web tier still hold nothing secret. The backend interface is pluggable, so a cloud key-management service or a hardware security module can replace OpenBao without lock-in.

**Everything else is automatic.**

- **Granting a license** wraps keys for every covered version, layer and target, for every enrolled key of the licensee, in one batch and without re-encrypting anything.
- **Licensees manage their own keys.** Their administrators enroll one key per developer, build machine or CI system through the portal, the API or `brio license key enroll`; private keys never leave them. Plans can cap the number of keys (seats).
- **Tokens renew themselves** while a license is active. Air-gapped licensees download a signed offline bundle instead.
- **The full lifecycle** is supported: trials, full licenses, extensions, upgrades, suspension, expiry and termination. None of them ever affects programs already built.
- **Build rights after expiry.** A plan can grant perpetual build rights for every version released during the license. For medical products this should be the default, so a customer can always rebuild the version it shipped to fix a defect.

**An API built for automation.**

- **Documented and stable:** an OpenAPI description, versioned, backward compatible within a major version, and each major version kept for at least twelve months after the next one. The portal and the `brio` command line use nothing else.
- **Safe to retry:** write operations take idempotency keys, lists are paginated, and every call is traceable in the audit log.
- **Least privilege:** tokens carry scopes such as `licenses:write` or `keys:enroll`, can be limited to one product, and write scopes require a passkey.
- **Webhooks:** signed events for licenses issued, changed, expiring, expired and terminated, keys enrolled and removed, seat limits reached, versions published and downloads, delivered at least once with IDs for deduplication.

**Oversight.** Publishing, license management, licensee self-service and administration are separate roles, and an organization can require a second person to approve license changes. Every operation needs a passkey and is recorded in a tamper-evident, hash-chained audit log. Reports show active licenses, seats in use, upcoming expiries and downloads per licensee, exportable in open formats. The registry stores only the personal data it needs, with export and deletion supported, as EU data-protection law expects.

**Local development with one command.** `brio registry dev` starts the whole licensed-registry stack on a laptop or in CI: the registry server with license management, the custodian with an OpenBao development server running Transit, and **Floci**, a free open-source local AWS emulator, providing S3-compatible storage and KMS at `localhost:4566`. The same stack is available to tests as container fixtures, so publishing, licensing, key enrollment and downloads can be tested end to end. Everything the development stack creates is marked as such: production registries refuse it, and `brio ship` refuses a release that depends on it, and it is signed by separate development root keys that production never trusts, so development keys can never leak into a product.

### Your own registry, at any size

| Team | Setup | Command |
| --- | --- | --- |
| One developer | The public registry, zero configuration | none |
| Small team | A static registry: just files in a bucket, a file share or a Git repository; no server to run | `brio registry init --static s3://team-packages` |
| Organization | A registry server with single sign-on, organizations and roles, web interface with search and docs, policies, audit log, backups and replication | `brio registry serve` |
| Regulated or air-gapped | The server, mirroring the public registry and proxying Rust crates and Python packages, so every dependency comes from one internal, audited source | `brio registry mirror` |

The registry protocol is open and documented, and no registry is privileged, including the public one. Moving from a static registry to a server, or to a different host, changes only configuration.

### Many registries and licenses in one program

A real product mixes public packages, internal packages, licensed packages from several vendors, Rust crates and Python packages:

```toml
[registries]
internal = "https://packages.hospital.local"
vendor-a = "https://registry.monitorsoft.example"

[dependencies]
hl7           = "1.9"                                        # public registry
ward-registry = { version = "2.4", registry = "internal" }
vitals-core   = { version = "3.1", registry = "vendor-a" }  # licensed
wr-legacy     = { package = "ward-registry", version = "1.2", registry = "vendor-a" }

[rust-dependencies]
serde = "1"
```

```text
$ brio licenses
  vitals-core 3.1   vendor-a   license MS-2291   code, bitcode   expires 2027-03-31
  dsp-kernels 2.0   vendor-b   covered by vitals-core (redistribution)
  All licenses present. Next expiry in 182 days.
```

How this stays sound, predictable and fast:

- **Identity includes the registry.** `ward-registry` from `internal` and from `vendor-a` are different packages; giving one a local name (`wr-legacy`) lets both be used side by side. The lockfile records every package's registry, and mirrors keep the identity of the registry they copy.
- **One resolution for everything.** Brio packages from every registry, Rust crates and Python packages are resolved in one run into one lockfile. Index data is fetched from all registries in parallel, and identical content is stored once, wherever it came from.
- **Licensed packages that use other licensed packages.** You need a license for each licensed package you must decrypt, unless the vendor in between has *redistribution rights*: then the other vendor's custodian wraps keys for you automatically, and neither vendor ever sees the other's keys. `brio licenses` shows which is which.
- **Every missing license at once.** Before downloading anything, the toolchain checks every license the whole graph needs and reports all missing or insufficient ones in one message, with the vendor and how to obtain each, never one failed build at a time.
- **Credentials never cross.** Sign-in and license tokens are kept per registry and per license in the OS key store, and a token for one registry is never sent to another.
- **Offline first, no silent fallback.** With everything in the local store and valid licenses, a build contacts no registry at all. If a registry is down and something is missing, the error names that registry; no other registry is tried, so an outage can never become a dependency-confusion attack.
- **Vendoring stays encrypted.** `brio vendor` copies licensed packages only encrypted, with your wrapped keys and license tokens, so an offline vendored folder never contains decrypted vendor code.
- **No surprises at release.** You are warned 30 days before a needed license expires, and `brio ship` reports every license that would lapse within the release's support period.

Everything else works unchanged across sources: permissions are checked per package whatever its registry, licensed code is checked against its declared permissions once decrypted, and remote build caches only share results built from licensed code with holders of the same license.

## Supply-chain security

A Brio program depends on Brio packages, Rust crates, Python packages, C libraries and embedded-language packages. `brio audit` checks all of them for known vulnerabilities in one pass, and the language design closes the most common supply-chain attack routes. Since 11 September 2026, the EU Cyber Resilience Act requires manufacturers to report actively exploited vulnerabilities within 24 hours, so knowing precisely which releases are affected is now a legal need as well as an engineering one.

### Finding vulnerabilities

```text
$ brio audit

── REACHABLE VULNERABILITY ───────────── BRIO-2026-0042 · high

`xmlparse 2.3.1` (Rust crate) has an entity-expansion flaw in `parse_document`.
Your code reaches it:

  api.import_order  →  hl7.decode  →  xmlparse.parse_document

Why it matters: a crafted HL7 message can exhaust memory in the import service.

How to fix:
  1. Upgrade to xmlparse 2.3.4 (semver-compatible)   [recommended · brio audit fix]

  3 other advisories affect dependencies but are not reachable from your code.
  They are recorded as "not affected" in .brio/vex.json.
```

- **Every ecosystem, one database format.** Advisories are read in the OSV format from the Brio advisory database, RustSec, the Python advisory database, the GitHub Advisory Database and CVE records for C libraries. Packages are identified by standard package URLs, for example `pkg:brio/ward-registry@2.1.0`.
- **Reachability, not just presence.** Brio already has the full call graph, including across the Rust and Python bridges, so `brio audit` reports whether your code can actually reach the vulnerable function, like Go's `govulncheck`. This removes most false alarms.
- **VEX statements.** For each advisory, the audit writes a machine-readable "affected / not affected" statement with its reason, in OpenVEX or CSAF format, so customers and regulators can see why an advisory does not apply.
- **Offline and hermetic.** Advisory databases can be mirrored, so audits work in air-gapped build environments and give reproducible results.
- **Old releases too.** `brio audit --release 1.3.0` checks the bill of materials of a version already shipped, so a new advisory is matched against every release still in the field.

### Gates and waivers

`brio build` warns about reachable vulnerabilities, and `brio ship` fails on any above the configured severity. An exception needs a waiver in `audit.toml`, with a reason, an owner and an expiry date; waivers appear in the assurance ledger and expire loudly. `brio audit fix` proposes the smallest semver-compatible upgrades and shows how the ledger would change.

### Closing attack routes

| Attack route | Common in other ecosystems | Brio |
| --- | --- | --- |
| Code runs during install or build | Build scripts, install hooks, procedural macros | Never: `comptime` code, embedded languages and extensions run sandboxed, with no network access |
| Tampered or impersonated packages | Unsigned uploads, stolen tokens | Signed packages with verified build provenance, and publishing from CI with short-lived credentials |
| Silent new permissions | A dependency quietly starts using the network | *Deny by default*: code can touch nothing outside the program unless it is handed a capability, packages need explicit permission for `unsafe` and foreign code, and the operating system confines the whole process to what was granted |
| Swapped versions | Lockfile without hashes | `Brio.lock` pins content hashes for every ecosystem |
| Unreviewed code | Nothing records who checked what | Recorded dependency audits, shareable between teams, in the style of `cargo vet` |
| Typosquatting | Look-alike names | Registry checks for similar names, and a warning when adding a dependency similar to a popular one |

```toml
[permissions]                          # everything the program may ever do
net      = ["api.hospital.local:443"]
fs.read  = ["/etc/pump"]
fs.write = ["/var/log/pump"]

[permissions.packages]                 # denied unless listed
pump-hal = ["unsafe", "ffi", "device"]
xmlparse = ["ffi"]                     # a Rust crate: no net, fs or spawn
```

### No ambient authority

Permissions in the manifest are necessary but not sufficient. A package granted network access for one purpose could otherwise use it for any other, and the compiler cannot see inside Rust, C or Python code. Brio therefore enforces permissions in three layers:

1. **Capabilities in the type system.** There is no global `open_file` or `connect`. Every operation on files, network, processes, environment, clock, randomness, standard I/O or devices needs a capability value, and only `main` receives them. A function that was not handed the network capability cannot reach the network, and the compiler proves it. Capabilities can be narrowed but never widened, so a library can be given exactly one host or one directory.
2. **Package permissions.** `unsafe` code and foreign code can bypass the type system, so each package needs explicit permission for them, and a foreign dependency must declare which effects it needs. Everything is denied unless listed.
3. **Operating-system confinement.** At startup the runtime confines the process to exactly the granted permissions, using the platform's sandbox (Landlock and seccomp on Linux, the App Sandbox on macOS, AppContainer on Windows). Even foreign and `unsafe` code cannot exceed them. Code that is not trusted at all can run as a WebAssembly component that sees only the capabilities passed to it.

The exceptions are few and named. Telemetry and logging use a `Telemetry` capability that every package receives by default but that a manifest can deny per package, and it cannot carry secret, sensitive or untrusted data. Python runs out of process unless explicitly allowed in. OS confinement is the second line of defence, not the first: it uses allow lists, blocks interfaces that bypass system-call filtering such as `io_uring`, and a critical program refuses to start where it cannot get the confinement it declared.

Passing capabilities by hand would clutter every call, so Brio has two small pieces of sugar. A `uses` clause in the signature states what a function needs, which keeps authority visible in the signature, in interface files and in review. Call sites then pass it implicitly from what the caller holds, and a `with` block narrows a capability for a region of code:

```brio
fn fetch_vitals(id: PatientId) -> Vitals ! NetError
    uses net
{
    let body = try http.get("https://api.hospital.local/vitals/{id}")   // net passed implicitly
    return try Vitals.decode(body)
}

fn main() -> () ! AnyError {
    with net.only(["api.hospital.local:443"])
    with fs.subdir("/var/log/pump")
    {
        try monitor.run()        // monitor.run declares: uses net, fs
    }
}
```

Each `with` line limits one capability for the block that follows, and several lines stack, applying from top to bottom. `with net.only(…)` is shorthand for `with net = net.only(…)`; it is allowed only for functions that limit or combine the same capability (those marked `@subset` or `@join`), so it can never swap in a different capability. Because `only` takes a list, the common case needs no combining at all; when two separately limited capabilities must be combined, `.join(…)` says so explicitly. The long form `with name = expression` remains available for everything else.

Any number of `with` lines can be stacked: each can only keep or reduce authority, so a long stack never adds up to more power, and stacking the same capability twice limits it further. For readability, the lint `long-with-stack` warns after six lines (configurable in `[lints]`) and suggests a capability set or a helper function instead. Every Brio compiler accepts at least 64 stacked lines.

**Implementation limits.** That last guarantee is one row of a table of minimum limits in the normative spec (Annex H), covering line length, nesting depth, parameters, generic depth, enum variants, dependency counts and more. Every conforming compiler accepts every program within them, so two compilers never disagree about a program that stays inside the table. A program beyond a compiler's actual limits gets a clear diagnostic naming the limit, never a crash or wrong code. Limits are enforced by counting rather than by running out of stack, so whether a program compiles never depends on the machine, and the toolchain warns when a program relies on a limit above the minimum.

This is sugar only: the compiler checks every flow exactly as if each capability were passed by hand, and `brio caps` shows the implicit flows too. A trait implementation cannot use more capabilities than its trait declares, and adding a `uses` item to a public function is a breaking change, so new authority never slips in unnoticed.

The same design pays off elsewhere. Because every external effect passes through a capability, deterministic simulation and fault injection simply substitute simulated capabilities. `brio caps` shows exactly which functions can reach the network or the disk, and how the capability got there. A program with no `[permissions]` table can do nothing outside its own memory. In scripts, top-level code uses its capabilities directly, so scripts stay as short as before, while functions inside a script still receive capabilities explicitly.

**When the operating system cannot confine the process.** Some targets, such as most microcontrollers, have no sandbox, and a host kernel may lack the needed feature at runtime. Brio never hides this:

| Situation | Result |
| --- | --- |
| Building for a target without confinement | Compiler warning naming the target |
| The same, with `--deny-warnings` or `warnings = "deny"` | Error |
| The same, in a program with `critical` modules | Error, unless built with `--allow-unconfined "reason"`; the reason goes into the assurance ledger and the release report |
| Confinement unavailable or failing when the program starts | Warning event in telemetry and on standard error |
| The same, in a `critical` program built as confined | The program refuses to start, unless the build acknowledged it as above |

### Your own capabilities

Teams can define capabilities for their own domain, such as access to patient records, permission to change an infusion rate, or administrator actions. They follow exactly the same rules as the built-in ones; in fact `Dir` and `Net` are declared the same way in the core library.

```brio
/// Access to patient records, optionally limited to one ward.
public capability PatientRecords as records {
    store: Dir
    ward:  WardId?                       // .None means every ward
}

impl PatientRecords {
    /// True if this capability covers ward `w`: it has no ward limit, or its limit is `w`.
    @pure
    public fn covers(self, w: WardId) -> Bool =
        self.ward == .None or self.ward == .Some(w)

    @pure
    public fn allows(self, a: RecordAction) -> Bool = match a {
        .Read(w) | .Write(w) => self.covers(w)
    }

    @grant                               // called once by the runtime at startup, only if the manifest grants it
    fn from_manifest(fs: Dir) -> Self = PatientRecords(store: fs.subdir("/srv/records"), ward: .None)

    @subset                              // proven never to allow more than `self`
    public fn only_ward(self, w: WardId) -> Self
        requires self.covers(w)
    = PatientRecords(store: self.store.clone(), ward: .Some(w))

    public fn read(self, id: PatientId) -> Record ! RecordError
        requires self.allows(.Read(ward_of(id)))
    { … }
}

fn discharge_summary(id: PatientId) -> Summary ! RecordError
    uses records
{
    return Summary.from(try records.read(id))
}
```

`@grant` marks the one function the runtime calls at startup to create the first `PatientRecords`, and only if the manifest grants it; it is not an initializer, and no code can call it. `@subset` marks functions that return a more limited copy of the capability: the compiler proves that its permissions are a subset of the original's. Its counterpart, @join, marks functions that combine two capabilities the holder already has, proven to allow nothing beyond the two together.

What makes this sound:

- **Denied by default.** A capability can come into existence in only four ways: from the runtime at startup, and only if the manifest grants it (`custom = ["hospital.records.PatientRecords"]` in `[permissions]`); built from other capabilities the caller already holds; by narrowing an existing one; or in tests.
- **Unforgeable.** Only the capability's own module can construct it. Capabilities cannot be stored in globals or constants, created at compile time, decoded from data or copied implicitly, so authority only moves along paths `brio caps` can show.
- **Narrowing is proven, not trusted.** Each capability states its policy as an `allows` function. For every narrowing function marked `@subset`, the compiler proves that the result allows nothing the original did not. Without the precondition of `only_ward` above, narrowing a ward-3 capability to ward 5 would widen it; the proof fails and the compiler shows that counterexample.
- **Same ergonomics.** `uses records` and `with records.only_ward(w) { … }` work exactly as for built-in capabilities.
- **Delegation and revocation.** A capability is cloned only if it opts in. `Revocable[C]` hands out a capability that can be withdrawn later, and its borrow cannot escape.
- **Visible and testable.** The ledger lists every custom capability, whether it is granted, and its proof status. Because custom capabilities are built on built-in ones, simulation and fault injection cover them automatically.

### Several capabilities at once

```brio
capability set Reporting = net, records, clock

fn nightly_report(ward: WardId) -> () ! ReportError
    uses Reporting
{
    let rows = try records.list(ward)
    try upload(rows, stamped: clock.now())        // upload uses net
}

fn main() -> () ! AnyError {
    let icu = 3 as WardId
    with net.only(["reports.hospital.local:443", "backup.hospital.local:443"])
    with records.only_ward(icu)
    {
        try nightly_report(icu)   // receives exactly: records for ward 3, two hosts, the clock
    }
}
```

| Question | Answer |
| --- | --- |
| Can a function use several? | Yes: list them in `uses`, including several of one type under different names (`uses src: Dir, dst: Dir`) |
| Which one does a call receive? | The one with the matching name and type, never a guess. Reordering items or adding capabilities to scope never changes what an existing call receives |
| How do I avoid repeating long lists? | A `capability set` names a group, and `uses Reporting` means exactly its items |
| Can I combine two I hold? | Yes, with `join`. The compiler proves the result allows nothing beyond the two inputs together, just as it proves narrowing never widens |
| An operation needs two at once? | It takes both. A custom capability can also hold several, with one policy for the combined actions |
| What about tasks and closures? | Capabilities sent to tasks must be `Send`; closures capture the capabilities they use, and `brio caps` shows it |
| Can I revoke several together? | Yes: a `RevocationGroup` shares one revoke handle |
| What if I ask for too much? | The compiler warns about unused capabilities and suggests narrower ones, such as `read_only()`; `brio caps --suggest` lists them all |

### Publishing your own advisories

`brio advisory new` drafts an OSV advisory for a vulnerability in your own package, with the affected versions and functions filled in from the interface history. `brio ship` includes the vulnerability status and VEX statements for the release, alongside the software bill of materials, which supports reporting under the Cyber Resilience Act and FDA premarket cybersecurity submissions.

## Developer experience

The goal is developers who ship with confidence because the tools have already told them what is true about their code.

### The compiler's voice

Brio's diagnostics learn from Elm, which showed that a compiler can explain rather than accuse, but they follow a style of their own. Every diagnostic answers three questions, always in the same order: *what* happened, *why it matters*, and *how to fix it*. The "why it matters" part is specific to Brio: it states the consequence for the running system, because in safety-critical code the consequence is what makes a reviewer care.

```text
── THESE UNITS DON'T MATCH ─────────────────── src/pump.brio:18 · BU0102

`set_rate` needs a rate in ml/h, but `drip_rate` is in ml/min:

18 │     pump.set_rate(to: drip_rate)
   │                       ━━━━━━━━━ F64<ml/min>

   expected  F64<ml/h>
   found     F64<ml/min>
                   ━━━━━━ only the unit differs

Why it matters: passed unchanged, the pump would run 60 times too slowly.

How to fix:
  1. Convert the unit                         [recommended · brio fix applies it]
       pump.set_rate(to: drip_rate as F64<ml/h>)

brio explain BU0102
```

```text
── PRECONDITION NOT PROVEN ─────────────────── src/dose.brio:27 · BP1004

`dose_for` requires `weight > 0.0<kg>`, and nothing here guarantees it:

27 │     let d = dose_for(patient.weight)
   │                      ━━━━━━━━━━━━━━

Counterexample: patient.weight = 0.0<kg>

Why it matters: with a zero weight, the dose calculation divides by zero.

How to fix, pick one:
  1. Make zero weights impossible in the type    [recommended]
       type Weight = F64<kg> in 0.5<kg> ..= 500.0<kg>
  2. Handle the missing weight before dosing     [brio fix applies it]
       if patient.weight <= 0.0<kg> { fail .WeightMissing }

brio explain BP1004
```

The rules behind this style:

- **The headline names the problem in the programmer's words.** "These units don't match", not "mismatched types".
- **One root cause, one report.** Errors caused by an earlier error are suppressed, and errors are ordered so the first cause comes first.
- **Types are shown as they are written, and compared.** When two types differ, the diagnostic marks the part that differs.
- **Counterexamples, not proof jargon.** A failed proof shows concrete values that break the contract.
- **Fixes are ranked.** The recommended fix comes first. A fix is labelled machine-applicable only when it is certain to be correct.
- **No compiler jargon.** Words such as *lifetime*, *monomorphization*, *unification* and *trait object* never appear. Diagnostics use the vocabulary of this spec: borrow, origin, mode, contract.
- **Brief by default.** At most three short paragraphs of prose. `brio explain` gives the full story, with examples.
- **Identical everywhere.** The terminal, the editor and the JSON output carry the same content.

### The build guarantee

A successful build ends with a summary of what it guarantees and what it takes on trust. The full record is written to `.brio/assurance.json`, and `brio ledger` shows it in detail.

```text
✓ Built monitor 1.4.0 (release) in 3.2 s

  Guaranteed          memory safety · no data races · no undefined behavior
                      every error handled · every match exhaustive
  Proven (critical)   412 obligations in 3 modules: no overflow, no out-of-bounds,
                      all contracts hold, all loops terminate, stack ≤ 18 KiB
  Checked at runtime  1,208 contracts in 21 modules (trap on violation)
  Taken on trust      2 assumptions · 1 trusted boundary · 4 unsafe blocks
                      37 Rust crates · 3 bridge claims · 1 Python module
                      12 unbounded functions
  Details             .brio/assurance.json · brio ledger
```

This is how Brio makes "if it compiles, it works" as true as it can be, and honest where it cannot be. No compiler can know whether the contracts describe what the users actually need, whether foreign code keeps its promises, or whether the hardware works. Brio does not hide those gaps: it lists every one of them, so nothing is taken on trust silently. The guarantee is: **everything the compiler did not prove or check is in the ledger, by name and with a reason.**

The same principle covers time. Code that compiles under an edition keeps compiling with every future Brio release. Changes that would break code arrive only in a new edition, with an automatic migration.

### Trusting the compiler itself

The build guarantee is only as good as the compiler that gives it. No compiler can be proven free of every bug, so Brio makes trust in the toolchain rest on independent evidence rather than on faith: every stage is checked by something other than itself.

| Risk | What checks it |
| --- | --- |
| The language rules themselves are unsound | Machine-checked proofs of the core language: type soundness, borrows, termination, capabilities, and the verifier |
| The prover wrongly says "proven" | Critical proofs, including termination and capability proofs, carry proof certificates validated by an independent checker; safety obligations are proven without any assumed fact, and vacuous proofs are reported. Agreement between solvers alone is not accepted, because solvers have had hundreds of soundness bugs |
| The optimizer or a code generator miscompiles | `brio test --cross-check` runs every test compiled by both code generators and under the interpreter, and flags any difference. Critical modules additionally get a proven compilation path or per-build translation validation |
| The compiler crashes or corrupts its own data | It validates its intermediate forms between phases; any internal error leaves no build artifact behind and writes a reproduction bundle |
| Incremental or parallel builds differ from clean ones | `brio build --verify-incremental` rebuilds from scratch and compares, on every release |
| A toolchain upgrade changes behavior silently | `brio upgrade-check` builds with both versions and compares tests, diagnostics, proofs and ledgers before you adopt it |
| Code behaves differently on another platform | Warnings wherever the program depends on implementation-defined behavior; errors with `--strict` |
| The compiler binary itself is compromised | Signed, reproducible releases with build provenance, built by a Rust toolchain that can itself be rebuilt from a full-source bootstrap, so a compiler that hides code in itself would be detected |
| A known toolchain bug affects your code | `brio audit` includes the toolchain, and `brio ship` checks your program against its published known-problems list |

What remains trusted is small and named: every build's ledger lists its *trusted base*, meaning the compiler parts not yet covered by proofs or validation, the proof checker, the linker, the operating system and the hardware. Shrinking that list is a goal of every release. Together with the assurance ledger, this means you can work at the edges of the language knowing that any failure, in your code or in the compiler, is caught by an independent check rather than discovered in production.

### Fearless at the edges

Rust's promise is *fearless concurrency*: if it compiles, there are no data races and no memory errors. Brio keeps that promise and closes the gaps Rust leaves, so that multithreaded, async and low-level code can be pushed hard without surprises in production.

| Failure in production | Rust | Brio |
| --- | --- | --- |
| Data races, use-after-free, null | Prevented | Prevented |
| Ignored errors | Warning | Error: a result cannot be dropped silently |
| Integer overflow | Wraps silently in release builds | Checked or proven in every build |
| Deadlock on locks | Possible | Prevented: locks have levels, must be taken in increasing order, and leaf locks cannot nest; the compiler checks this across calls |
| Lock held across `await` | Possible, a common source of deadlocks | Error; `AsyncMutex` exists for that purpose |
| Blocking call inside async code | Freezes the executor silently | Error, unless wrapped in `blocking { … }` |
| Subtle atomic ordering | Allowed silently | Weaker orderings need a stated reason, recorded in the ledger; interleaving tests explore schedules exhaustively |
| Hanging forever | Invisible | Only in functions declared `unbounded`; waits need deadlines otherwise |
| Cancellation leaving state half-updated | Possible when a future is dropped | Cancellation is an ordinary error the task handles |
| `unwrap()` crash | One keystroke | No reasonless trap exists; `expect(reason)` is listed as a trap site |
| Memory leaks through reference cycles | Possible | Reported by every test run, with the allocation site |
| A dependency doing something unexpected | Possible | Deny-by-default capabilities and OS confinement |

**The trap map.** A trap stops the program on purpose, because continuing with broken assumptions is worse. For code that must not stop, `brio ledger --traps` lists every place where a trap can still happen in production: unproven checks, `expect` calls, unbounded waits, infallible allocations and foreign calls. Each entry comes with the change that removes it, such as a proof, `get` instead of indexing, or a `try_` operation. A team can drive the list to zero for a module and have `brio ship` keep it there. `critical` code is already required to have an empty list.

**Isolation for availability.** When one subsystem must not take down the rest, run it as a WebAssembly component: a trap inside it ends only that component, and the host carries on.

### What v0.1 hardened

Before v0.1, the design was put through an adversarial review against every documented class of failure in comparable languages, compilers, package ecosystems and cryptographic schemes. The main changes, all reflected in the normative spec:

| Area | Weakness found | Fix in v0.1 |
| --- | --- | --- |
| Optimizer facts | Range and non-aliasing facts could be violated by C, Rust, Python or device memory, turning bad input into undefined behavior | The optimizer receives only proven facts or values checked at a trust boundary; a machine-checked aliasing model backs every non-aliasing fact; hardened critical builds re-check memory a hardware fault could corrupt |
| Type system | Rust's longest-standing soundness bug comes from implied bounds on lifetimes | Brio has no implied bounds and no specialization, and parameter passing is copy-only for types without interior mutability |
| Proofs | Two solvers agreeing is not proof; solvers have had hundreds of soundness bugs | Critical proofs need checked certificates; safety obligations are proven without assumed facts; vacuous proofs are reported |
| Termination | Termination checkers fail at hidden calls | The call graph includes drops, operators, iterators, wakers and foreign entry points; critical code gets termination certificates |
| Traps | A trap handler running arbitrary code is an attack surface | The handler runs on a reserved stack, cannot allocate or lock, and holds only the capabilities registered for it, such as a safe-state output |
| Python | In-process Python could bypass every capability | Python runs in a confined worker process by default |
| Editors and assistants | Opening a repository ran its compile-time code; assistant tools can be poisoned | Untrusted workspaces get syntax services only; the assistant server never returns secrets or decrypted licensed code, and its tool descriptions are signed |
| Caches and packages | Any writer could poison a shared cache; binaries could differ from source | Only trusted builders write; cache keys enumerate every input; registries rebuild open packages independently before use |
| Sign-in | Device-code sign-in is being phished at scale | Browser sign-in with PKCE by default; tokens bound to the machine; email codes cannot change sign-in methods |
| Licensed packages | Standard AES-GCM lets one ciphertext decrypt to different code under different keys | Key-committing encryption, signed key wraps, and a custodian that uses an HSM or enclave where available |

Some earlier claims were also corrected rather than kept. Brio does not claim its borrow rules are "equivalent to Rust's", since Rust's have known holes; it claims soundness against its own machine-checked core. It does not claim a verified toolchain; it claims independent checks with a named trusted base. And it no longer says "no ambient authority" without naming the exceptions.

### Editor

The compiler is the language server, so the editor always agrees with the build. It adds an ownership lens showing borrows, moves and drops inline; a contract gutter marking each `requires` and `ensures` as proven, runtime-checked or failing; and units and ranges on hover.

### Support in every IDE

The language server is only one of the pieces editors need. Brio supplies all of them from **one binary with one analysis engine**, so every editor sees the same diagnostics and behavior, and supporting a new editor only takes a small plugin.

| Piece | Command or artifact | What it gives editors |
| --- | --- | --- |
| Language server | `brio lsp` | Diagnostics, completion, navigation, refactoring, formatting, embedded-language services |
| Debug adapter | `brio debug` (Debug Adapter Protocol) | Breakpoints, stepping and inspection in any editor that supports the protocol |
| Debugger helpers | LLDB and GDB formatters shipped with the toolchain | Readable `List`, `T?`, unit and range values; breakpoints and stepping inside any embedded language, through its source maps |
| Build server | `brio bsp` (Build Server Protocol) | Project import, run configurations and test targets, notably in JetBrains IDEs |
| Test discovery | Language-server extension and `brio test --list --format json` | Test explorers with run and debug buttons for tests, property tests and benchmarks |
| Tree-sitter grammar | `tree-sitter-brio`, with injections | Highlighting and folding in Neovim, Helix and Zed, plus GitHub and diff tools; every embedded language highlighted inside its literals, using the grammar its package ships |
| TextMate grammar | `brio.tmLanguage.json` | Base highlighting in VS Code, Sublime and JetBrains bundles, and on GitHub through Linguist |
| Code-navigation index | `brio index` (SCIP) | Go-to-definition in GitHub and Sourcegraph code review, without a compiler |
| Report formats | SARIF, JUnit XML, LCOV and Cobertura | Code-scanning results, test results and coverage in CI and review tools |
| Browser build | Language server compiled to WebAssembly | A playground, and web editors such as vscode.dev and github.dev |
| AI assistants | `brio mcp` (Model Context Protocol) | Assistants query real types, diagnostics, proof results and the ledger instead of guessing |

**Editor packages** stay thin. Each one only starts `brio lsp` and `brio debug` and renders Brio's custom messages:

- a VS Code extension, published to both the Visual Studio Marketplace and Open VSX;
- a JetBrains plugin, using JetBrains' own language-server support or the LSP4IJ plugin;
- a Zed extension;
- ready-made configurations for Neovim, Emacs (Eglot), Helix and Sublime LSP.

Brio features that the standard protocols can't express, such as proof status, the assurance ledger and flag usage, use documented protocol extensions. Editors that don't know them ignore them and keep working.

**One installer.** `brioup` installs the compiler, language server, debug adapter and debugger helpers as one matched set. Each project pins its toolchain version in `brio-toolchain.toml`, so the editor, the terminal and CI always run the same version.

### Completion you can trust

Completion in Brio is sound, stable and fast, for Brio code, every dependency and every embedded language.

- **Every suggestion compiles.** A suggestion is offered only if inserting it causes no new name, type, unit, mode, profile or capability error. Accepting it also inserts what Brio requires: argument labels, `mut` or `move` markers from the parameter modes, `try` or `await`, and the `use` line if the item is not yet imported.
- **Suggestions follow types.** When the position expects a `MlPerHour`, values of that type come first, and literals are suggested with their unit. Items that are `unsafe`, `unbounded` or deprecated are marked; items the profile forbids, such as allocation in `critical` code, are not offered.
- **It generates structure, not just names.** All missing `match` arms, all missing trait members with their contracts, record fields with required ones first, and contract terms such as `result` and `old`.
- **Embedded languages too.** Inside `#db"…"`, the SQL language suggests tables and columns from the migrations. Inside `${…}`, Brio suggests values of the column's type, because languages report the type each hole expects.
- **Every dependency, even without source.** Completion for dependencies comes from their interface files, so private-source packages complete as well as open ones. Rust and C items come through bridges; Python items come from type stubs where available and are labelled unverified otherwise.
- **AI suggestions are checked first.** Inline suggestions from AI assistants are compiled before they are shown, and are labelled as not coming from the compiler.
- **Stable ordering.** The same code and cursor always give the same suggestions in the same order, and the list does not reshuffle while you type.
- **Fast, even in broken code.** The parser recovers from any error, and completion never waits for proofs, code generation or other functions' bodies. The target is 50 ms to first results at the 95th percentile in a million-line workspace, with a hard limit of 200 ms; slow embedded-language services are cut off rather than waited for. Latency is measured in CI.

### Fast feedback

The target is under one second from saving a file to running code when only a function body changed. `brio dev` provides watch mode and hot reloading of function bodies in development builds, which pairs with Tauri's frontend hot reload.

### Testing built in

Contracts and range types become test generators automatically:

```brio
type HeartRate = U16 in 20 ..= 300

fn classify(hr: HeartRate) -> Alarm
    ensures hr < 40 implies result == .Low(hr)

test "classification" {
    assert classify(35) == .Low(35)
}
// `brio test` also generates thousands of HeartRate values in 20..=300,
// checks every `ensures`, and shrinks any failure to a minimal case
```

| Tool | Purpose |
| --- | --- |
| Property tests | Generated from contracts and range types |
| `brio fuzz` | Coverage-guided fuzzing |
| Snapshot tests | Compare output against approved snapshots |
| `brio mutate` | Mutation testing: do the tests actually catch bugs? |
| MC/DC coverage | Branch coverage required for DO-178C Level A |
| Deterministic scheduler | Seeded task scheduling so race conditions replay exactly |

### Release readiness: `brio ship`

`brio ship` produces a release report and can block the release. It covers proof status, test and property-test results, mutation score, MC/DC coverage for `critical` modules, every `unsafe` block with its justification, interface changes with the required semantic-version bump, bridge hot-path crossings, performance against the baseline, and supply chain (SBOM, reproducible build, security advisories).

### Other tooling

- **One tool.** `brio` handles building, testing, formatting, documentation, packages and publishing, including to a private registry.
- **One formatting style.** The formatter has no configuration, like gofmt.
- **`brio perf`.** Explains why code is slow: allocations, remaining bounds checks, dictionary-dispatch fallbacks and inlining decisions, with annotated assembly.
- **Native debugger support.** Units appear in variable values, contracts can be evaluated at breakpoints, and recorded failing tests can be stepped backward.

## Scripting mode

Brio also runs as a scripting language, so existing Python scripts can move to Brio one at a time. A script is an ordinary `.brio` file with statements at the top level. There is no separate dialect: the type system, borrow checker, contracts and traps are the same as in packages, so a script can later become a module of a package without being rewritten.

### What a script looks like

```brio
#!/usr/bin/env brio
---
[dependencies]
ward-registry = "2"    # a Brio package from a private registry
---
use std.{args, csv}

@derive(csv.Decode)
record Reading { patient: String, rate: U16 }

@derive(args.Parse)
record Options { input: Path, threshold: U16 = 120 }

let opts = try args.parse[Options]()
var alarms = 0
for row in try csv.read[Reading](opts.input) {
    let r = try row
    if r.rate > opts.threshold {
        print("{r.patient} ({ward_registry.ward_of(r.patient)}): {r.rate}")
        alarms += 1
    }
}
print("{alarms} readings above {opts.threshold}")
```

The script runs with `./check_rates.brio --input today.csv` or `brio run check_rates.brio -- --input today.csv`. The optional block between `---` lines is an inline manifest, so a script needs no `Brio.toml`. Top-level statements form an implicit `main` that returns `() ! AnyError`, so `try` works at the top level and any error is printed with its cause chain before the script exits with status 1.

### How scripts run

| Aspect | Behavior |
| --- | --- |
| Execution | Compiled to native code with the fast development back end (TPDE or Cranelift), not interpreted |
| Start-up | The compiled script is cached by the hash of its source, dependencies and toolchain; an unchanged script starts without recompiling. Targets: under 1 s for a first run of a typical script, under 20 ms from cache |
| Dependencies | Declared in the inline manifest; `brio script lock` pins them in a `check_rates.brio.lock` file next to the script |
| Errors | Any error type converts to `AnyError`; `.context("reading input")` adds a message, like Python tracebacks but without exceptions |
| Safety | Same checks as packages: types, borrows, contracts in check mode, overflow and bounds traps. The `critical` profile is not allowed in scripts |
| Interactive use | `brio repl` and a Jupyter kernel, both built on the same compiler |
| Growing up | `brio script promote check_rates.brio` turns a script into a package, moving the top-level statements into `main` |

### Batteries for scripting

The standard library includes the modules scripts need most, so common scripts need no dependencies at all:

| Python | Brio |
| --- | --- |
| `argparse` | `std.args`, with `@derive(args.Parse)` on a record |
| `pathlib`, `os`, `shutil` | `std.path`, `std.fs` |
| `subprocess` | `std.process`, with `run("git", ["status"])`; arguments are passed as a list, never through a shell, so there is no shell injection |
| `json`, `csv` | `std.json`, `std.csv`, with derived decoders for records |
| `re` | `std.regex` |
| `requests` | `std.http` |
| `logging`, `datetime` | `std.log`, `std.time` |
| `pandas` | The Rust `polars` crate, through the normal Rust bridge |

### Moving Python scripts to Brio

Python interop lets the migration happen gradually instead of all at once:

- **Brio calling Python.** `use python pandas` imports a Python module. Python values have the dynamic type `PyObject`, and every Python call returns `T ! PyError`, so Python failures are ordinary errors. Because Python can do anything its process can do, it runs by default in a separate, confined worker process that receives only the capabilities passed to it; running it inside the program's own process must be requested explicitly and is recorded in the ledger as unrestricted authority. This lets a script port its logic while still using a Python-only library.
- **Python calling Brio.** `@python.export` on a public function builds a Python extension module, so existing Python scripts can call Brio code. Hot loops or safety-relevant logic can move first, with the rest of the script unchanged.
- **Translation.** `brio migrate --from python` translates type-annotated Python (PEP 484 hints) into draft Brio. Where types are missing, it leaves typed holes that the compiler points to one by one.

The recommended order is: scripts with no third-party libraries first; then scripts whose libraries have Brio or Rust equivalents; last, scripts that depend on Python-only libraries, which keep those libraries through `use python` until a replacement exists. Python interop is available only in the `standard` profile, and every use appears in the `brio ship` report.

## Migration strategy

Migration happens one module at a time, and a module moves to Brio only when tests show it behaves identically to the Rust original and runs as fast.

### Migration tools

- **`brio migrate`.** Converts a Rust file to idiomatic Brio, mapping `&`/`&mut` to parameter modes and `Result` to `!`. Anything it cannot convert confidently is left as a marked TODO.
- **`brio equiv`.** Runs the Rust original and the Brio port side by side on generated inputs and reports any difference in output.
- **Benchmark comparison.** The same harness compares the port's speed against the Rust original.

### Order of migration

1. **Leaf modules with pure logic first.** Calculations, parsers and validation code with few dependencies prove the toolchain with the least risk.
2. **Shared internal libraries next.** Once converted, these deliver the biggest build-time gains, since interface-hash rebuilds stop changes from rippling through the tree.
3. **Application layers last.** Tauri shells and anything heavily tied to generic Rust APIs stay in Rust longest and call into Brio through bridges.

A module is merged only when three gates pass: `brio equiv` reports no differences, benchmarks are within 3% of the Rust original, and `brio ship` reports no blocking issues.

### Production use before qualification

Brio is meant for production use limited to well-tested areas that are not life-critical. The `critical` profile is available from the start so teams build the discipline early, but `critical` builds are labelled unqualified. No life-critical function moves to Brio until the toolchain is qualified under the relevant standard (IEC 62304, ISO 26262 or DO-178C).

## Licensing

Brio is open source. Its toolchain, standard library and runtime, documentation and specification are all public, and every licensing choice follows a mature language that has used it. This section is to be reviewed with IP counsel; it is not legal advice.

### Precedents

| Language | Toolchain and compiler | Runtime and standard library | Lesson for Brio |
| --- | --- | --- | --- |
| LLVM, Cranelift, TPDE | Apache-2.0 with LLVM exception | Same | The license of Brio's own back ends; using it avoids license friction |
| Swift | Apache-2.0 with Runtime Library Exception | Same | A runtime exception lets compiled programs ship without attribution |
| Rust | MIT or Apache-2.0, dual | Same | The norm for the Rust crates Brio interoperates with |
| Java (OpenJDK) | GPL-2.0 with Classpath Exception | Same | A copyleft compiler still works when the runtime has an exception |
| GCC | GPL-3.0 | GPL-3.0 with GCC Runtime Library Exception | Without a runtime exception, compiled programs would be affected by the compiler's license |
| Go, Dart, Zig, Kotlin | BSD-3, BSD-3, MIT, Apache-2.0 | Same | Permissive licenses are the norm for modern languages |

The common thread: **every successful language guarantees that programs compiled with it belong entirely to their authors.** Brio does the same.

### What each part is licensed under

| Part | License |
| --- | --- |
| Compiler, language server, debug adapter, installer and all other tools | Apache-2.0 with LLVM exception |
| Standard library and runtime | Apache-2.0 with LLVM exception |
| Programs compiled with Brio, and bridge code generated for them | Belong to their authors, with no obligations |
| Editor plugins | Apache-2.0 |
| Tree-sitter and TextMate grammars | MIT |
| Language specification | CC BY 4.0 |
| Documentation and website text | CC BY 4.0 |
| Code examples in the documentation | MIT-0 |
| Website code | Proprietary |

Apache-2.0 with LLVM exception is chosen for the code because it is the license of LLVM, Cranelift and TPDE, which Brio is built on. Its explicit patent grant protects users and contributors. The LLVM exception means programs that embed runtime code need no attribution notices. Every source file carries `SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception`.

### Terms that hold for every part

1. **Output.** Programs compiled with Brio, generated interface files and bridge code, and the embedded runtime belong to the user and may be distributed royalty-free without restriction.
2. **Safety disclaimer.** The toolchain is not qualified under any safety standard and must not be the sole means of assurance for safety-critical software, matching rule GEN-15 of the specification. The README and website say so prominently.
3. **Third-party components** remain under their own licenses, listed in `THIRD-PARTY-NOTICES`: LLVM, Cranelift and TPDE (Apache-2.0 with LLVM exception), the SMT solver, PyO3 and CPython (for Python interoperability), and others. No GPL, LGPL or AGPL component may enter the toolchain; the `brio ship` bill of materials checks this.
4. **Contributions** come under the Developer Certificate of Origin or a contributor license agreement. All employee and contractor agreements assign IP to the company, and no code is copied from GPL projects.
5. **Trademarks** are not licensed by the code licenses. The "Brio" name is cleared and registered, and its use follows a published trademark policy like those of Rust and Python: with the code open, the name is what protects the project.

### Before the repository is public

1. **Remove internal material.** Scan the full history for secrets and credentials, and remove client names and company-specific examples from code, docs and specifications, or rewrite them as generic examples.
2. **Add the community files:** `LICENSE`, `NOTICE`, `README`, `CONTRIBUTING`, `CODE_OF_CONDUCT`, `GOVERNANCE` and `SECURITY`, with a private process for reporting vulnerabilities. Security reports matter even more for a language used in safety-critical systems.
3. **State the safety position** in the README, as in term 2.

### Governance

Language changes go through a public proposal process (specification, Clause 29). Breaking changes happen only in new editions. The company leads the project at first; a neutral foundation can take over once there are outside maintainers.

### Sustaining the project

The proven model for safety-critical toolchains is to keep the toolchain free and open and sell what regulated users need around it: qualification kits and evidence, long-term support for pinned versions, and certification help. Ferrocene, a qualified Rust toolchain, works this way.

## Closing the gaps

A review of this design against what a production language for life-critical and general systems needs found fifteen gaps. All of them are now closed in the normative specification, which states the rules; this section summarizes the decisions. Four items can only be closed by doing work rather than by specifying it; they are listed at the end.

### Sound and Turing complete, deliberately split

No language can be Turing complete and also decide every property of every program. Brio keeps Turing completeness, but makes every use of it visible:

- **Functions are bounded by default.** The compiler accepts a function without annotation when it can see that it finishes: `for` loops over finite collections, loops with `bounded` or `decreases`, recursion on smaller parts of the data, and waits with a deadline. Most code is like this, so the termination guarantee costs nothing.
- **Everything else says so.** A function that may run without bound is declared `unbounded fn`, and calling one makes the caller unbounded too. `main`, scripts and tests are unbounded without ceremony.
- **No hidden loops.** Recursion hidden in stored callbacks or dynamic values is caught by a conservative whole-program call graph. Waits without a deadline, such as `mutex.lock()`, are unbounded. Foreign code is unbounded unless explicitly claimed otherwise, and every claim is on the ledger.
- **Contracts stay consistent.** Functions used in contracts must be bounded, because a non-terminating function could be used to "prove" anything.
- **Critical code is bounded.** `critical` modules allow `unbounded` only for a task's outermost loop, each iteration of which is bounded.
- **Builds always finish.** Trait resolution has a depth limit, like every other compile-time evaluation, so every build ends with a result or a diagnostic.

Every unbounded function appears in the assurance ledger. A reviewer can therefore see exactly where the program could hang, and a search for `unbounded` finds every such place.

Soundness rests on evidence, not on the design alone:

- every construct is translated into a small core language, *Brio Core*;
- the Core semantics is mechanized in a proof assistant, with proofs of type soundness, data-race freedom, borrow soundness and verifier soundness, in the style of RustBelt; the aliasing rules for `unsafe` code follow the tree-based model of Tree Borrows;
- `brio interp` detects undefined behavior in tests, and critical builds are tested back to back against it;
- a public conformance suite covers every rule, and each toolchain publishes a machine-readable known-problems list that `brio ship` checks automatically.

### How each gap is closed

| Gap | Decision | Spec |
| --- | --- | --- |
| Freestanding and embedded targets | Three library tiers (`core`, `alloc`, `std`); target specifications; `@entry` and `@interrupt`; `Shared[T, ceiling]` for data shared with interrupts; `Volatile[T]`; typed registers generated from device descriptions; an interrupt-driven executor | Clause 30 |
| Allocation | An `Allocator` trait and per-collection allocators; `try_` forms of every allocating operation; `@alloc(fallible)` modules; bounded collections, static pools and arenas | Clause 31 |
| Workspaces, registries, vendoring | Workspaces with one lockfile and inherited settings; any conforming registry, mirrors, `brio vendor` and fully offline builds; yanking | Clause 32 |
| Serialization and schema evolution | Derivable `Encode` and `Decode`; decoding never produces an invalid value; schema history written in source attributes and guarded by hashes in Brio.lock, with compatibility checked and migrations required for breaking changes | Clause 33 |
| Language evolution | Public proposals; preview features behind explicit gates; deprecation for a full edition before removal; editions at most every three years, each with a five-year long-term-support line | Clause 29 |
| Conformance and formal semantics | Brio Core, mechanized proofs, a conformance suite, qualification kits and a certified core subset | Clauses 27, 28 |
| Standard library | Structure and tiers fixed; item-by-item API in a companion document | Clause 38 |
| Proof back end | SMT-LIB; critical obligations proven with checked proof certificates, with safety obligations proven without assumed facts | CON-26 |
| Real-time | `@period`, `@deadline` and `@wcet`; static timing bounds on analyzable targets, and measured bounds labelled as such; schedulability analysis | Clause 34 |
| Fault tolerance | Verified state machines that always reach their safe state; `Redundant[T]`; majority voting over diverse computations; watchdogs | Clause 34 |
| Security | `Secret[T]`, zeroed on drop and compared in constant time; `@constant_time` functions; `Untrusted[T]` input; embedded-language holes always passed as data, so injection cannot happen by accident | Clause 35 |
| Numerics and SIMD | Fixed-point, decimal and big numbers with explicit rounding; correctly rounded math functions, identical on every target; portable `Simd[T, N]` | Clause 36 |
| Time, dates and text | `Duration` as a unit type; monotonic `Instant`; time-zone-aware civil types with explicit handling of ambiguous times; Unicode and localized formatting | Clause 37 |
| Documentation tests, accessible diagnostics, build timings | Doc examples run as tests; a screen-reader mode and translatable messages; `brio build --timings` | TOOL-30, DIAG-8, DIAG-9, TOOL-31 |
| Earlier open questions | Rust version pinning, ASCII identifiers for this edition, and the performance promise: never more than 3% slower than Rust with overflow checks by geometric mean or 10% on any benchmark, and "faster" claimed only per named workload (Clause 44) | Annex D |

### Work that remains

1. Register the "Brio" trademark and publish the trademark policy.
2. Write the companion documents *Brio Core* and *Brio Standard Library*.
3. Build the mechanized proofs and the conformance suite.
4. Have a toolchain release independently assessed for each safety standard in scope.

GPU computing and over-the-air updates are part of v0.1 (see "Accelerators" and "Software updates" below); nothing is deferred to a later edition.

### Soundness audit

A later audit of the specification found and closed these holes:

- **Assumptions can never cause undefined behavior.** Bounds and overflow checks are always proven or run, never assumed, and a check is removed only by proofs built from proven or checked facts (CON-27, CON-28).
- **Proven code is safe from unproven callers.** Preconditions are checked on entry for any caller that does not prove them (CON-29).
- **Foreign values are always validated.** Values entering from Rust are checked, whatever their bridge contracts say (RUST-10).
- **Interrupt data is safe on multi-core chips.** `Shared` adds a spin lock across cores, or all users must run on one core (EMB-11).
- **Behavior no longer depends on unspecified details.** Values with interior mutability are always passed by address (OWN-35), `Map` iterates in insertion order (LIB-6), and contract evaluation is fully defined (CON-30, CON-31).
- **Cancellation is predictable.** It arrives as an ordinary `Cancelled` error, and in-flight work is never dropped mid-wait (CONC-15). Operating-system signals arrive as events (CONC-19).

## Open questions and next steps

### Open questions

- [x] Trait implementation syntax: `impl`, as specified in Clause 12 of the specification.
- [x] Proof back end: SMT-LIB, with checked proof certificates for critical code (CON-26).
- [x] Integer overflow: always checked or proven, in every build (CON-27).
- [x] rustdoc output and LLVM pinning: each Brio release pins one rustc version whose LLVM matches Brio's (specification, Annex D).
- [ ] Name checks: trademarks, the brio-lang domain, and package registry names.

### Next steps

- [x] Write a formal grammar covering the syntax (specification, Annex A).
- [ ] Prototype the bridge generator against one real application crate, then Tauri.
- [ ] Build a minimal compiler front end in Rust, with the Cranelift and TPDE back ends, for one leaf module, and measure both against LLVM on the benchmark suite. Build Brio Lens alongside it, so that the very first phase already emits its stage records.
- [ ] Set up the Rust-versus-Brio benchmark harness before any migration starts.
- [ ] Pick the first production module to migrate and define its equivalence tests.

### Prior art to study

- **Carbon**: bidirectional interop with an existing language, the closest analogue to Brio's migration path.
- **Swift library evolution**: private-source libraries with resilient generics.
- **SPARK and Ada**: contracts, proof and safety profiles.
- **Hylo**: mutable value semantics.
- **Zig**: `comptime` and explicit allocators.
- **Ferrocene**: what qualifying a Rust-family toolchain involved.
