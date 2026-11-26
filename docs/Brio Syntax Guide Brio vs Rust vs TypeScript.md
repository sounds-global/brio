# Brio Syntax Guide: Brio vs Rust vs TypeScript

Oct 6, 2026 · Angelo Fornerino

## How to read this guide

Brio reads like a blend of Rust's safety, Swift's clarity and TypeScript's familiarity: every example below shows the same idea in Brio, Rust and TypeScript, so developers can map what they know and stakeholders can see what changes. Short constructs are compared in tables; longer ones are shown as three code blocks in the same order. Where TypeScript or Rust has no direct equivalent, the closest common pattern is shown and marked as such.

Brio's syntax follows four rules: words over symbols where meaning matters (`and`, `or`, `not`, `try`), intent written at the call site (`mut`, `move`, `out`), no hidden costs or hidden authority (capabilities, explicit conversions), and safety guarantees that the compiler proves rather than trusts. This guide follows the normative Brio Language Specification, version 0.1, which governs where they differ.

## Basics

Brio separates statements by newlines, like TypeScript without semicolons; bindings are immutable by default, like Rust; and a name cannot be shadowed within one function, so a name always means one thing.

| Concept | Brio | Rust | TypeScript |
| --- | --- | --- | --- |
| Line comment | `// note` | `// note` | `// note` |
| Documentation comment | `/// Returns the rate.` | `/// Returns the rate.` | `/** Returns the rate. */` |
| Immutable binding | `let rate = 12` | `let rate = 12;` | `const rate = 12;` |
| Mutable binding | `var count = 0` | `let mut count = 0;` | `let count = 0;` |
| Typed binding | `let rate: I32 = 12` | `let rate: i32 = 12;` | `const rate: number = 12;` |
| Compile-time constant | `const MAX_RATE: I32 = 1_200` | `const MAX_RATE: i32 = 1_200;` | `const MAX_RATE = 1_200;` |
| Global variable | `static var hits: U64 = 0` | `static mut HITS: u64 = 0;` (unsafe) | `let hits = 0;` (module scope) |
| Re-declaring a name | Not allowed in one function | `let x = x + 1;` (shadowing) | Not allowed in one scope |
| Assertion | `assert rate > 0, "rate must be positive"` | `assert!(rate > 0, "…");` | `console.assert(rate > 0, "…");` |
| Statement separator | Newline (`;` optional) | `;` | `;` (optional) |

```brio
// Brio
let patient = PatientId(42)
var retries = 0
retries += 1
```

```rust
// Rust
let patient = PatientId(42);
let mut retries = 0;
retries += 1;
```

```typescript
// TypeScript
const patient = new PatientId(42);
let retries = 0;
retries += 1;
```

## Types and literals

Brio's types are Rust's types with clearer names and three additions Rust and TypeScript lack: units of measure, range types and checked conversions. `Int` and `UInt` are 64 bits on every platform, and no number converts to another type implicitly.

| Concept | Brio | Rust | TypeScript |
| --- | --- | --- | --- |
| Integers | `I8` … `I128`, `U8` … `U128`, `Int`, `UInt` | `i8` … `i128`, `u8` … `u128` | `number`, `bigint` |
| Floating point | `F32`, `F64` | `f32`, `f64` | `number` |
| Boolean, character | `Bool`, `Char` | `bool`, `char` | `boolean`, `string` |
| String slice, owned string | `Str`, `String` | `&str`, `String` | `string` |
| String interpolation | `"Rate {rate} ml/h"` | `format!("Rate {rate} ml/h")` | `` `Rate ${rate} ml/h` `` |
| Formatting | `"{dose:.2}"` | `format!("{dose:.2}")` | `dose.toFixed(2)` |
| Multi-line, raw string | `"""` … `"""`, `r"C:\path"` | `r#"…"#` | `` `…` `` |
| Unit of measure | `let rate = 120.0<ml/h>` | No equivalent (newtype per unit) | No equivalent |
| Declaring units | `unit ml, h` | No equivalent | No equivalent |
| Range type | `type Percent = I32 in 0..=100` | No equivalent (validated newtype) | No equivalent |
| Ranges | `0..<10`, `0..=10` | `0..10`, `0..=10` | No literal syntax |
| Optional value | `I32?`, `.Some(5)`, `.None` | `Option<i32>`, `Some(5)`, `None` | `number \| undefined` |
| Default for missing | `limit ?? 100` | `limit.unwrap_or(100)` | `limit ?? 100` |
| Optional chaining | `patient?.ward` | `patient.as_ref().map(\|p\| p.ward)` | `patient?.ward` |
| Tuple | `(I32, String)`, `pair.0` | `(i32, String)`, `pair.0` | `[number, string]`, `pair[0]` |
| Fixed array, slice | `[U8; 64]`, `[U8]` | `[u8; 64]`, `&[u8]` | `Uint8Array` |
| List, map, set | `List[T]`, `Map[K, V]`, `Set[T]` | `Vec<T>`, `HashMap<K, V>`, `HashSet<T>` | `T[]`, `Map<K, V>`, `Set<T>` |
| List literal | `[1, 2, 3]` | `vec![1, 2, 3]` | `[1, 2, 3]` |
| Checked conversion | `count as U8` (traps if out of range) | `u8::try_from(count)?` | No equivalent |
| Lossy conversion | A named library function, never `as` | `x as i32` | `Math.trunc(x)` |
| Wrapping, saturating | `a +% b`, `a +\| b` | `a.wrapping_add(b)`, `a.saturating_add(b)` | No equivalent |
| Logical operators | `and`, `or`, `not`, `implies` | `&&`, `\|\|`, `!` | `&&`, `\|\|`, `!` |

Maps and sets iterate in insertion order in Brio, unlike Rust's `HashMap`, so output is the same on every run.

```brio
// Brio: units are checked by the compiler
unit ml, h
let volume = 500.0<ml>
let hours = 4.0<h>
let rate = volume / hours            // 125.0<ml/h>
let wrong = volume + hours           // compile error: cannot add ml and h
```

```rust
// Rust: units by convention only
let volume_ml = 500.0;
let hours = 4.0;
let rate_ml_per_h = volume_ml / hours;
let wrong = volume_ml + hours;       // compiles
```

```typescript
// TypeScript: units by convention only
const volumeMl = 500.0;
const hours = 4.0;
const rateMlPerH = volumeMl / hours;
const wrong = volumeMl + hours;      // compiles
```

## Functions

Brio functions look like Rust's, with two additions borrowed from Swift and Ada: argument labels at the call site, and *parameter modes* that say what a function does with each argument. The mode is repeated at the call site, so a reader sees at the call which arguments can change or be consumed.

| Concept | Brio | Rust | TypeScript |
| --- | --- | --- | --- |
| Function | `fn area(w: F64, h: F64) -> F64 { return w * h }` | `fn area(w: f64, h: f64) -> f64 { w * h }` | `function area(w: number, h: number): number { return w * h; }` |
| Expression body | `fn area(w: F64, h: F64) -> F64 = w * h` | No shorthand | `const area = (w: number, h: number) => w * h;` |
| Argument labels | `fn copy(from src: Path, to dst: Path)`, called as `copy(from: a, to: b)` | No labels | Object parameter `copy({ from: a, to: b })` |
| Default argument | `fn retry(times: I32 = 3)` | No default arguments | `function retry(times = 3)` |
| No return value | `-> ()` or omitted | `-> ()` or omitted | `: void` |
| Generic function | `fn largest[T: Ord](a: T, b: T) -> T` | `fn largest<T: Ord>(a: T, b: T) -> T` | `function largest<T>(a: T, b: T): T` |
| Method outside a block | `fn Reading.is_urgent(self) -> Bool` | Inside `impl Reading { … }` only | Inside `class Reading { … }` only |
| Closure | `\|x\| x * 2` | `\|x\| x * 2` | `(x) => x * 2` |
| Closure that takes ownership | `move \|x\| x + offset` | `move \|x\| x + offset` | Not needed (garbage collected) |
| Function type | `fn(I32) -> Bool`, or `Fn(I32) -> Bool` for closures | `fn(i32) -> bool`, `impl Fn(i32) -> bool` | `(x: number) => boolean` |

**Parameter modes.** The default mode only reads the argument. `inout` may change it, `take` consumes it, and `out` must initialize it. The compiler chooses how to pass each one (registers, copy or address); the mode states only what the function may do.

| Mode | Declared as | Called as | Rust equivalent | TypeScript equivalent |
| --- | --- | --- | --- | --- |
| Read only (default) | `fn show(r: Reading)` | `show(r)` | `fn show(r: &Reading)` · `show(&r)` | Any parameter (no enforcement) |
| May modify | `fn sort(xs: inout List[I32])` | `sort(mut xs)` | `fn sort(xs: &mut Vec<i32>)` · `sort(&mut xs)` | Any object parameter (no enforcement) |
| Consumes | `fn send(msg: take Message)` | `send(move msg)` | `fn send(msg: Message)` · `send(msg)` | No equivalent |
| Must initialize | `fn parse(text: Str, into: out Header)` | `parse(text, into: out h)` | `&mut MaybeUninit<Header>` (unsafe) | No equivalent |

```brio
// Brio
fn normalize(values: inout List[F64], by factor: F64) {
    for i in 0..<values.len() { values[i] /= factor }
}

var readings = [10.0, 20.0, 30.0]
normalize(mut readings, by: 10.0)
```

```rust
// Rust
fn normalize(values: &mut Vec<f64>, factor: f64) {
    for v in values.iter_mut() { *v /= factor; }
}

let mut readings = vec![10.0, 20.0, 30.0];
normalize(&mut readings, 10.0);
```

```typescript
// TypeScript
function normalize(values: number[], factor: number): void {
    for (let i = 0; i < values.length; i++) values[i] /= factor;
}

const readings = [10.0, 20.0, 30.0];
normalize(readings, 10.0);
```

## Control flow

Control flow is Rust's, with `is` for pattern tests, ranges that say whether the end is included, and loops that can state their own upper bound. Every `match` must cover every case, as in Rust.

| Concept | Brio | Rust | TypeScript |
| --- | --- | --- | --- |
| Condition | `if rate > max { … } else { … }` | `if rate > max { … } else { … }` | `if (rate > max) { … } else { … }` |
| Conditional value | `let label = if urgent { "now" } else { "later" }` | `let label = if urgent { "now" } else { "later" };` | `const label = urgent ? "now" : "later";` |
| Pattern test | `if alarm is .Critical(level) { … }` | `if let Alarm::Critical(level) = alarm { … }` | `if (alarm.kind === "critical") { const level = alarm.level; … }` |
| Unwrap or leave | `let .Some(p) = find(id) else { return }` | `let Some(p) = find(id) else { return };` | `const p = find(id); if (p === undefined) return;` |
| For loop | `for r in readings { … }` | `for r in &readings { … }` | `for (const r of readings) { … }` |
| Counting loop | `for i in 0..<n { … }` | `for i in 0..n { … }` | `for (let i = 0; i < n; i++) { … }` |
| While, infinite loop | `while busy { … }`, `loop { … }` | `while busy { … }`, `loop { … }` | `while (busy) { … }`, `while (true) { … }` |
| Labelled break | `outer: for … { break outer }` | `'outer: for … { break 'outer; }` | `outer: for (…) { break outer; }` |
| Bounded loop | `while waiting bounded 100 else { fail .Timeout } { … }` | No equivalent | No equivalent |
| Guaranteed tail call | `return tail step(state)` | Unstable (`become`) | No equivalent |

```brio
// Brio
match alarm {
    .Critical(level) if level > 8 => page_on_call(level)
    .Critical(level)              => notify(level)
    .Warning | .Info               => log.info("alarm", kind: alarm)
}
```

```rust
// Rust
match alarm {
    Alarm::Critical(level) if level > 8 => page_on_call(level),
    Alarm::Critical(level)              => notify(level),
    Alarm::Warning | Alarm::Info         => tracing::info!(kind = ?alarm, "alarm"),
}
```

```typescript
// TypeScript (no exhaustiveness check without extra code)
switch (alarm.kind) {
    case "critical":
        if (alarm.level > 8) pageOnCall(alarm.level); else notify(alarm.level);
        break;
    case "warning":
    case "info":
        console.info("alarm", alarm.kind);
        break;
}
```

## Data types

Brio's `record` and `enum` are Rust's `struct` and `enum`, with Swift-style construction by labels and variants written with a leading dot when the type is known. Records may state invariants that are checked or proven, and `distinct type` makes a new type that cannot be mixed up with its base type.

| Concept | Brio | Rust | TypeScript |
| --- | --- | --- | --- |
| Record | `record Reading { patient: PatientId, rate: F64<ml/h> }` | `struct Reading { patient: PatientId, rate: f64 }` | `interface Reading { patient: PatientId; rate: number }` |
| Construct | `Reading(patient: p, rate: 120.0<ml/h>)` | `Reading { patient: p, rate: 120.0 }` | `{ patient: p, rate: 120.0 }` |
| Field default | `retries: I32 = 3` | `#[derive(Default)]` or a constructor | `retries = 3` in a class |
| Copy with changes | `Reading(..r, rate: 90.0<ml/h>)` | `Reading { rate: 90.0, ..r }` | `{ ...r, rate: 90.0 }` |
| Invariant | `invariant rate <= 1_200.0<ml/h>` inside the record | No equivalent | No equivalent |
| Enum | `enum Priority { Low, Normal, Urgent }` | `enum Priority { Low, Normal, Urgent }` | `type Priority = "low" \| "normal" \| "urgent"` |
| Enum value | `.Urgent` (type known) or `Priority.Urgent` | `Priority::Urgent` | `"urgent"` |
| Variant with data | `Critical(level: I32, since: Instant)` | `Critical { level: i32, since: Instant }` | `{ kind: "critical"; level: number; since: Date }` |
| Distinct type | `distinct type PatientId = U64` | `struct PatientId(u64);` | Branded type `number & { __brand: "PatientId" }` |
| Distinct range type | `distinct type Bpm = I32 in 20..=300` | Validated newtype | No equivalent |
| Type alias | `type Ward = String` | `type Ward = String;` | `type Ward = string;` |
| C-style union | `union Raw { word: U32, bytes: [U8; 4] }` (reads are `unsafe`) | `union Raw { word: u32, bytes: [u8; 4] }` | No equivalent |
| Stable public layout | `frozen record …` | A `struct` without `#[non_exhaustive]` | No equivalent |

```brio
// Brio
record Infusion {
    patient: PatientId
    rate: F64<ml/h>
    invariant rate >= 0.0<ml/h> and rate <= 1_200.0<ml/h>
}

impl Infusion {
    fn is_fast(self) -> Bool = self.rate > 500.0<ml/h>
    fn slow_down(inout self, by step: F64<ml/h>) { self.rate -= step }
}

var drip = Infusion(patient: PatientId(42), rate: 600.0<ml/h>)
if drip.is_fast() { drip.slow_down(by: 100.0<ml/h>) }
```

```rust
// Rust
struct Infusion { patient: PatientId, rate: f64 }

impl Infusion {
    fn is_fast(&self) -> bool { self.rate > 500.0 }
    fn slow_down(&mut self, step: f64) { self.rate -= step; }
}

let mut drip = Infusion { patient: PatientId(42), rate: 600.0 };
if drip.is_fast() { drip.slow_down(100.0); }
```

```typescript
// TypeScript
class Infusion {
    constructor(public patient: PatientId, public rate: number) {}
    isFast(): boolean { return this.rate > 500.0; }
    slowDown(step: number): void { this.rate -= step; }
}

const drip = new Infusion(new PatientId(42), 600.0);
if (drip.isFast()) drip.slowDown(100.0);
```

## Traits and generics

Traits work as in Rust; generic parameters use square brackets, so angle brackets stay free for units. A trait name used as a type means *one kind, fixed at compile time* (fast, no keyword), and `any` means *the kind may vary at run time*.

| Concept | Brio | Rust | TypeScript |
| --- | --- | --- | --- |
| Trait | `trait Sensor { fn read(self) -> F64 }` | `trait Sensor { fn read(&self) -> f64; }` | `interface Sensor { read(): number }` |
| Implement a trait | `impl Sensor for Thermometer { fn read(self) -> F64 = self.celsius }` | `impl Sensor for Thermometer { fn read(&self) -> f64 { self.celsius } }` | `class Thermometer implements Sensor { read() { return this.celsius; } }` |
| Generic type | `record Pair[T] { a: T, b: T }` | `struct Pair<T> { a: T, b: T }` | `interface Pair<T> { a: T; b: T }` |
| Bounded generic | `fn log_all[T: Display](xs: [T])` | `fn log_all<T: Display>(xs: &[T])` | `function logAll<T extends Printable>(xs: T[])` |
| Where clause | `where T: Ord + Hash` | `where T: Ord + Hash` | `T extends A & B` |
| Static use (one kind, compile time) | `fn poll(s: Sensor)` | `fn poll(s: &impl Sensor)` | No distinction |
| Same kind, many values | `fn poll_all(s: [Sensor])` | `fn poll_all<S: Sensor>(s: &[S])` | No distinction |
| Dynamic use (kinds vary) | `fn log_all(s: [any Sensor])` | `fn log_all(s: &[Box<dyn Sensor>])` | `function logAll(s: Sensor[])` |
| Hidden concrete return type | `fn open_default() -> Sensor` | `fn open_default() -> impl Sensor` | Returns the interface type |
| Associated type | `type Item` inside a trait | `type Item;` | No equivalent |
| Derived implementations | `@derive(Eq, Hash, Debug)` | `#[derive(PartialEq, Eq, Hash, Debug)]` | No equivalent |

`any Sensor` is an ordinary value: it works in lists, fields and return types without a `Box`. Small implementations are stored inline; larger ones are placed on the heap when converted, which `@no_alloc` code forbids.

```brio
// Brio
trait Sensor { fn read(self) -> F64 }

fn average(sensors: [Sensor]) -> F64 {            // all one kind: specialized, inlinable
    var total = 0.0
    for s in sensors { total += s.read() }
    return total / (sensors.len() as F64)
}

let mixed: List[any Sensor] = [Thermometer(celsius: 37.0), PulseOximeter(spo2: 98.0)]
```

```rust
// Rust
trait Sensor { fn read(&self) -> f64; }

fn average<S: Sensor>(sensors: &[S]) -> f64 {
    let total: f64 = sensors.iter().map(|s| s.read()).sum();
    total / sensors.len() as f64
}

let mixed: Vec<Box<dyn Sensor>> = vec![Box::new(Thermometer { celsius: 37.0 }), Box::new(PulseOximeter { spo2: 98.0 })];
```

```typescript
// TypeScript
interface Sensor { read(): number }

function average(sensors: Sensor[]): number {
    const total = sensors.reduce((sum, s) => sum + s.read(), 0);
    return total / sensors.length;
}

const mixed: Sensor[] = [new Thermometer(37.0), new PulseOximeter(98.0)];
```

## Error handling

Errors are values, as in Rust, written `T ! E`: a function returning `Config ! LoadError` gives either a `Config` or a `LoadError`. `try` passes an error upward, `fail` returns one, and a result can never be ignored silently. There are no exceptions and no panics: a broken assumption is a *trap* that stops the program safely.

| Concept | Brio | Rust | TypeScript |
| --- | --- | --- | --- |
| Result type | `Config ! LoadError` | `Result<Config, LoadError>` | `Config` (and `throw`) |
| Pass the error up | `let c = try load(path)` | `let c = load(path)?;` | `const c = load(path);` (throws) |
| Return an error | `fail .NotFound(path)` | `return Err(LoadError::NotFound(path));` | `throw new NotFoundError(path);` |
| Recover locally | `let c = load(path) else \|e\| { use_defaults(e) }` | `let c = match load(path) { Ok(c) => c, Err(e) => use_defaults(e) };` | `try { … } catch (e) { … }` |
| Add context | `try load(path).context("reading pump config")` | `load(path).context("…")?` (anyhow) | `throw new Error("…", { cause: e })` |
| Any error, for scripts | `() ! AnyError` | `anyhow::Result<()>` | `Promise<void>` |
| Stop with a stated reason | `cfg.expect(reason: "validated at startup")` | `cfg.expect("validated at startup")` | `if (!cfg) throw new Error("…")` |
| Unwrap without a reason | Not available | `cfg.unwrap()` | `cfg!` |
| Postcondition on failure | `ensures on fail state == old(state)` | No equivalent | No equivalent |
| Ignoring a result | Compile error | Warning (`#[must_use]`) | Allowed |

```brio
// Brio
fn load_limits(path: Str) -> Limits ! ConfigError
    uses fs
{
    let text = try fs.read_text(path)
    let limits = try Limits.decode_toml(text)
    if limits.max_rate > 1_200.0<ml/h> { fail .OutOfRange("max_rate") }
    return limits
}
```

```rust
// Rust
fn load_limits(path: &str) -> Result<Limits, ConfigError> {
    let text = std::fs::read_to_string(path)?;
    let limits: Limits = toml::from_str(&text)?;
    if limits.max_rate > 1_200.0 { return Err(ConfigError::OutOfRange("max_rate")); }
    Ok(limits)
}
```

```typescript
// TypeScript
function loadLimits(path: string): Limits {
    const text = fs.readFileSync(path, "utf8");          // throws on failure
    const limits = toml.parse(text) as Limits;
    if (limits.maxRate > 1_200.0) throw new OutOfRangeError("maxRate");
    return limits;
}
```

## Ownership and memory

Brio has Rust's memory safety without a garbage collector, but most code never names a reference: parameter modes cover the common cases. Where a result borrows from an argument, `from` names the argument instead of a lifetime such as `'a`.

| Concept | Brio | Rust | TypeScript |
| --- | --- | --- | --- |
| Read-only borrow | Default parameter mode | `&T` | Not tracked |
| Exclusive borrow | `inout` parameter, `mut` at the call | `&mut T`, `&mut x` | Not tracked |
| Move ownership | `take` parameter, `move` at the call | Pass by value | Not tracked |
| Result borrowed from an argument | `fn first(xs: [Item]) -> ref Item from xs` | `fn first<'a>(xs: &'a [Item]) -> &'a Item` | Not tracked |
| From one of several | `-> ref Str from a \| b` | `fn f<'a>(a: &'a str, b: &'a str) -> &'a str` | Not tracked |
| Stored reference | `ref T`, `ref mut T` | `&'a T`, `&'a mut T` | Object reference |
| Explicit dereference | `r.*`, `r.* = v` | `*r`, `*r = v` | Not needed |
| Heap box, shared, thread-shared | `Box[T]`, `Rc[T]`, `Arc[T]` | `Box<T>`, `Rc<T>`, `Arc<T>` | Every object |
| Breaking cycles | `Weak[T]` | `Weak<T>` | `WeakRef<T>` |
| Interior mutability | `Cell[T]`, `RefCell[T]`, `Mutex[T]`, atomics | `Cell<T>`, `RefCell<T>`, `Mutex<T>`, atomics | Not needed |
| Raw pointers | `Ptr[T]`, `MutPtr[T]` | `*const T`, `*mut T` | No equivalent |
| Unsafe code | `unsafe("the buffer outlives the call") { … }` | `unsafe { … }` (reason in a comment) | No equivalent |
| Generic origin | `fn f[origin o](x: ref T from o)` | `fn f<'a>(x: &'a T)` | No equivalent |
| Freeing memory | Automatic at end of scope | Automatic at end of scope | Garbage collector |

Every `unsafe` block must state why it is sound, and every one is listed in the build's assurance ledger.

```brio
// Brio
fn longest(a: Str, b: Str) -> ref Str from a | b {
    return if a.len() >= b.len() { a } else { b }
}

fn archive(record: take Record, store: inout Archive) {
    store.add(move record)                 // record is moved; using it later is an error
}
```

```rust
// Rust
fn longest<'a>(a: &'a str, b: &'a str) -> &'a str {
    if a.len() >= b.len() { a } else { b }
}

fn archive(record: Record, store: &mut Archive) {
    store.add(record);
}
```

```typescript
// TypeScript (ownership is not tracked)
function longest(a: string, b: string): string {
    return a.length >= b.length ? a : b;
}

function archive(record: Record, store: Archive): void {
    store.add(record);                     // record can still be used and changed afterwards
}
```

## Contracts and verification

Brio functions can state what they require and guarantee, and the compiler either proves it or checks it at run time, per module. Neither Rust nor TypeScript has this in the language; in Rust it needs third-party verifiers such as Prusti, Creusot or Kani.

| Concept | Brio | Rust | TypeScript |
| --- | --- | --- | --- |
| Precondition | `requires rate >= 0.0<ml/h>` | `debug_assert!` by hand, or a verifier crate | Manual check |
| Postcondition | `ensures result <= limit` | Verifier crate only | Manual check |
| Value before the call | `ensures count == old(count) + 1` | Verifier crate only | No equivalent |
| Quantifier | `forall i in 0..<n: xs[i] >= 0` | Verifier crate only | `xs.every(x => x >= 0)` (run time) |
| Implication | `urgent implies notified` | `!urgent \|\| notified` | `!urgent \|\| notified` |
| Type invariant | `invariant low <= high` in a record | No equivalent | No equivalent |
| Loop invariant | `for i in 0..<n invariant total >= 0 { … }` | Verifier crate only | No equivalent |
| Termination measure | `decreases n` | Verifier crate only | No equivalent |
| Function that may run forever | `unbounded fn serve()` | Every function may | Every function may |
| Side-effect-free function | `@pure fn dose(…)` | `const fn` (narrower) | No equivalent |
| Verification mode per module | `prove`, `check`, `observe` or `assume`, set in `[verify]` of `Brio.toml` | No equivalent | No equivalent |

Functions are *bounded* by default: the compiler proves they finish. A function that may legitimately run forever, such as a server loop, says so with `unbounded fn`, and every caller sees it. Safety checks such as overflow and bounds are never assumed away, whatever the mode.

```brio
// Brio
@pure
fn clamp_rate(rate: F64<ml/h>, max: F64<ml/h>) -> F64<ml/h>
    requires max > 0.0<ml/h>
    ensures result >= 0.0<ml/h> and result <= max
= if rate < 0.0<ml/h> { 0.0<ml/h> } else if rate > max { max } else { rate }

fn sum(xs: [I64]) -> I64
    requires xs.len() <= 1_000_000
    requires forall x in xs: x >= 0 and x <= 1_000_000
    ensures result >= 0
{
    var total = 0
    for x in xs invariant total >= 0 { total += x }
    return total
}
```

```rust
// Rust (run-time checks by hand; proofs need an external verifier)
fn clamp_rate(rate: f64, max: f64) -> f64 {
    debug_assert!(max > 0.0);
    let result = rate.clamp(0.0, max);
    debug_assert!(result >= 0.0 && result <= max);
    result
}
```

```typescript
// TypeScript (run-time checks by hand)
function clampRate(rate: number, max: number): number {
    if (!(max > 0)) throw new RangeError("max must be positive");
    return Math.min(Math.max(rate, 0), max);
}
```

## Concurrency

Brio keeps Rust's guarantee of no data races and adds three more: tasks live inside a `tasks` block that waits for all of them, locks have levels so they cannot deadlock, and cancellation is an ordinary error the task handles. Blocking calls inside async code and locks held across `await` are compile errors.

| Concept | Brio | Rust | TypeScript |
| --- | --- | --- | --- |
| Async function | `async fn fetch(id: PatientId) -> Vitals ! NetError` | `async fn fetch(id: PatientId) -> Result<Vitals, NetError>` | `async function fetch(id: PatientId): Promise<Vitals>` |
| Await | `let v = try await fetch(id)` | `let v = fetch(id).await?;` | `const v = await fetch(id);` |
| Structured tasks | `tasks { spawn a(); spawn b() }` waits for both | `std::thread::scope` or `tokio::join!` | `await Promise.all([a(), b()])` |
| Cancellation | A `Cancelled` error at the next wait | Future dropped mid-way | `AbortController` |
| Mutex | `Mutex[Queue]`, a leaf lock by default | `Mutex<Queue>` | Not needed (one thread) |
| Nested locks without deadlock | `Mutex[Ward, level: 1]`, then `Mutex[Bed, level: 2]` | Not checked | Not applicable |
| Lock with a deadline | `try queue.lock(within: 10<ms>)` | `try_lock` loops by hand | Not applicable |
| Lock held across await | Compile error; use `AsyncMutex` | Allowed (deadlock risk) | Not applicable |
| Blocking call in async code | `blocking { read_file_sync(p) }` | `tokio::task::spawn_blocking` | Not applicable |
| Thread-safety markers | `Send`, `Sync` (automatic) | `Send`, `Sync` | Not applicable |
| Weak atomic ordering | `@ordering("single writer") count.store(n, .Release)` | `count.store(n, Ordering::Release)` | `Atomics.store` |
| Exhaustive interleaving test | `@interleavings(max: 3) test "…" { … }` | `loom` crate | No equivalent |

```brio
// Brio
unbounded fn run(sensor: inout Sensor, bus: Bus) -> () ! MonitorError {
    try tasks {
        spawn monitor(mut sensor)          // borrows sensor until the block ends
        spawn alarm_loop(bus)
    }
}
```

```rust
// Rust
async fn run(sensor: &mut Sensor, bus: Bus) -> Result<(), MonitorError> {
    tokio::try_join!(monitor(sensor), alarm_loop(bus))?;
    Ok(())
}
```

```typescript
// TypeScript
async function run(sensor: Sensor, bus: Bus): Promise<void> {
    await Promise.all([monitor(sensor), alarmLoop(bus)]);
}
```

## Capabilities and permissions

In Brio, code can touch files, the network, processes or devices only through *capability* values that it was given; nothing is reachable through globals. A program can do nothing outside its own memory unless its manifest grants it, and a dependency can do only what it is handed. Rust and TypeScript (Node.js) give every line of code full access by default; Deno's permission flags are the nearest equivalent, but per process, not per function.

| Concept | Brio | Rust | TypeScript |
| --- | --- | --- | --- |
| Program entry | `fn main(caps: Caps) -> () ! AnyError` | `fn main()` with full access | Module code with full access |
| Grant permissions | `[permissions]` in `Brio.toml` | No equivalent | `deno run --allow-net=api.example.com` |
| Declare what a function needs | `fn fetch(id: Id) -> Data ! NetError uses net` | No equivalent | No equivalent |
| Limit one capability for a block | `with net.only(["api.hospital.local:443"]) { … }` | No equivalent | No equivalent |
| Limit several | One `with` line per capability, stacked | No equivalent | No equivalent |
| Name a group of capabilities | `capability set Reporting = net, records, clock` | No equivalent | No equivalent |
| Define a capability | `capability PatientRecords as records { … }` | No equivalent | No equivalent |
| Create it at startup, if granted | `@grant fn from_manifest(fs: Dir) -> Self` | No equivalent | No equivalent |
| A more limited copy, proven | `@subset fn only_ward(self, w: WardId) -> Self` | No equivalent | No equivalent |
| Combine two you hold, proven | `@join fn join(self, other: Self) -> Self` | No equivalent | No equivalent |
| Package permissions | `[permissions.packages]` in `Brio.toml` | No equivalent | No equivalent |

```brio
// Brio
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
        try monitor.run()               // can reach only that host and that folder
    }
}
```

```rust
// Rust: any function can open any connection or file
fn fetch_vitals(id: PatientId) -> Result<Vitals, NetError> {
    let body = reqwest::blocking::get(format!("https://api.hospital.local/vitals/{id}"))?.text()?;
    Ok(serde_json::from_str(&body)?)
}
```

```typescript
// TypeScript (Node.js): any module can open any connection or file
async function fetchVitals(id: PatientId): Promise<Vitals> {
    const res = await fetch(`https://api.hospital.local/vitals/${id}`);
    return (await res.json()) as Vitals;
}
```

## Modules, packages and interoperability

Each `.brio` file is one module, and everything is private unless marked `public`. Brio imports Rust crates, C and C++ headers, Python modules and WebAssembly components with the same `use` keyword, so a Rust codebase can be migrated one module at a time.

| Concept | Brio | Rust | TypeScript |
| --- | --- | --- | --- |
| Module declaration | `module hospital.records` (optional; from the file path otherwise) | `mod records;` in the parent | One module per file |
| Import | `use hospital.records.{PatientRecords, ward_of}` | `use hospital::records::{PatientRecords, ward_of};` | `import { PatientRecords, wardOf } from "./records";` |
| Import with a new name | `use hospital.records as rec` | `use hospital::records as rec;` | `import * as rec from "./records";` |
| Public item | `public fn …` | `pub fn …` | `export function …` |
| Public within the package | `public(package) fn …` | `pub(crate) fn …` | Not exported from the package entry |
| Package manifest | `Brio.toml` | `Cargo.toml` | `package.json` |
| Add a dependency | `brio add hl7` | `cargo add hl7` | `npm install hl7` |
| Use a Rust crate | `use rust serde_json.{from_str}` | `use serde_json::from_str;` | Not applicable |
| Use a C header, with safety notes | `use c "vendor/pump_hal.h" as hal { fn hal_read(buf: out [U8; 64]) -> I32 }` | `bindgen` plus `unsafe` wrappers | Native addon (N-API) |
| Use a Python module | `use python pandas.{read_csv}` | `pyo3` | Child process or a bridge |
| Use a WebAssembly component | `use wit "sensors.wit" as sensors` | `wit-bindgen` | `jco` |
| Call C functions directly | `extern "C" { fn crc32(data: Ptr[U8], len: USize) -> U32 }` | `extern "C" { … }` | Not applicable |

```toml
# Brio.toml
[package]
name = "pump-controller"
version = "1.4.0"
edition = "2026"

[dependencies]
hl7 = "1.9"
ward-registry = { version = "2.4", registry = "internal" }

[rust-dependencies]
serde = "1"

[permissions]
net = ["api.hospital.local:443"]
fs.read = ["/etc/pump"]
```

```toml
# Cargo.toml (Rust)
[package]
name = "pump-controller"
version = "1.4.0"
edition = "2024"

[dependencies]
hl7 = "1.9"
serde = "1"
```

```json
// package.json (TypeScript)
{
  "name": "pump-controller",
  "version": "1.4.0",
  "dependencies": { "hl7": "^1.9.0" }
}
```

## Compile-time code and embedded languages

Brio replaces Rust's macro systems with ordinary Brio code that runs at compile time, in a sandbox without network access. Other languages such as SQL, GraphQL or HTML can be written inside Brio with `#` literals: they are parsed and type-checked at compile time, and every value inserted into them is passed as data, never as text, so injection attacks are impossible.

| Concept | Brio | Rust | TypeScript |
| --- | --- | --- | --- |
| Compile-time function | `comptime fn table_size(n: Int) -> Int` | `const fn` (limited) | No equivalent |
| Compile-time block | `comptime { … }` | `const { … }` (limited) | No equivalent |
| Generate code | `quote { fn $name() -> I32 = $value }` | `macro_rules!`, procedural macros | No equivalent |
| Derive implementations | `@derive(Eq, Hash, Encode, Decode)` | `#[derive(…)]` | No equivalent |
| Attribute | `@traced`, `@inline(always)` | `#[…]` | `@decorator` |
| Declare an embedded language | `language db = sql.Postgres(schema: "schema.sql")` | No equivalent | No equivalent |
| Embedded literal | `#db"SELECT name FROM patients WHERE id = ${id}"` | `sqlx::query!("… WHERE id = $1", id)` | `` sql`… WHERE id = ${id}` `` (tagged template) |
| Multi-line embedded literal | `#db{ … }` or `#db"""` … `"""` | Macro arguments | Tagged template |
| Checked against the schema at compile time | Yes, with editor completion inside the literal | With `sqlx` and a live database | No |

```brio
// Brio
language db = sql.Postgres(schema: "schema.sql")

fn ward_patients(ward: WardId) -> List[PatientRow] ! DbError
    uses db
{
    return try #db"""
        SELECT id, name, admitted_at
        FROM patients
        WHERE ward_id = ${ward}
        """
}
```

```rust
// Rust (sqlx)
async fn ward_patients(pool: &PgPool, ward: WardId) -> Result<Vec<PatientRow>, sqlx::Error> {
    sqlx::query_as!(PatientRow,
        "SELECT id, name, admitted_at FROM patients WHERE ward_id = $1", ward.0)
        .fetch_all(pool).await
}
```

```typescript
// TypeScript (a tagged-template SQL library)
async function wardPatients(ward: WardId): Promise<PatientRow[]> {
    return sql<PatientRow[]>`
        SELECT id, name, admitted_at
        FROM patients
        WHERE ward_id = ${ward}`;
}
```

## Observability, flags and experiments

Tracing, metrics, logs, feature flags and A/B experiments are part of the language, checked at compile time and exported over OpenTelemetry. Types such as `Sensitive` and `Secret` keep patient data and keys out of logs automatically.

| Concept | Brio | Rust | TypeScript |
| --- | --- | --- | --- |
| Trace a function | `@traced(patient)` | `#[tracing::instrument(fields(patient))]` | OpenTelemetry API by hand |
| Structured log | `log.info("infusion started", patient: id, rate: r)` | `tracing::info!(patient = ?id, rate = r, "infusion started")` | `logger.info({ patient: id, rate: r }, "infusion started")` |
| Metric | `metric alarms_raised: Counter(ward: WardId)` | `opentelemetry` meter by hand | `prom-client` or OpenTelemetry by hand |
| Redacted in logs and traces | `Sensitive[T]` | No equivalent (crates such as `secrecy`) | No equivalent |
| Zeroed secret, constant-time comparison | `Secret[T]` | `secrecy`, `zeroize` crates | No equivalent |
| Unvalidated outside input | `Untrusted[T]`, then `.validate(check)` | No equivalent | No equivalent |
| Feature flag | `flag new_alarm_tones: Bool = false` | A flag SDK by hand | A flag SDK by hand |
| Read a flag | `new_alarm_tones.value()` | SDK call | SDK call |
| A/B experiment | `experiment layout: Layout by ClinicianId { … }` | An experimentation SDK by hand | An experimentation SDK by hand |
| Variant for a unit | `layout.variant(for: clinician.id)` | SDK call | SDK call |
| Who flags and experiments apply to | `@audience(refresh: 24<h>) fn hospital_traits(…)` and `audience = hospital_traits` | No equivalent | No equivalent |
| Measured result per unit | `@experiment_outcome(window: 14 * 24<h>) fn …` | No equivalent | No equivalent |

```brio
// Brio
experiment triage_queue: QueueMode by HospitalId {
    control    = .Fifo
    audience   = hospital_traits
    eligible   = |h| h.monthly_active_users >= 10_000
    balance_by = |h| h.region
    metrics    = [median_wait_minutes]
    guardrails = [alarm_ack_seconds]
    hypothesis = "Priority ordering shortens median waits without slowing alarm response"
}

let mode = triage_queue.variant(for: hospital.id)       // never blocks; records the exposure
```

```rust
// Rust (an experimentation SDK; names vary by vendor)
let ctx = EvaluationContext::new(hospital.id.to_string()).with("region", hospital.region.as_str());
let mode: QueueMode = client.get_variant("triage_queue", &ctx).unwrap_or(QueueMode::Fifo);
```

```typescript
// TypeScript (an experimentation SDK; names vary by vendor)
const ctx = { targetingKey: hospital.id, region: hospital.region };
const mode = await client.getStringValue("triage_queue", "fifo", ctx);
```

## Safety-critical and embedded code

One attribute puts a module in the `critical` profile: no heap allocation after start-up, no recursion, no dynamic dispatch, bounded loops, and every safety check proven rather than checked at run time. Embedded features (interrupts, shared data, device registers, real-time deadlines, verified state machines) are built in rather than left to libraries.

| Concept | Brio | Rust | TypeScript |
| --- | --- | --- | --- |
| Safety-critical module | `@profile(critical, level: "IEC62304-C") module pump.control` | Ferrocene plus coding guidelines | Not used for this |
| Allocate only at start-up | `@init fn setup(…)` | By convention | Not applicable |
| No allocation, no dynamic calls | `@no_alloc`, `@no_dyn` | By convention | Not applicable |
| Fixed-capacity collections | `BoundedList[T, 32]`, `BoundedString[64]`, `Pool[T, 8]` | `heapless::Vec<T, 32>` | Not applicable |
| Interrupt handler | `@interrupt(TIMER0, priority: 3) fn on_tick()` | `#[interrupt] fn TIMER0()` (`cortex-m-rt`) | Not applicable |
| Data shared with interrupts | `Shared[Counts, ceiling: 3]`, `s.lock(\|c\| { … })` | `critical_section::Mutex` | Not applicable |
| Device register | `Volatile[U32]` | `core::ptr::read_volatile` | Not applicable |
| Periodic task with a deadline | `@period(10<ms>) @deadline(8<ms>) fn control_step()` | RTIC task attributes | Not applicable |
| Verified state machine | `@state_machine(safe: .Stopped) enum PumpState { … }` with an `@transition` function | No equivalent | No equivalent |
| GPU code in critical modules | Not allowed (compile error) | Not checked | Not applicable |
| GPU kernel | `@kernel fn scale(item: WorkItem, …)` | `rust-gpu` or CUDA bindings | WebGPU shaders |

```brio
// Brio
@profile(critical, level: "IEC62304-C")
module pump.control

@state_machine(safe: .Stopped)
enum PumpState { Stopped, Priming, Infusing, Alarmed }

@transition
fn next(state: take PumpState, event: PumpEvent) -> PumpState = match state {
    .Stopped  => if event is .Start { .Priming } else { .Stopped }
    .Priming  => if event is .Primed { .Infusing } else if event is .Fault { .Stopped } else { .Priming }
    .Infusing => if event is .Occlusion { .Alarmed } else if event is .Fault { .Stopped } else { .Infusing }
    .Alarmed  => if event is .Acknowledged or event is .Fault { .Stopped } else { .Alarmed }
}
```

```rust
// Rust (the safe-state property is checked by tests or external tools)
#[derive(Clone, Copy)]
enum PumpState { Stopped, Priming, Infusing, Alarmed }

fn next(state: PumpState, event: PumpEvent) -> PumpState {
    use PumpState::*;
    match (state, event) {
        (Stopped, PumpEvent::Start) => Priming,
        (Priming, PumpEvent::Primed) => Infusing,
        (Infusing, PumpEvent::Occlusion) => Alarmed,
        (_, PumpEvent::Fault) | (Alarmed, PumpEvent::Acknowledged) => Stopped,
        (s, _) => s,
    }
}
```

```typescript
// TypeScript (not used for safety-critical device code)
type PumpState = "stopped" | "priming" | "infusing" | "alarmed";
```

## Testing, benchmarks and scripts

Tests and benchmarks are language items, property tests need no library, and any `.brio` file with top-level statements is a script that compiles to native code and runs in milliseconds once cached.

| Concept | Brio | Rust | TypeScript |
| --- | --- | --- | --- |
| Unit test | `test "clamps negative rates" { assert clamp_rate(-1.0<ml/h>, max: 10.0<ml/h>) == 0.0<ml/h> }` | `#[test] fn clamps_negative_rates() { assert_eq!(…); }` | `test("clamps negative rates", () => { expect(…).toBe(0); });` |
| Property test | `test "never exceeds max" forall (r: F64<ml/h>) { … }` | `proptest!` crate | `fast-check` library |
| Benchmark | `bench "parse HL7" { let msg = sample(); measure { parse(msg) } }` | `criterion` crate | `tinybench` or `vitest bench` |
| Run tests | `brio test` | `cargo test` | `npx vitest` |
| Run under every flag and variant | `brio test --flags all --experiments all` | No equivalent | No equivalent |
| Deterministic simulation | `brio sim --seed 42` | No equivalent | No equivalent |
| Script | `.brio` file with top-level code, run with `brio run` | `cargo script` (unstable) | `.ts` file run with `tsx` or `deno` |
| Inline dependencies | Frontmatter between `---` lines | Frontmatter (unstable) | Not available |
| Interactive shell | `brio repl` | No equivalent | `ts-node`, `deno repl` |

```brio
#!/usr/bin/env brio run
---
[dependencies]
hl7 = "1.9"

[permissions]
fs.read = ["./messages"]
---
use hl7.{parse}

for path in try fs.list("./messages") {
    let msg = try parse(try fs.read_text(path))
    print("{path}: {msg.kind}")
}
```

```rust
// Rust (a binary crate with Cargo.toml listing hl7)
fn main() -> anyhow::Result<()> {
    for entry in std::fs::read_dir("./messages")? {
        let path = entry?.path();
        let msg = hl7::parse(&std::fs::read_to_string(&path)?)?;
        println!("{}: {}", path.display(), msg.kind);
    }
    Ok(())
}
```

```typescript
// TypeScript (run with tsx; hl7 installed with npm)
import { readdirSync, readFileSync } from "node:fs";
import { parse } from "hl7";

for (const name of readdirSync("./messages")) {
    const msg = parse(readFileSync(`./messages/${name}`, "utf8"));
    console.log(`${name}: ${msg.kind}`);
}
```

**Everyday commands.** `brio build`, `brio run`, `brio test`, `brio bench`, `brio fmt`, `brio lint`, `brio doc`, `brio add`, `brio update`, `brio audit`, `brio caps` (show every capability flow), `brio prove` (run the verifier), `brio ledger` (what was proven, checked or assumed) and `brio ship` (all release gates). Rust developers will recognise most of them from Cargo.

## Keyword map

Every Brio keyword and its nearest counterpart in Rust and TypeScript; "none" means the language has no built-in equivalent.

| Brio | Meaning | Rust | TypeScript |
| --- | --- | --- | --- |
| `let`, `var`, `const`, `static` | Immutable, mutable, compile-time, global | `let`, `let mut`, `const`, `static` | `const`, `let`, `const`, module `let` |
| `fn`, `async fn`, `comptime fn` | Function, async function, compile-time function | `fn`, `async fn`, `const fn` | `function`, `async function`, none |
| `unbounded` | Function may run forever | none | none |
| `return`, `return tail`, `fail` | Return, guaranteed tail call, return an error | `return`, `become` (unstable), `return Err(…)` | `return`, none, `throw` |
| `try`, `await` | Propagate an error, wait for a result | `?`, `.await` | none, `await` |
| `if`, `else`, `is`, `match` | Conditions and patterns | `if`, `else`, `if let`, `match` | `if`, `else`, none, `switch` |
| `for`, `in`, `while`, `loop`, `break`, `continue` | Loops | Same | `for … of`, `while`, `break`, `continue` |
| `bounded` | Upper bound on loop iterations | none | none |
| `and`, `or`, `not`, `implies` | Logical operators | `&&`, `\|\|`, `!` | `&&`, `\|\|`, `!` |
| `record`, `enum`, `union`, `distinct type`, `type` | Data types | `struct`, `enum`, `union`, newtype, `type` | `interface`, union of literals, none, branded type, `type` |
| `frozen` | Layout fixed for compatibility | default for structs | none |
| `trait`, `impl`, `where`, `Self`, `self` | Traits and implementations | Same | `interface`, `implements`, `extends`, `this` |
| `any` | Kind may vary at run time | `dyn` | none (always dynamic) |
| `inout`, `take`, `out` | Parameter modes | `&mut`, by value, none | none |
| `mut`, `move`, `out` (at a call) | Call-site markers | `&mut`, implicit move, none | none |
| `ref`, `from`, `origin` | References and where they borrow from | `&`, lifetime `'a` | none |
| `requires`, `ensures`, `invariant`, `decreases`, `old`, `result`, `forall`, `exists` | Contracts | none (verifier crates) | none |
| `assert` | Run-time assertion | `assert!` | `console.assert` |
| `tasks`, `spawn` | Structured concurrency | `thread::scope`, `spawn` | `Promise.all` |
| `blocking` | Run blocking code off the async executor | `spawn_blocking` | none |
| `unsafe("reason")` | Unchecked code with a stated reason | `unsafe` | none |
| `uses`, `with` | Capabilities a function needs; limiting them for a block | none | none |
| `capability`, `capability set` | Define a capability; name a group | none | none |
| `use`, `module`, `public`, `as` | Modules and visibility | `use`, `mod`, `pub`, `as` | `import`, file, `export`, `as` |
| `use rust`, `use c`, `use cpp`, `use python`, `use wit` | Import from another ecosystem | none (`bindgen`, `pyo3`) | none |
| `extern`, `rust bridge` | Foreign functions; Rust bridge declarations | `extern` | none |
| `language`, `#name"…"` | Embedded language and its literal | none (macros) | Tagged templates |
| `quote`, `comptime` | Generate code, run at compile time | `quote!`, `const` | none |
| `unit` | Units of measure | none | none |
| `test`, `bench`, `measure` | Tests and benchmarks | `#[test]`, `criterion` | Test framework |
| `metric`, `flag`, `experiment` | Telemetry, feature flags, A/B tests | none (crates) | none (SDKs) |
| `@grant`, `@subset`, `@join` | Create, limit and combine capabilities | none | none |
| `@audience`, `@experiment_outcome` | Who flags and experiments apply to; measured results | none | none |
| `@profile`, `@init`, `@no_alloc`, `@no_dyn` | Safety-critical rules | none | none |
| `@interrupt`, `@period`, `@deadline`, `@state_machine`, `@transition`, `@kernel` | Embedded, real-time and GPU code | `#[interrupt]`, none | none |
| `@pure`, `@traced`, `@derive`, `@inline`, `@vectorize` | Purity, tracing, derived code, optimization guarantees | `#[derive]`, `#[inline]`, `#[instrument]` | Decorators |
