# Brio Language Specification

Sep 28, 2026 · Angelo Fornerino

This document is the normative definition of the Brio programming language, version 0.1 (edition 2026). Version 0.1 incorporates the adversarial security and soundness review and the performance review of September 2026. Where it differs from the Brio Language Design Spec, this document governs.

This specification is published under the Creative Commons Attribution 4.0 International license (CC BY 4.0). Brio's reference implementation is licensed under Apache-2.0 WITH LLVM-exception.

## 1 Scope, conformance and notation

### 1.1 Scope

**\[GEN-1\]** This specification defines the syntax, static semantics and dynamic semantics of Brio source programs, the format and meaning of Brio interface files, the structure of Brio packages, the rules for interoperation with Rust and C, and the minimum requirements for a conforming Brio toolchain.

**\[GEN-2\]** This specification does not define the complete standard library. Clause 4 and Clause 12 define the library items the language itself depends on (the *core items*); all other library content is specified separately.

**\[GEN-3\]** This specification does not define performance. Statements about speed are informative only.

### 1.2 Normative language

**\[GEN-4\]** The key words MUST, MUST NOT, SHALL, SHALL NOT, SHOULD, SHOULD NOT, MAY and OPTIONAL are to be interpreted as described in RFC 2119 and RFC 8174 when, and only when, they appear in capitals.

**\[GEN-5\]** Each normative rule carries an identifier in brackets, such as **\[GEN-5\]**. Identifiers are stable across revisions: a removed rule's identifier is never reused. Tests, diagnostics and traceability records SHOULD cite rule identifiers.

**\[GEN-6\]** Text labelled *Note* or *Example*, and every annex marked informative, is informative. All other text is normative.

### 1.3 Categories of behavior

**\[GEN-7\]** A rule stating that a construct *is ill-formed* or that a program *SHALL NOT* do something requires a conforming implementation to reject the program with at least one error diagnostic.

**\[GEN-8\]** A *trap* is a defined runtime outcome: execution of the current program stops as specified in Clause 10. A trap is not undefined behavior.

**\[GEN-9\]** *Implementation-defined* behavior is behavior that each implementation SHALL document. *Unspecified* behavior is behavior among a set of permitted outcomes that an implementation need not document. Annex B lists both.

**\[GEN-10\]** *Undefined behavior* is behavior for which this specification imposes no requirements. A program that uses no `unsafe` blocks and no foreign code, other than code produced by the bridge generator, SHALL NOT be able to exhibit undefined behavior, whatever its contracts, assumptions and verification modes (\[CON-28\]). Clause 15 lists the conditions under which undefined behavior can occur.

### 1.4 Conformance

**\[GEN-11\]** A *conforming program* is a program that is not ill-formed and that, when every contract in it holds, does not exhibit undefined behavior.

**\[GEN-12\]** A *conforming implementation* SHALL:

1. accept every conforming program, subject only to documented resource limits;
2. reject every ill-formed program with at least one error diagnostic;
3. give every accepted program the meaning this specification assigns to it;
4. document all implementation-defined behavior listed in Annex B.

**\[GEN-13\]** An implementation MAY provide extensions, provided that no conforming program changes meaning. Programs that use an extension SHALL be diagnosed when compiled with the `--strict` option.

**\[GEN-14\]** A *conforming toolchain* is a conforming implementation that also meets the requirements of Clause 19.

**\[GEN-15\]** An implementation SHALL record in every build artifact whether it was produced by a qualified toolchain. Until an implementation has been qualified under a safety standard, it SHALL mark all artifacts, including those built with the `critical` profile, as *unqualified*.

### 1.5 Notation

**\[GEN-16\]** Syntax is given in an extended Backus–Naur form:

| Form | Meaning |
| --- | --- |
| `A ::= B` | A is defined as B |
| `"x"` | The literal token `x` |
| `A B` | A followed by B |
| `A \| B` | A or B |
| `A?` | Zero or one A |
| `A*` | Zero or more A |
| `A+` | One or more A |
| `( A )` | Grouping |
| `UPPER` | A token class defined in Clause 3 |

**\[GEN-17\]** Where a clause gives a grammar fragment and Annex A gives the collected grammar, Annex A governs if they differ.

**\[GEN-18\]** Each clause is organized, where applicable, as *Syntax*, *Legality rules* (static semantics), *Dynamic semantics*, *Implementation requirements* and *Examples*.

### 1.6 Normative references

| Reference | Used for |
| --- | --- |
| RFC 2119, RFC 8174 | Requirement key words |
| The Unicode Standard, version 15.0 or later | Source text, `Char` and `Str` |
| IEEE 754-2019 | Floating-point semantics |
| ISO/IEC 14882:2020 (C++20), clause 6.9.2 | Memory model for atomic operations |
| The Rust Reference, edition 2024 | Semantics of bridged Rust items |
| Semantic Versioning 2.0.0 | Package versions |
| TOML 1.0.0 | Manifest format |

*Note:* IEC 62304, ISO 26262, IEC 61508 and DO-178C are informative references for Clause 16; this specification does not claim compliance with them.

## 2 Terms and definitions

**\[TERM-1\]** For the purposes of this specification, the following terms apply. Terms defined here are printed in italics at their first use in a clause.

| Term | Definition |
| --- | --- |
| Borrow | A temporary, non-owning access to a place, either shared or exclusive. |
| Bridge | Generated code, in Brio and Rust, that connects Brio code to a Rust crate (Clause 18). |
| Check mode | Verification mode in which a contract is evaluated at runtime (Clause 11). |
| Contract | A `requires`, `ensures`, `invariant`, `decreases` or `bounded` clause. |
| Core item | A library item on which the language semantics depend, such as `Option` or `Drop`. |
| Diverging expression | An expression of type `Never`; it never produces a value. |
| Exclusive borrow | A borrow through which the place may be mutated and during which no other access to the place is permitted. |
| Frozen type | A public record or enum whose layout or variant set is part of its package's interface (17.6). |
| Ghost evaluation | Evaluation of a contract expression that has no effect on the program's observable behavior. |
| Initialization phase | Execution of `@init` functions before operational execution begins (16.3). |
| Interface file | A `.brioh` file, generated by the toolchain, describing the interface of one module (17.3). |
| Item | A top-level declaration: a function, type, trait, implementation, constant, unit, test or bridge. |
| Module | The items of one `.brio` source file. |
| Opaque type | A public type whose layout is hidden from client packages. |
| Origin | A compile-time name for the set of places a reference may borrow from (Clause 9). |
| Owner | The binding or place that is responsible for dropping a value. |
| Package | A unit of distribution and versioning, described by one `Brio.toml` manifest. |
| Parameter mode | One of default (shared borrow), `inout`, `take` or `out`. |
| Place | An expression that denotes a memory location: a variable, a field of a place, an element of a place, or a dereferenced reference. |
| Profile | A named set of restrictions applied to a module: `standard` or `critical` (Clause 16). |
| Prove mode | Verification mode in which a contract is discharged statically (Clause 11). |
| Pure function | A function marked `@pure` that has no observable effects (11.2). |
| Range type | A type whose values are those of a base type restricted to a range (5.3). |
| Shared borrow | A borrow through which the place may be read but not mutated. |
| Trap | A defined abnormal termination (Clause 10). |
| Trap handler | The function that runs when a trap occurs (10.4). |
| Unit of measure | A compile-time dimension attached to a numeric type (5.1). |
| Value semantics | The rule, under the `critical` profile, that references are never stored in data structures (16.2). |

## 3 Lexical structure

### 3.1 Source text

**\[LEX-1\]** A source file SHALL be encoded in UTF-8. A byte order mark at the start of the file is permitted and ignored.

**\[LEX-2\]** A line terminator is LF (U+000A) or the sequence CR LF. A CR not followed by LF is ill-formed.

**\[LEX-3\]** The following code points SHALL NOT appear anywhere in a source file, including comments and string literals: U+0000, the bidirectional formatting characters U+202A–U+202E and U+2066–U+2069, and any other character of general category Cc except tab (U+0009), LF and CR. They MAY be written in literals by escape sequences.

*Note:* This rule prevents source that displays differently from how it compiles.

**\[LEX-4\]** Source files use the extension `.brio`. Interface files use the extension `.brioh`; they are generated by the toolchain into the build directory and are never written by hand (\[PKG-25\]).

### 3.2 Whitespace and comments

**\[LEX-5\]** Space (U+0020) and tab are whitespace. Whitespace separates tokens and is otherwise insignificant, except where 3.7 and 3.8 give it meaning.

**\[LEX-6\]** A line comment starts with `//` and extends to the end of the line. A block comment starts with `/*` and ends with the matching `*/`; block comments nest. An unterminated block comment is ill-formed.

**\[LEX-7\]** A comment beginning with exactly `///` is an *outer documentation comment* and attaches to the following item, field, variant or parameter. A comment beginning with `//!` is an *inner documentation comment* and attaches to the enclosing module. A documentation comment that attaches to nothing is ill-formed.

### 3.3 Identifiers

**\[LEX-8\]** An identifier matches `[A-Za-z_][A-Za-z0-9_]*`, is not a single `_`, and is not a keyword. Identifiers are case-sensitive.

**\[LEX-9\]** Identifiers are restricted to ASCII in edition 2026. Non-ASCII text MAY appear in comments and literals.

**\[LEX-10\]** A *raw identifier* is written between backticks, such as `` `match` ``. It denotes the identifier with the enclosed name, which MAY be a keyword. Raw identifiers exist chiefly to name Rust items whose names are Brio keywords.

**\[LEX-11\]** Identifiers beginning with `__brio` are reserved for the implementation. A program that declares one is ill-formed.

### 3.4 Keywords

**\[LEX-12\]** The following are reserved keywords and SHALL NOT be used as identifiers except as raw identifiers:

```text
and       any       as        assert    async     await     bounded
break     comptime  const     continue  decreases distinct  else
ensures   enum      exists    extern    fail      false     fn
for       forall    frozen    if        impl      implies   in
inout     interface invariant is        let       loop      match
module    move      mut       not       old       opaque    or
origin    out       public    quote     record    ref       requires
return    rust      self      Self      spawn     static    take
tasks     test      trait     true      try       type      unbounded
union     unit      unsafe    use       var       where     while
```

**\[LEX-13\]** The following are reserved for future use and SHALL NOT be used as identifiers: `defer`, `effect`, `ghost`, `macro`, `region`, `some`, `yield`.

**\[LEX-14\]** The following are *contextual keywords*. They have special meaning only in the contexts listed and are ordinary identifiers elsewhere:

| Word | Context |
| --- | --- |
| `result` | Inside an `ensures` clause |
| `error` | Inside an `ensures on fail` clause |
| `on` | After `ensures` |
| `package` | Inside `public(...)` |
| `bridge`, `instantiate` | Inside a `rust bridge` declaration |

In addition, `capability`, `language`, `metric`, `flag`, `experiment` and `bench` are contextual keywords at the start of an item (Clauses 21, 22 and 41); `by` is a contextual keyword in an `experiment` item; `measure` is a contextual keyword inside a `bench` item; `uses` and `with` are contextual keywords as defined in 41.7; `tail` is a contextual keyword immediately after `return` (\[PERF-7\]); and `python`, `c`, `cpp` and `wit` are contextual keywords immediately after `use` (Clauses 20 and 25).

### 3.5 Punctuators

**\[LEX-15\]** The punctuators are:

```text
(   )   [   ]   {   }   ,   ;   :   .   ..  ..<  ..=  .*
?   ?.  ??  !   ->  =>  @   $   =   ==  !=  <   <=   >   >=
+   -   *   /   %   +%  -%  *%  +|  -|  *|  &   |    ^   ~
<<  >>  +=  -=  *=  /=  %=  &=  |=  ^=  <<= >>= +%=  -%= *%=
```

**\[LEX-16\]** Tokenization is greedy: the longest punctuator that matches is taken, except as stated in \[LEX-26\] and \[LEX-27\].

### 3.6 Literals

**\[LEX-17\]** *Integer literals* are decimal, hexadecimal (`0x`), octal (`0o`) or binary (`0b`). A decimal literal other than `0` SHALL NOT begin with `0`. The underscore `_` MAY appear between two digits and has no meaning. Integer literals have no type suffixes; their type is determined by 4.9.

**\[LEX-18\]** A *floating-point literal* is a decimal literal with a fractional part (`1.5`), an exponent (`1e9`), or both. Digits SHALL appear on both sides of the decimal point. A hexadecimal floating-point literal (`0x1.8p3`) denotes an exactly representable value.

**\[LEX-19\]** A *character literal* is a single Unicode scalar value or escape sequence between single quotes: `'a'`, `'\n'`, `'\u{1F600}'`.

**\[LEX-20\]** The escape sequences are `\n`, `\r`, `\t`, `\0`, `\\`, `\'`, `\"`, `\{`, `\}`, `\u{H…}` (one to six hexadecimal digits denoting a Unicode scalar value) and, in byte literals only, `\xHH`. Any other escape is ill-formed.

**\[LEX-21\]** A *string literal* is enclosed in `"`. It SHALL NOT contain an unescaped line terminator. An unescaped `{` begins an *interpolation*, which extends to the matching `}`. An interpolation contains an expression, optionally followed by `:` and a format specification (7.12). The interpolated expression SHALL NOT contain a string literal, a character literal or a line terminator.

**\[LEX-22\]** A *multi-line string literal* begins with `"""` followed immediately by a line terminator, and ends with `"""` on a line containing only whitespace before it. The whitespace preceding the closing delimiter is removed from the start of every line; a line that does not begin with that whitespace is ill-formed. Escapes and interpolation behave as in \[LEX-21\].

**\[LEX-23\]** A *raw string literal* is `r"…"` or `r#"…"#` with any number of `#` characters. It contains no escapes and no interpolation and ends at the first `"` followed by the same number of `#`.

**\[LEX-24\]** A *byte literal* is `b'…'` and a *byte string literal* is `b"…"`. They contain only ASCII characters and escapes and do not permit interpolation.

**\[LEX-25\]** The Boolean literals are `true` and `false`.

### 3.7 Units on literals and tuple indices

**\[LEX-26\]** When a `<` immediately follows a numeric literal with no intervening whitespace, it begins a *unit annotation*, which extends to the matching `>` and contains a unit expression (5.1). With whitespace, `<` is the less-than operator. Example: `12.5<m/s>` is a literal with a unit; `x < y` is a comparison.

**\[LEX-27\]** Immediately after a `.` token, a decimal digit sequence is tokenized as an integer literal even if a further `.` and digits follow. Thus `t.0.1` is the tuple-field access `(t.0).1`, not `t` followed by `0.1`.

### 3.8 Newlines and statement separation

**\[LEX-28\]** The lexer emits a NEWLINE token for each maximal run of line terminators, whitespace and comments that contains at least one line terminator. The parser treats each NEWLINE as a statement separator equivalent to `;`, except where \[LEX-29\] to \[LEX-32\] cause it to be ignored.

**\[LEX-29\]** A NEWLINE is ignored when it occurs inside a `(…)` or `[…]` delimiter pair and is not inside a `{…}` block nested within that pair.

**\[LEX-30\]** A NEWLINE is ignored when the last token before it is one of: `,`, `(`, `[`, `{`, `.`, `?.`, `->`, `=>`, `:`, `=`, a compound assignment operator, a binary operator (7.2), or one of the keywords `and`, `or`, `not`, `implies`, `as`, `is`, `in`, `else`, `try`, `await`, `mut`, `move`, `out`, `requires`, `ensures`, `invariant`, `decreases`, `bounded`, `where`.

**\[LEX-31\]** A NEWLINE is ignored when the first token after it is one of: `.`, `?.`, `??`, `=>`, `else`, `requires`, `ensures`, `invariant`, `decreases`, `bounded`, `where`, `and`, `or`, `implies`, `as`, `is`, or a binary operator other than `-` and `|`. A NEWLINE immediately before `{` is ignored when the syntactic context requires a block at that position.

**\[LEX-32\]** Inside a match arm list, a NEWLINE that follows a complete match arm is never ignored under \[LEX-31\]. Consequently, a match arm that begins with an implicit enum member such as `.Low(r)` always starts a new arm.

*Note:* A NEWLINE after `return` is never ignored, so `return` at the end of a line returns no value.

*Example:*

```brio
let total = subtotal
    + tax                       // continues: next line starts with +
let name = patient?.ward
    ?.name ?? "unassigned"      // continues: next line starts with ?.
```

## 4 Types

### 4.1 Type categories

**\[TYPE-1\]** Every expression has exactly one static type, determined at compile time. Brio has primitive types, compound types, nominal types (records, enums, distinct types, unions), reference and pointer types, function types, dynamic types (`any`), and range subtypes.

**\[TYPE-2\]** Records, enums, distinct types and unions are compared by name (nominal equivalence). Tuples, arrays, slices, optionals, results, references and function types are compared by structure. A `type` alias without a range constraint is transparent: it denotes the same type as its definition.

### 4.2 Primitive types

| Type | Values | Size |
| --- | --- | --- |
| `Bool` | `true`, `false` | 1 byte; representation 0 or 1 |
| `Char` | Unicode scalar values | 4 bytes |
| `I8`, `I16`, `I32`, `I64`, `I128` | Two's-complement signed integers | 1, 2, 4, 8, 16 bytes |
| `U8`, `U16`, `U32`, `U64`, `U128` | Unsigned integers | 1, 2, 4, 8, 16 bytes |
| `ISize`, `USize` | Pointer-sized integers | Implementation-defined |
| `Int`, `UInt` | Aliases of `I64` and `U64` | 8 bytes |
| `F32`, `F64` | IEEE 754 binary32 and binary64 | 4, 8 bytes |
| `Str` | Valid UTF-8 byte sequences (dynamically sized) | See 4.8 |
| `()` | The single value `()` | 0 bytes |
| `Never` | No values | 0 bytes |

**\[TYPE-3\]** `Int` and `UInt` are fixed at 64 bits on every target so that program behavior does not depend on the target's word size.

### 4.3 Integer arithmetic

**\[TYPE-4\]** The operators `+`, `-`, `*` and unary `-` on integer types produce the mathematically exact result. If that result is not representable in the operand type, the operation *overflows*. Overflow is a trap in check mode and SHALL be proven impossible in prove mode.

**\[TYPE-5\]** The *wrapping* operators `+%`, `-%` and `*%` produce the exact result reduced modulo 2ⁿ into the operand type. The *saturating* operators `+|`, `-|` and `*|` produce the exact result clamped to the type's minimum or maximum. Neither kind overflows.

**\[TYPE-6\]** `/` truncates toward zero. `%` produces a remainder with the sign of the dividend. Division or remainder by zero, and signed `MIN / -1` or `MIN % -1`, trap in check mode and SHALL be proven impossible in prove mode.

**\[TYPE-7\]** `<<` and `>>` shift by the value of the right operand, which SHALL be less than the bit width of the left operand; otherwise the shift traps or SHALL be proven impossible. `>>` is arithmetic on signed types and logical on unsigned types. The bitwise operators `&`, `|`, `^` and `~` operate on the two's-complement representation.

### 4.4 Floating-point arithmetic

**\[TYPE-8\]** Operations on `F32` and `F64` follow IEEE 754-2019 with round-to-nearest, ties-to-even. An implementation SHALL NOT contract separate operations into fused operations, SHALL NOT flush subnormal values to zero, and SHALL NOT reorder floating-point operations, unless the programmer requests it with `@fp(fast)` (16.5).

**\[TYPE-9\]** Floating-point operations never trap outside the `critical` profile. `%` on floating-point types computes the IEEE remainder with the sign of the dividend (the same result as C's `fmod`).

**\[TYPE-10\]** Floating-point types implement `PartialEq` and `PartialOrd` but not `Eq`, `Ord` or `Hash`.

### 4.5 Compound types

**\[TYPE-11\]** A *tuple type* `(T1, T2, …, Tn)` with n ≥ 2 holds one value of each component type. Components are accessed as `t.0`, `t.1`, and so on. A parenthesized single type `(T)` is `T`.

**\[TYPE-12\]** An *array type* `[T; N]` holds exactly N values of type T, where N is a constant expression of type `USize`.

**\[TYPE-13\]** A *slice type* `[T]` denotes a contiguous sequence of zero or more values of type T whose length is known only at runtime.

**\[TYPE-14\]** A *function type* `fn(T1, …, Tn) -> R` denotes a pointer to a function with no captured state. Parameter modes are part of the function type, for example `fn(inout Buffer) -> ()`. Closures have anonymous types that implement `Fn`, `FnMut` or `FnOnce` (12.8).

### 4.6 Optional and result types

**\[TYPE-15\]** `T?` is notation for the core enum `Option[T]`, whose variants are `.Some(T)` and `.None`.

**\[TYPE-16\]** `T ! E` is notation for the core enum `Result[T, E]`, whose variants are `.Ok(T)` and `.Err(E)`. In type syntax `?` binds tighter than `!`, so `T? ! E` is `Result[Option[T], E]`. `!` is not associative: `A ! B ! C` is ill-formed.

**\[TYPE-17\]** A value of type `T ! E` SHALL NOT be discarded implicitly. An expression statement whose value has a `Result` type is ill-formed unless it is explicitly discarded with `let _ = expr`.

### 4.7 References and pointers

**\[TYPE-18\]** `ref T` is a shared reference to a value of type T and `ref mut T` is an exclusive reference. A reference type MAY carry an origin, written `ref T from o` (9.6). References are never null and always refer to a valid, initialized value of their type.

**\[TYPE-19\]** The core types `Box[T]`, `Rc[T]` and `Arc[T]` are owning pointers with the semantics of Rust's `Box<T>`, `Rc<T>` and `Arc<T>`. The core types `Ptr[T]` and `MutPtr[T]` are raw pointers usable only as described in Clause 15.

**\[TYPE-20\]** The postfix operator `.*` dereferences a reference or owning pointer and denotes the referenced place. Field access and method calls dereference automatically, so `r.field` means `r.*.field`.

### 4.8 Dynamically sized types

**\[TYPE-21\]** `[T]` and `Str` are *dynamically sized*. A dynamically sized type SHALL appear only as the target of a reference or owning pointer, or as the type of a parameter in default or `inout` mode. It SHALL NOT be the type of a local variable, a field other than the last field of a record, an array element, or a return value, except as permitted by \[OWN-34\]. Dynamic values (`any`) are sized (\[TRAIT-12\]).

### 4.9 Type inference and literal types

**\[TYPE-22\]** The parameter types, return type and error type of every function, and the types of every constant, static and field, SHALL be written explicitly. Types of local bindings, closure parameters and generic arguments MAY be inferred. Inference is local to one function body.

**\[TYPE-23\]** An integer literal takes the integer type required by its context. If the context does not determine a type, it has type `Int`. A floating-point literal takes the floating-point type required by its context, or `F64`. A literal whose value is not representable in its type is ill-formed.

### 4.10 Conversions

**\[TYPE-24\]** The only implicit conversions are:

1. from `T` to `T?`, producing `.Some(value)`;
2. between a range subtype and its base type, as defined in 5.3;
3. from `Never` to any type;
4. from `ref mut T` to `ref T`;
5. when passing an argument to a default-mode or `inout` parameter: from `String` to `Str`, from `List[T]` or `[T; N]` to `[T]`, and from `Box[T]`, `Rc[T]` or `Arc[T]` to `T`; and, wherever a value of type `any Bounds` is expected, from any sized type implementing the bounds (\[TRAIT-12\], \[TRAIT-21\]).

In addition, a `Formatted` value converts to `String` as specified in \[FMT-2\]. There are no implicit numeric conversions.

**\[TYPE-25\]** The expression `T(x)` converts `x` to `T` using the core trait `From`. Implementations of `From` between primitive numeric types exist only where every value converts exactly (for example `I64(x)` for `x: I32`).

**\[TYPE-26\]** The expression `x as T` performs a *checked conversion*:

| From | To | Behavior |
| --- | --- | --- |
| Integer | Integer | The value, if representable; otherwise traps |
| Integer | Float | Nearest representable value, ties to even; never traps |
| Float | Integer | Truncated toward zero, if representable; traps on NaN or out-of-range values |
| `F64` | `F32` | Rounded to nearest, ties to even; values beyond `F32` range become infinities |
| `F32` | `F64` | Exact |
| `U32` | `Char` | The scalar value, if valid; otherwise traps |
| Base type | Range subtype | See 5.3 |
| Distinct type | Underlying type, or the reverse | See 5.4 |
| Numeric with unit | Same dimension, other unit | See 5.2 |

Any other `as` conversion is ill-formed. In prove mode, every trap condition in this table SHALL be proven impossible.

*Note:* Bit-reinterpreting conversions are library functions such as `x.wrapping_as[U8]()` and `f.to_bits()`, so that lossy conversions are always visible by name.

### 4.11 Layout

**\[TYPE-27\]** The layout of a record or enum without a `@repr` attribute is unspecified, and the implementation MAY reorder fields. `@repr(C)` gives the layout C would give. `@repr(transparent)` on a record with exactly one non-zero-sized field gives it the layout of that field.

**\[TYPE-28\]** `Option[ref T]`, `Option[ref mut T]`, `Option[Box[T]]` and `Option[fn(…) -> R]` SHALL have the same size as the non-optional type, with `.None` represented by the null pointer.

### 4.12 Core traits of the type system

**\[TYPE-29\]** The following core traits have meaning in the language itself:

| Trait | Meaning |
| --- | --- |
| `Copy` | Values are duplicated bitwise on use instead of moved. Implementable only if every field is `Copy` and the type does not implement `Drop`. |
| `Clone` | Values can be duplicated explicitly with `.clone()`. |
| `Drop` | The type has a destructor, run when an owner goes out of scope (9.8). |
| `PartialEq`, `Eq` | The `==` and `!=` operators. |
| `PartialOrd`, `Ord` | The `<`, `<=`, `>` and `>=` operators. |
| `Hash`, `Debug`, `Display`, `Default` | Hashing, debug formatting, user formatting, default values. |
| `From` | Explicit conversion with `T(x)` and error conversion in `try` (10.2). |
| `Iterator`, `IntoIterator` | `for` loops (7.9). |
| `Send`, `Sync` | Transfer between and sharing across tasks (14.3). |
| `Sized` | The type has a size known at compile time. |
| `Fn`, `FnMut`, `FnOnce` | Callable values (12.8). |

**\[TYPE-30\]** `Send`, `Sync` and `Sized` are *auto traits*: the implementation derives them from a type's structure following the rules of the Rust Reference. `Copy`, `Clone`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `Hash`, `Debug` and `Default` MAY be derived with `@derive`.

**\[TYPE-31\]** Whether the result of a floating-point operation is NaN is fully specified, but the sign and payload bits of a NaN result are unspecified and MAY differ between targets. Code that must be bit-reproducible SHALL NOT depend on the bits of NaN values.

## 5 Units of measure, range types and distinct types

### 5.1 Units of measure

*Syntax:*

```text
UnitDecl    ::= "unit" UnitDef ("," UnitDef)*
UnitDef     ::= IDENT ("=" ScaledUnit)?
ScaledUnit  ::= (Number "*")? UnitExpr ("/" Number)?
UnitExpr    ::= UnitFactor (("*" | "/") UnitFactor)*
UnitFactor  ::= IDENT ("^" "-"? INTEGER)? | "(" UnitExpr ")" | "1"
UnitAnnot   ::= "<" UnitExpr ">"
```

**\[UNIT-1\]** A `UnitDef` without `=` declares a *base unit*, which introduces a new, independent dimension. A `UnitDef` with `=` declares a *derived unit*, equal to the given unit expression multiplied by an exact rational scale factor.

**\[UNIT-2\]** Unit names occupy their own namespace, separate from values and types, and are looked up only inside unit annotations and unit declarations. Units are items: they follow the visibility and import rules of Clause 17.

**\[UNIT-3\]** Every unit expression has a *normal form*: a product of base units with non-zero integer exponents, and a positive rational scale factor. Two unit expressions are *the same unit* if their normal forms are equal, and have *the same dimension* if their products of base units are equal. Derived unit expressions such as `m/s` need no declaration.

**\[UNIT-4\]** A *unit-annotated type* `N<u>` is formed from a numeric primitive type N and a unit expression u. `N<1>` is the same type as N. A unit-annotated type has the same size, alignment and representation as N.

**\[UNIT-5\]** Scale factors SHALL be written as integer or decimal literals and are exact. Units with an offset, such as degrees Celsius relative to kelvin, cannot be declared; conversions involving offsets are ordinary functions.

**\[UNIT-6\]** The operators on unit-annotated values are:

| Operation | Requirement | Result unit |
| --- | --- | --- |
| `a + b`, `a - b`, `a % b` | Same unit | That unit |
| `a == b`, `a < b` and other comparisons | Same unit | `Bool` |
| `a * b` | None | Product of units |
| `a / b` | None | Quotient of units |
| `-a` | None | Unit of `a` |

Operands of the same dimension but different scale, such as `m` and `km`, are not the same unit and SHALL be converted explicitly.

**\[UNIT-7\]** A numeric literal without a unit annotation is dimensionless. It SHALL NOT be used where a unit-annotated type is required, except for the literals `0` and `0.0`, which convert implicitly to any unit.

**\[UNIT-8\]** `x as N<v>` converts a value of unit `u` to unit `v` of the same dimension by multiplying by the ratio of their scale factors. For an integer type N the conversion SHALL be exact: an inexact result traps in check mode and SHALL be proven impossible in prove mode. Conversion between different dimensions is ill-formed.

*Note:* A unit is removed by dividing by a unit literal and added by multiplying by one: `d / 1.0<m>` is dimensionless, and `x * 1.0<m>` has unit `m`.

**\[UNIT-9\]** A generic parameter declared as `unit u` ranges over unit expressions, so library functions can be generic over units:

```brio
fn sqrt[unit u](x: F64<u^2>) -> F64<u>
```

**\[UNIT-10\]** The `Display` implementation of a unit-annotated value writes the number, a space, and the unit. The textual form of the unit is implementation-defined.

*Example:*

```brio
unit m, s, kg, g
unit km = 1000 * m
unit mg = g / 1000
unit h  = 3600 * s
unit ml, N = kg * m / s^2

let speed = 12.5<m/s>
let trip  = (speed * 2.0<h>) as F64<km>  // 90.0 km
let bad   = speed + 1.0<km/h>            // ill-formed: different units
```

### 5.2 Range types

*Syntax:*

```text
RangeTypeDecl ::= "type" IDENT "=" Type "in" ConstExpr ("..<" | "..=") ConstExpr
```

**\[RNG-1\]** A range type declaration introduces a *range type* R whose values are the values of the *base type* B that lie within the given range. B SHALL be an integer type, a floating-point type, `Char`, a unit-annotated numeric type, or another range type.

**\[RNG-2\]** The bounds SHALL be constant expressions of type B, and the range SHALL be non-empty. Bounds of a floating-point range SHALL be finite. A value of a floating-point range type is never NaN.

**\[RNG-3\]** Range constraints appear only in named `type` and `distinct type` declarations. An anonymous range type, such as a parameter declared `x: Int in 0 ..= 9`, is ill-formed.

**\[RNG-4\]** A value of range type R converts implicitly to its base type B, without any check.

**\[RNG-5\]** A value of type B converts implicitly to R wherever an R is required: in initialization, assignment, argument passing, return and field initialization. The conversion performs a *range check*. A failed range check traps in check mode; in prove mode the check SHALL be proven to succeed.

**\[RNG-6\]** Arithmetic operators apply to R through its base type: the operands convert to B and the result has type B. A compound assignment such as `r += 1` computes in B and range-checks the result on store.

**\[RNG-7\]** Every range type R provides the constants `R.min` and `R.max`, the function `R.contains(x: B) -> Bool`, and the function `R.try_from(x: B) -> R?`, which never traps.

**\[RNG-8\]** A range type has the same representation as its base type. An implementation MAY assume that every value of type R is within its range. A value outside the range can be produced only by `unsafe` code, and doing so is undefined behavior.

**\[RNG-9\]** Exhaustiveness checking of `match` (8.4) takes range types into account. A `match` over a `Percent` with arms `0 ..= 49` and `50 ..= 100` is exhaustive.

*Example:*

```brio
type Percent   = Int in 0 ..= 100
type HeartRate = U16 in 20 ..= 300
type Altitude  = F64<m> in -500.0<m> ..= 20_000.0<m>

fn raise(p: inout Percent) {
    p += 10          // computed as Int, range-checked on store
}
```

### 5.3 Distinct types

*Syntax:*

```text
DistinctDecl ::= "distinct" "type" IDENT GenericParams? "=" Type
                 ("in" ConstExpr ("..<" | "..=") ConstExpr)?
```

**\[DIST-1\]** A distinct type declaration introduces a new nominal type D with the same representation as its *underlying type* U. D and U are different types, and no implicit conversion exists between them.

**\[DIST-2\]** `x as U` converts a D to its underlying type, and `v as D` converts a U to D. If D has a range constraint, the conversion to D performs a range check as in \[RNG-5\].

**\[DIST-3\]** D implements `Copy`, `Clone`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `Hash` and `Debug` automatically if U does. Comparison operators apply only between two values of D. D inherits no other operations or trait implementations.

**\[DIST-4\]** Arithmetic operators are made available on D only by deriving the core operator traits, for example `@derive(Add, Sub)`. A derived operator applies U's operator and produces a D.

*Example:*

```brio
distinct type PatientId = U64

@derive(Add, Sub)
distinct type Doses = U32 in 0 ..= 48

let id = 1042 as PatientId
let raw: U64 = id as U64
let wrong = id + 1          // ill-formed: PatientId has no +
```

## 6 Declarations and items

### 6.1 Items and namespaces

**\[ITEM-1\]** The items of a module are visible throughout that module regardless of their order in the file.

**\[ITEM-2\]** There are four namespaces: *types* (records, enums, unions, distinct types, aliases, traits, generic type parameters), *values* (functions, constants, statics, local bindings), *units* (Clause 5) and *modules*. Two items in the same scope and namespace SHALL NOT have the same name.

**\[ITEM-3\]** Paths use `.` as the separator in every context, for example `pump.controller.Controller.new`. A path is resolved by looking up its first segment in the innermost enclosing scope, then the module, then names imported by `use`, then the core prelude.

### 6.2 Visibility

**\[ITEM-4\]** An item, field or associated item is *private* by default: it is visible only within its module. `public(package)` makes it visible within its package. `public` makes it visible to every package that depends on its package.

**\[ITEM-5\]** Enum variants have the visibility of their enum. Record fields are private unless marked. An item SHALL NOT expose in its public signature a type that is less visible than the item itself.

### 6.3 Constants and statics

*Syntax:*

```text
ConstDecl  ::= "const" IDENT ":" Type "=" Expr
StaticDecl ::= "static" "var"? IDENT ":" Type "=" Expr
```

**\[ITEM-6\]** A `const` declares a compile-time constant. Its initializer SHALL be evaluable at compile time (Clause 13). Each use of a constant denotes a fresh copy of its value.

**\[ITEM-7\]** A `static` declares a single immutable value with a fixed address for the whole execution. Its initializer SHALL be evaluable at compile time. Its type SHALL implement `Sync`.

**\[ITEM-8\]** A `static var` declares a mutable global. Every read or write of a `static var` is an unsafe operation (Clause 15). `static var` is forbidden under the `critical` profile.

### 6.4 Type aliases

**\[ITEM-9\]** `type A[P…] = T` declares an alias. Without a range constraint it is transparent (\[TYPE-2\]); with one it declares a range type (5.2).

### 6.5 Records

*Syntax:*

```text
RecordDecl ::= "frozen"? "record" IDENT GenericParams? WhereClause?
               "{" (FieldDecl | TypeInvariant) (SEP (FieldDecl | TypeInvariant))* SEP? "}"
FieldDecl  ::= Attribute* Visibility? IDENT ":" Type ("=" ConstExpr)?
TypeInvariant ::= "invariant" Expr
SEP        ::= "," | NEWLINE
```

**\[ITEM-10\]** A record is a nominal product type with named fields. A field with `= ConstExpr` has a default value.

**\[ITEM-11\]** Every record has a *memberwise initializer*, called with the record's name and labeled arguments: `Reading(patient: p, rate: r, at: t)`. Every field without a default SHALL be given exactly once. Arguments MAY appear in any order and are evaluated in the order written. An argument that is a bare identifier with the same name as the field MAY omit its label: `Reading(patient, rate, at: now)`.

**\[ITEM-12\]** The memberwise initializer is usable only where every field is visible. Outside that scope, records are constructed through functions.

**\[ITEM-13\]** The *record update* form `Reading(..base, rate: r)` creates a record whose unnamed fields are moved or copied from `base`. It is usable only where every field is visible.

**\[ITEM-14\]** A record with one or more `invariant` clauses has a *type invariant*: the conjunction of those clauses, in which `self` denotes the record value. Clause 11 defines when invariants are checked. The fields of a record with a type invariant are read-only outside the record's module, whatever their visibility.

### 6.6 Enums

*Syntax:*

```text
EnumDecl ::= "frozen"? "enum" IDENT GenericParams? WhereClause?
             "{" Variant (SEP Variant)* SEP? "}"
Variant  ::= Attribute* IDENT ("(" VariantField ("," VariantField)* ","? ")")?
             ("=" ConstExpr)?
VariantField ::= Type | IDENT ":" Type
```

**\[ITEM-15\]** An enum is a nominal sum type. Each variant has no payload, a positional payload (`Low(HeartRate)`) or a labeled payload (`SensorFault(code: U16, since: Instant)`). A single variant SHALL NOT mix positional and labeled fields.

**\[ITEM-16\]** A variant is constructed with its qualified name, `Alarm.Low(35)`, or, where the expected type is known, with the implicit member form `.Low(35)`. Labeled payloads are constructed with labels, following the rules of \[ITEM-11\].

**\[ITEM-17\]** Explicit discriminants (`= ConstExpr`) are permitted only when no variant has a payload, or when the enum has a `@repr` integer attribute. Discriminants SHALL be distinct.

**\[ITEM-18\]** An enum SHALL NOT contain itself by value, directly or indirectly. Recursive enums use an owning pointer such as `Box`.

### 6.7 Unions

**\[ITEM-19\]** A `union` declares a type whose fields share storage. A union SHALL carry `@repr(C)`. Writing a field is safe. Reading a field is an unsafe operation. Unions are forbidden under the `critical` profile.

### 6.8 Functions

*Syntax:*

```text
FnDecl     ::= FnQualifier* "fn" FnName GenericParams? "(" ParamList? ")"
               ("->" Type OriginSpec?)? WhereClause? UsesClause? Contract* FnBody?
FnQualifier ::= "unsafe" | "unbounded" | "async" | "comptime" | "extern" STRING
FnName     ::= IDENT | TypePath "." IDENT
ParamList  ::= Param ("," Param)* ","?
Param      ::= SelfParam | Attribute* IDENT? IDENT ":" Mode? Type ("=" ConstExpr)?
SelfParam  ::= Mode? "self"
Mode       ::= "inout" | "take" | "out"
OriginSpec ::= "from" IDENT ("|" IDENT)*
FnBody     ::= Block | "=" Expr
```

**\[FN-1\]** A function whose return type is omitted returns `()`. A function body given as `= Expr` returns the value of Expr. A function body given as a block SHALL end every path with `return`, `fail`, a diverging expression, or, if the return type is `()` or `() ! E`, the end of the block. In a function returning `() ! E`, reaching the end of the body or executing `return` without a value returns `.Ok(())`.

**\[FN-2\]** A parameter written with two identifiers before `:`, as in `to rate: MlPerHour`, has the *label* `to` and the *name* `rate`. An argument for a labeled parameter SHALL be written with its label: `set_rate(to: x)`. An argument for an unlabeled parameter MAY be written positionally or with the parameter's name as a label.

**\[FN-3\]** Arguments SHALL appear in parameter order. Positional arguments SHALL precede labeled ones. A parameter with a default value MAY be omitted; its default SHALL be a constant expression.

**\[FN-4\]** Parameter modes and call-site markers are defined in Clause 9. Contracts are defined in Clause 11. `async` is defined in Clause 14. `unsafe` and `extern` are defined in Clause 15. `comptime` is defined in Clause 13.

**\[FN-5\]** A function without a body is permitted only as a trait member, in an `extern` block, in an API notes block (\[FFI-12\]), or in a generated interface file.

**\[FN-6\]** A function declared inside `impl T { … }`, or declared with the qualified name form `fn T.name(…)`, is an *associated function* of T. The qualified form is permitted only in the module that declares T and is equivalent to a member of an inherent `impl T`. An associated function whose first parameter is a `SelfParam` is a *method*.

**\[FN-7\]** An associated function is called as `T.name(args)`. A method is also called as `value.name(args)`; the receiver is borrowed or moved according to the mode of its `self` parameter, without a call-site marker.

**\[FN-8\]** A package that builds an executable SHALL declare exactly one function `main` in its root module, with one of the signatures `fn main()`, `fn main() -> () ! E` where E implements `Display`, or either of these with `async`.

### 6.9 Traits and implementations

**\[ITEM-20\]** `trait` declarations and `impl` blocks are defined in Clause 12.

### 6.10 Tests

*Syntax:*

```text
TestDecl ::= "test" STRING ("forall" "(" ParamList ")")? Block
```

**\[ITEM-21\]** A `test` item is compiled only when building tests. Its string names the test and SHALL be unique within its module. The semantics of tests, including `forall` property tests, are defined in 19.4.

### 6.11 Attributes

*Syntax:*

```text
Attribute ::= "@" Path ("(" AttrArgs? ")")?
```

**\[ATTR-1\]** An attribute applies to the item, field, variant, parameter, statement or expression that immediately follows it. The attributes are those listed below and those defined in Clauses 20 to 41, each in the clause that defines it. Any other attribute is ill-formed unless its path begins with `rust.`, `python.`, `lint.` or `doc.`.

| Attribute | Applies to | Defined in |
| --- | --- | --- |
| `@profile(standard \| critical, level: STRING?)` | Module | 16.1 |
| `@verify(prove \| check \| assume, STRING?)` | Module, item | 11.6 |
| `@pure` | Function | 11.2 |
| `@init` | Function | 16.3 |
| `@trap_handler` | Function | 10.4 |
| `@trusted(STRING)` | Function | 16.4 |
| `@req(STRING, …)` | Item | 19.7 |
| `@derive(Trait, …)` | Record, enum, distinct type | 12.9 |
| `@deriver(Trait)` | `comptime` function | 13.5 |
| `@specialize(Type, …)` | Generic function | 12.7 |
| `@inlinable` | Function | 17.6 |
| `@inline(always \| never)` | Function | Annex B |
| `@hot`, `@no_alloc`, `@no_dyn` | Function | 16.5 |
| `@max_stack(bytes)` | Function | 16.2 |
| `@fp(strict \| fast)` | Module, function | 4.4, 16.5 |
| `@repr(C \| transparent \| U8 … I64)` | Record, enum, union | 4.11 |
| `@export(c, STRING)` | Function | 15.5 |
| `@deprecated(STRING)` | Item | 19.1 |
| `@gen(fn)` | Type, test parameter | 19.4 |
| `@rust.attr(…)`, `@rust.derive(…)` | Public item | 18.6 |

## 7 Expressions and statements

### 7.1 Place and value expressions

**\[EXPR-1\]** A *place expression* denotes a memory location: a local binding, a parameter, a `static`, a field of a place (`p.f`, `p.0`), an indexed element of a place (`p[i]`), or a dereference (`r.*`). Every other expression is a *value expression*.

**\[EXPR-2\]** A place is *mutable* if it is a `var` binding, an `inout` or `out` parameter, a dereference of a `ref mut` or of an owning pointer reached through a mutable place, or a field or element of a mutable place. Assignment, `mut` arguments and `ref mut` SHALL be applied only to mutable places.

### 7.2 Operator precedence

**\[EXPR-3\]** Operators bind in the following order, from tightest to loosest:

| Level | Operators | Associativity |
| --- | --- | --- |
| 1 | `.name` `.0` `(…)` `[…]` `?.` `.*` (postfix) | Left |
| 2 | `-` `~` `try` `await` `ref` `ref mut` (prefix) | Right |
| 3 | `as` | Left |
| 4 | `*` `/` `%` `*%` `*\|` | Left |
| 5 | `+` `-` `+%` `-%` `+\|` `-\|` | Left |
| 6 | `<<` `>>` | Left |
| 7 | `&` | Left |
| 8 | `^` | Left |
| 9 | `\|` | Left |
| 10 | `..<` `..=` | None |
| 11 | `??` | Right |
| 12 | `==` `!=` `<` `<=` `>` `>=` `in` `is` | None |
| 13 | `not` (prefix) | Right |
| 14 | `and` | Left |
| 15 | `or` | Left |
| 16 | `implies` | Right |

The *binary operators* referred to in 3.8 are those at levels 3 to 12 and 14 to 16.

**\[EXPR-4\]** Operators at a level marked *None* SHALL NOT be chained: `a < b < c` is ill-formed.

**\[EXPR-5\]** An expression that combines `and` and `or` without parentheses separating them is ill-formed. `a and b or c` must be written `(a and b) or c`.

### 7.3 Evaluation order

**\[EXPR-6\]** Evaluation is strictly left to right. Operands are evaluated before the operator is applied. In a call, the callee and receiver are evaluated first, then the arguments in the order written. In an assignment, the subexpressions of the place are evaluated first, then the right-hand side, then the store occurs.

**\[EXPR-7\]** `and`, `or`, `implies`, `??` and `?.` evaluate their right operand only when needed. `a implies b` evaluates `b` only when `a` is `true`.

### 7.4 Primary expressions

**\[EXPR-8\]** Literals denote values of the types given in 3.6 and 4.9. A string literal without interpolation has type `ref Str from static`. A string literal with interpolation has the core type `Formatted`, which implements `Display` and converts to `String` as specified in 7.12.

**\[EXPR-9\]** An *implicit member expression* `.Name` or `.Name(args)` denotes a variant or associated constant of the *expected type*. The expected type is determined by a type annotation, a parameter type, a return type, the other operand of `==` or `!=`, or the scrutinee of a `match`. Without an expected type, an implicit member expression is ill-formed.

**\[EXPR-10\]** Tuple expressions are `(a, b, …)`. Array expressions are `[a, b, …]` and `[v; N]`, where v's type SHALL implement `Copy` or N SHALL be a constant not greater than 1.

### 7.5 Blocks

**\[EXPR-11\]** A block `{ … }` introduces a scope. When a block is used as a value, as the branch of an `if` or `match`, the body of a closure, or an `unsafe` or `comptime` block, its value is the value of its final statement if that statement is an expression statement, and `()` otherwise.

**\[EXPR-12\]** The block that forms a function body is not used as a value. A function returns only by `return`, `fail`, reaching the end of a `()` function, or an `= Expr` body (\[FN-1\]).

### 7.6 Operator semantics

**\[EXPR-13\]** Arithmetic, bitwise and shift operators on primitive types are defined in Clause 4. On other types they call the core operator traits listed in 12.10.

**\[EXPR-14\]** `==` and `!=` call `PartialEq`. `<`, `<=`, `>` and `>=` call `PartialOrd`. Both operands SHALL have the same type.

**\[EXPR-15\]** `x in c` calls the core trait `Contains` and has type `Bool`. Ranges, slices, arrays, strings, sets and maps implement `Contains`.

**\[EXPR-16\]** `e is P` tests whether the value of `e` matches pattern P and has type `Bool`. When `e is P` appears as a condition, or as a conjunct joined by `and` in a condition, the bindings of P are in scope in the parts of the condition to its right and in the block executed when the condition is true.

**\[EXPR-17\]** For `a` of type `T?`, `a ?? b` evaluates to the payload of `a` if it is `.Some`, and to `b` otherwise. `b` SHALL have type `T` or `T?`; the result has the type of `b`.

**\[EXPR-18\]** For `a` of type `T?`, `a?.m` evaluates to `.None` if `a` is `.None`, and otherwise to `a`'s payload`.m` wrapped in `.Some`. If `.m` already has an optional type, the result is not wrapped again.

**\[EXPR-19\]** `a ..< b` and `a ..= b` construct the core range values `Range[T]` and `ClosedRange[T]`. A range whose lower bound is not below its upper bound (for `..<`) or exceeds its upper bound (for `..=`) is empty; constructing one never traps.

**\[EXPR-20\]** `a[i]` calls the core trait `Index` (or `IndexMut` in a mutable place context). For arrays, slices and lists, an index outside the bounds traps in check mode and SHALL be proven impossible in prove mode. `a[r]` with a range `r` produces a sub-slice with the same checks.

**\[EXPR-21\]** When the callee of `x[…]` resolves to a generic item, the brackets supply generic arguments instead of an index: `max[I32](a, b)`.

### 7.7 Statements

*Syntax:*

```text
Stmt       ::= LetStmt | VarStmt | AssignStmt | AssertStmt | LocalItem | ExprStmt | ";"
LetStmt    ::= "let" Pattern (":" Type)? "=" Expr ("else" ("|" Pattern "|")? Block)?
VarStmt    ::= "var" IDENT (":" Type)? ("=" Expr)?
AssignStmt ::= PlaceExpr AssignOp Expr
AssertStmt ::= "assert" Expr ("," Expr)?
LocalItem  ::= FnDecl | ConstDecl
```

**\[STMT-1\]** `let` binds the names in its pattern immutably. Without `else`, the pattern SHALL be irrefutable.

**\[STMT-2\]** With `else`, the pattern MAY be refutable and the `else` block SHALL have type `Never`. If the initializer has type `T ! E` and the pattern is not a variant pattern of `Result`, the pattern is matched against the `.Ok` payload and `|p|` MAY bind the error. If the initializer has type `T?`, the pattern is matched against the `.Some` payload and `|p|` SHALL NOT be written.

*Example:*

```brio
let frame = port.read_frame() else |e| {
    log.warn("retrying after {e}")
    return .Retry
}
let ward = patient.ward else { return .Unassigned }
```

**\[STMT-3\]** `var` declares a mutable binding. A `var` without an initializer SHALL be definitely assigned on every path before it is read.

**\[STMT-4\]** A local binding SHALL NOT have the same name as another local binding or parameter of the same function, including those of enclosing closures and blocks.

*Note:* Brio forbids shadowing because two variables with the same name in one function are a common source of review errors.

**\[STMT-5\]** `AssignOp` is `=` or a compound assignment operator from 3.5. `a op= b` has the meaning of `a = a op b`, except that the subexpressions of `a` are evaluated once. A tuple of places MAY be assigned from a tuple expression: `(a, b) = (b, a)`.

**\[STMT-6\]** `assert c` and `assert c, message` are checked according to the verification mode of the enclosing item, exactly as a contract (Clause 11). `message` SHALL have type `Str` or `Formatted`.

**\[STMT-7\]** An expression statement whose value has a type other than `()` or `Never` is ill-formed, except as the final statement of a block used as a value (7.5) or where the value is explicitly discarded with `let _ = expr`.

### 7.8 Conditionals

*Syntax:*

```text
IfExpr ::= "if" Expr Block ("else" (IfExpr | Block))?
```

**\[CTRL-1\]** The condition SHALL have type `Bool`. An `if` without `else` has type `()`. An `if` with `else` used as a value has the common type of its branches.

### 7.9 Loops

*Syntax:*

```text
LoopExpr  ::= Label? "loop" LoopSpec* Block
WhileExpr ::= Label? "while" Expr LoopSpec* Block
ForExpr   ::= Label? "for" Pattern "in" Expr LoopSpec* Block
LoopSpec  ::= "invariant" Expr | "decreases" Expr | "bounded" Expr ("else" Block)?
Label     ::= IDENT ":"
```

**\[CTRL-2\]** `loop` repeats its block until a `break`. Its type is the type of the values given to `break`, or `Never` if it contains no `break` that targets it.

**\[CTRL-3\]** `while c B` evaluates c before each iteration and runs B while c is `true`. Bindings introduced by `is` in c are in scope in B (\[EXPR-16\]). Its type is `()`.

**\[CTRL-4\]** `for p in e B` has the meaning of:

```brio
var it = e.into_iter()
loop {
    let x = it.next() else { break }
    let p = x
    B
}
```

where `it` and `x` are fresh names. The pattern p SHALL be irrefutable. Its type is `()`.

**\[CTRL-5\]** `break` and `continue` apply to the innermost enclosing loop, or to the loop named by their label. `break v` is permitted only in `loop`. Labels occupy their own namespace and SHALL NOT repeat within one function.

**\[CTRL-6\]** Loop specifications (`invariant`, `decreases`, `bounded`) are defined in 11.5.

### 7.10 Returns and diverging expressions

**\[CTRL-7\]** `return e` ends the innermost enclosing function or closure with value e. `return` alone is `return ()`.

**\[CTRL-8\]** `fail e` is permitted only in a function or closure whose return type is `T ! E`. It is equivalent to `return .Err(E(e))`, using `From` when the type of e is not E.

**\[CTRL-9\]** `return`, `fail`, `break`, `continue`, a `loop` with no `break`, and a call to a function whose return type is `Never` have type `Never`. The core function `unreachable(message: Str) -> Never` traps; in prove mode, every call to it SHALL be proven unreachable.

### 7.11 Closures

*Syntax:*

```text
Closure ::= "move"? "|" (ClosureParam ("," ClosureParam)*)? "|" ("->" Type)? (Expr | Block)
ClosureParam ::= Pattern (":" Mode? Type)?
```

**\[CTRL-10\]** A closure is an anonymous function value that may capture bindings from its environment. Capture rules are in 9.9. A closure's body is a value block (\[EXPR-11\]); `return` inside it returns from the closure.

### 7.12 String interpolation and formatting

*Syntax (inside `{…}` of a string literal):*

```text
Interpolation ::= Expr (":" FormatSpec)?
FormatSpec    ::= (Fill? Align)? Sign? "0"? Width? ("." Precision)? "?"?
Align         ::= "<" | "^" | ">"
```

**\[FMT-1\]** Each interpolated expression is formatted with `Display`, or with `Debug` when the format specification ends in `?`. Fill, alignment, sign, width and precision have the meanings they have in Rust's formatting syntax.

**\[FMT-2\]** A value of type `Formatted` converts implicitly to `String` where a `String` is required; this conversion allocates. A `Formatted` value can also be written to any core `Writer` without allocation.

**\[FMT-3\]** Interpolated expressions are evaluated in the order written, when the literal is evaluated.

## 8 Patterns and matching

### 8.1 Pattern syntax

```text
Pattern        ::= PatternAlt ("|" PatternAlt)*
PatternAlt     ::= PatternBase ("as" IDENT)?
PatternBase    ::= "_" | LiteralPattern | RangePattern | BindingPattern
                 | TuplePattern | VariantPattern | RecordPattern | SlicePattern
                 | "(" Pattern ")"
BindingPattern ::= "var"? IDENT
LiteralPattern ::= "-"? INTEGER UnitAnnot? | CHAR | STRING | "true" | "false" | QualifiedConst
RangePattern   ::= RangeBound ("..<" | "..=") RangeBound
RangeBound     ::= "-"? (INTEGER | FLOAT) UnitAnnot? | CHAR | QualifiedConst
TuplePattern   ::= "(" Pattern ("," Pattern)+ ("," "..")? ","? ")"
VariantPattern ::= ("." IDENT | TypePath "." IDENT) ("(" FieldPatterns ")")?
RecordPattern  ::= TypePath "(" FieldPatterns ")"
FieldPatterns  ::= ".." | FieldPattern ("," FieldPattern)* ("," "..")?
FieldPattern   ::= IDENT ":" Pattern | IDENT | Pattern
SlicePattern   ::= "[" (SliceElem ("," SliceElem)*)? "]"
SliceElem      ::= Pattern | ".." ("as" IDENT)?
QualifiedConst ::= TypePath "." IDENT
```

**\[PAT-1\]** A bare identifier in a pattern is always a *binding*. A constant, associated constant or nullary variant is matched only by a qualified path (`Limits.MAX`) or an implicit member (`.None`). An unqualified identifier never refers to an existing constant.

*Note:* This removes the ambiguity, present in several languages, where a misspelled constant silently becomes a catch-all binding.

**\[PAT-2\]** In a variant or record pattern with labeled fields, `label: P` matches the field named label against P, and a bare `label` binds the field to a new binding of the same name. Positional fields are matched by position. A trailing `..` ignores the remaining fields; without it, every field SHALL be mentioned.

**\[PAT-3\]** Floating-point values SHALL NOT be matched by literal patterns. They MAY be matched by range patterns.

**\[PAT-4\]** In a range pattern, the lower bound SHALL NOT exceed the upper bound, and a `..<` range SHALL NOT be empty.

**\[PAT-5\]** `P as x` matches P and additionally binds the whole matched value to x.

**\[PAT-6\]** Each alternative of an or-pattern SHALL bind the same set of names, with the same types and binding modes.

### 8.2 Refutability

**\[PAT-7\]** A pattern is *irrefutable* if it matches every value of its type. Wildcards, bindings, tuple patterns of irrefutable patterns, and record patterns of irrefutable patterns are irrefutable, as are variant patterns of enums with exactly one variant. All other patterns are *refutable*.

**\[PAT-8\]** Irrefutable patterns are required in `let` without `else`, in `for` loops, in function and closure parameters, and in `forall` test parameters.

### 8.3 Binding modes

**\[PAT-9\]** The *scrutinee* of a `match` or `is` is the expression being matched. How bindings are created depends on how the scrutinee is written:

| Scrutinee | Bindings to `Copy` parts | Bindings to other parts | Scrutinee afterwards |
| --- | --- | --- | --- |
| A place `x` | Copies | Shared borrows | Unchanged and usable |
| `mut x` (x a mutable place) | Exclusive borrows | Exclusive borrows | Usable after the match |
| `move x` | Copies | Moved out of x | Moved-from parts unusable |
| A value expression | Copies | Moved out of the temporary | Not applicable |

**\[PAT-10\]** A `var` binding creates a mutable binding. It is permitted only where the binding would copy or move, not where it would borrow.

**\[PAT-17\]** A binding created as a borrow is an *alias* for the matched place and has the type of that place, not a reference type. Reading the binding reads the place; for an exclusive borrow, assigning to the binding writes the place.

**\[PAT-11\]** In `let`, the initializer is evaluated as a value: a place initializer is moved, or copied if its type is `Copy`, and the value is then destructured.

*Example:*

```brio
match mut pump.state {
    .Running(rate) => rate = min(rate, MAX_RATE)    // rate: exclusive borrow
    _ => {}
}
```

### 8.4 Match expressions

*Syntax:*

```text
MatchExpr ::= "match" ("mut" | "move")? Expr "{" MatchArm (SEP MatchArm)* SEP? "}"
MatchArm  ::= Pattern ("if" Expr)? "=>" (Expr | Block)
```

**\[PAT-12\]** The scrutinee is evaluated exactly once. Arms are tried in order. The first arm whose pattern matches and whose guard, if any, evaluates to `true` is selected, and its expression is evaluated. The value of the `match` is the value of the selected arm; all arms SHALL have a common type.

**\[PAT-13\]** During evaluation of a guard, all bindings of the arm are shared borrows, whatever the binding mode of the arm.

**\[PAT-14\]** A `match` SHALL be *exhaustive*: every value of the scrutinee's type SHALL be matched by at least one arm whose guard is absent. Exhaustiveness takes into account enum variants, `Bool`, the full domain of integer and `Char` types, and the ranges of range types. Guards are not considered.

**\[PAT-15\]** A match on a non-frozen enum declared in another package SHALL include a wildcard arm (17.6).

**\[PAT-16\]** An arm that can never be selected, because earlier arms without guards already match every value its pattern matches, is ill-formed.

*Example:*

```brio
let action = match alarm {
    .None                     => Action.Continue
    .Low(r) if r < 30         => Action.Page(.Urgent)
    .Low(_) | .High(_)        => Action.Page(.Normal)
    .SensorFault(code, ..)    => Action.Service(code)
}
```

## 9 Ownership, borrowing and parameter modes

### 9.1 Ownership and moves

**\[OWN-1\]** Every value has exactly one owner at a time. When an owner goes out of scope while still owning a value, the value is dropped (9.8).

**\[OWN-2\]** Using a place as a value *moves* the value out of the place, unless the type implements `Copy`, in which case it is copied. After a move, the place is *moved-from*.

**\[OWN-3\]** A moved-from place SHALL NOT be read, borrowed or moved again until it is assigned a new value. If a place is moved on some paths but not others, it is treated as moved-from after the paths join.

**\[OWN-4\]** A field MAY be moved out of a local binding of record or tuple type (a *partial move*). The remaining fields stay usable; the whole value SHALL NOT be used until the moved field is reassigned. Partial moves out of a type that implements `Drop` are ill-formed.

**\[OWN-5\]** Every binding and `out` parameter SHALL be definitely initialized before it is read.

### 9.2 Parameter modes

**\[OWN-6\]** Each parameter has one of four modes:

| Mode | Declared as | Callee may | Caller writes | Rust equivalent |
| --- | --- | --- | --- | --- |
| Default | `x: T` | Read | `f(a)` | `&T` |
| `inout` | `x: inout T` | Read and write | `f(mut a)` | `&mut T` |
| `take` | `x: take T` | Own, and therefore read, write, move and drop | `f(move a)` | `T` |
| `out` | `x: out T` | Write, then read | `f(out a)` | `&mut MaybeUninit<T>` |

**\[OWN-7\]** In the callee, every parameter is a place of type T, not a reference. A default-mode parameter is an immutable place. `inout` and `out` parameters are mutable places whose reads and writes act on the caller's argument. A `take` parameter is an owned local that MAY be rebound as mutable with `var`.

**\[OWN-8\]** In default mode, neither the callee nor anything it calls may mutate the argument or anything reachable through it, except through the interior-mutability types of 9.7.

**\[OWN-9\]** A parameter mode states what the callee may do with an argument, not how it is passed. For every mode, the implementation chooses the mechanism: in registers, by copying, or by address. `inout` and `out` arguments MAY be passed by copying in and copying back, because exclusivity (\[OWN-17\]) guarantees that no other code can observe the place during the call. The choice SHALL NOT change the behavior of any conforming program. The implementation SHALL pass an argument by address when:

1. the function's result borrows from the parameter (9.6);
2. the parameter's type contains an interior-mutability type (\[OWN-35\]);
3. the parameter's type is dynamically sized, or its size is not known where the call is compiled, for example under dictionary passing (12.6);
4. the call crosses a foreign boundary whose calling convention requires it (Clauses 15, 18 and 25).

**\[OWN-10\]** The argument for an `out` parameter SHALL be a mutable place. If that place is initialized before the call, its value is dropped before the call begins. The callee SHALL NOT read an `out` parameter before writing it, and SHALL definitely initialize it on every path that returns, including paths that `fail`. After the call returns, the place is initialized.

**\[OWN-11\]** A method's `self` parameter takes a mode in the same way: `self`, `inout self`, `take self` or `out self`. Receivers are passed without call-site markers (\[FN-7\]); an `inout self` method SHALL be called only on a mutable place.

### 9.3 Call-site markers

**\[OWN-12\]** An argument for an `inout` parameter SHALL be written `mut p`, where p is a mutable place. An argument for an `out` parameter SHALL be written `out p`. These markers SHALL NOT be used for arguments in any other mode.

**\[OWN-13\]** An argument for a `take` parameter that is a place of a non-`Copy` type SHALL be written `move p`. `move` MAY be written for a `Copy` type and is then a copy. A value expression passed to a `take` parameter needs no marker.

**\[OWN-14\]** The markers make every mutation and every transfer of ownership visible at the call site.

*Example:*

```brio
fn checksum(buf: Buffer) -> U32
fn normalize(buf: inout Buffer)
fn archive(buf: take Buffer)
fn fill(buf: out Buffer)

var buf: Buffer
fill(out buf)
normalize(mut buf)
let sum = checksum(buf)
archive(move buf)
let again = checksum(buf)   // ill-formed: buf was moved
```

### 9.4 Exclusivity

**\[OWN-15\]** A *borrow* of a place is *shared* or *exclusive*. Default-mode arguments, `ref` expressions and shared pattern bindings create shared borrows. `inout` arguments, `ref mut` expressions, `inout self` receivers and exclusive pattern bindings create exclusive borrows.

**\[OWN-16\]** A borrow is *live* from the point it is created until the last point at which it, or a value derived from it, is used.

**\[OWN-17\]** While an exclusive borrow of a place is live, that place, and every place that overlaps it, SHALL NOT be read, written, moved or borrowed except through that borrow. While a shared borrow of a place is live, that place and every overlapping place SHALL NOT be written, moved or exclusively borrowed.

**\[OWN-18\]** Two places *overlap* if one is a prefix of the other or they are the same place. Distinct named fields of the same record, and distinct components of the same tuple, do not overlap. Elements `a[i]` and `a[j]` are treated as overlapping.

*Example:* `transfer(mut a.left, mut a.right)` is permitted; `swap(mut a[i], mut a[j])` is ill-formed and is written `a.swap(i, j)`.

**\[OWN-19\]** An exclusive borrow created for a method receiver or a `mut` argument is *reserved* when evaluated and becomes live when the call begins. Shared reads of the place during the reservation are permitted. Thus `list.push(list.len())` is well-formed.

### 9.5 Reference values

**\[OWN-20\]** `ref p` creates a value of type `ref T` that borrows the place p shared. `ref mut p` creates a value of type `ref mut T` that borrows the mutable place p exclusively. Reference values MAY be stored in bindings, collections and, outside the `critical` profile, record fields.

**\[OWN-21\]** A value of type `ref T` or `ref mut T` behaves as its target for field access, method calls, comparisons and interpolation. `r.*` denotes the target place explicitly; assignment through an exclusive reference is written `r.* = v`.

### 9.6 Origins

**\[OWN-22\]** Every reference type, and every type with origin parameters, has one or more *origins*: compile-time names for the set of borrows its values may depend on. Origins replace Rust's lifetime annotations.

**\[OWN-23\]** A function whose return type contains origins declares where they come from with `from`: `-> Str from a | b` states that the result borrows from parameters a and b and from nothing else. Every parameter named in `from` SHALL be in default or `inout` mode, or have a type that itself contains origins.

**\[OWN-34\]** In a return type, `T from …` where T contains no origins, including a dynamically sized T such as `Str` or `[U8]`, denotes `ref T from …`. The returned expression SHALL then be a place, and it is borrowed implicitly. This is the only way a dynamically sized type appears in a return type (\[TYPE-21\]).

**\[OWN-35\]** A default-mode argument whose type contains an interior-mutability type (9.7), directly or through its fields, SHALL be passed by address, so that mutation through it acts on the caller's value. This refines \[OWN-9\]: the passing mechanism can never change a program's behavior.

**\[OWN-37\]** The address of a parameter is unspecified. A `Ptr` or `MutPtr` created from a parameter is valid only until the call returns, and comparing it with a pointer created from the caller's argument has an unspecified result. Only `unsafe` code can dereference such a pointer (\[UNS-4\]), and \[TRUST-8\] warns about comparisons that depend on it, so the passing mechanism never affects safe code.

**\[OWN-38\]** *Stable calling convention.* For a function that can be called from outside its package, and for every call through a function value or dynamic value, the passing mechanism of each parameter and of the result SHALL be determined by the declared signature and the target alone, following the *Brio ABI* that the implementation documents for that target. Separately compiled packages, including those distributed without source (\[PKG-21\]), therefore always agree on how arguments are passed. For calls whose callee and caller are compiled together, the implementation MAY choose a different mechanism, provided every entry point reachable from outside the package still follows the Brio ABI. The Brio ABI of a target changes only in a new major version, recorded in build metadata (\[PKG-22\], \[PKG-24\]).

**\[OWN-40\]** *Freeze.* A type is *freeze* unless it contains, directly or through any of its fields, an interior-mutability type (\[OWN-35\]). A type parameter is treated as not freeze unless it is bounded by the core trait `Freeze`. Only arguments of freeze types MAY be passed by copying in default mode (\[OWN-9\]).

**\[OWN-41\]** *No implied bounds.* Every relationship between origins that a function relies on SHALL be stated in its signature. The compiler SHALL NOT infer such relationships from the types of parameters or results. Well-formedness of origins is checked wherever a function is converted to a function type and wherever a generic function with origin parameters is used. This rule excludes by construction the class of unsoundness caused by implied bounds in other languages.

**\[OWN-42\]** *Aliasing model.* Which accesses through references and pointers are permitted is defined in Brio Core by a tree-structured aliasing model in the style of Tree Borrows. Safe code cannot violate it. `unsafe` code that violates it has undefined behavior, which `brio interp` detects (\[MODEL-6\]). The mechanization of \[MODEL-4\] SHALL prove that every non-aliasing fact the implementation gives its optimizer follows from this model (\[PERF-3\]).

**\[OWN-43\]** Specialization of generic items is not part of the language. Implementations of `Deref` and `DerefMut` for the core pointer and pinning types are defined only in the core library.

**\[OWN-36\]** At the end of each test, `brio test` SHALL report any `Rc` or `Arc` values that are unreachable but not freed, which indicates a reference cycle, with the allocation site of each. The core types `Weak` and `ArcWeak` break such cycles.

**\[OWN-24\]** When `from` is omitted and the return type contains origins, the origins are inferred as follows:

1. for a method whose receiver is `self` or `inout self`, the result borrows from the receiver;
2. otherwise, if exactly one parameter is in default or `inout` mode, the result borrows from it;
3. otherwise, the declaration is ill-formed and `from` SHALL be written.

**\[OWN-25\]** A record or enum that contains references declares origin parameters: `record View[origin o] { data: ref [U8] from o }`. Within function bodies, origin arguments are always inferred; in signatures they are bound by `from` or by \[OWN-24\].

**\[OWN-26\]** The origin `static` denotes borrows that are valid for the whole execution, such as references to `static` items and string literals.

**\[OWN-27\]** An implementation SHALL reject every program in which a reference could be used after its referent is dropped, moved or reassigned, or in which \[OWN-17\] could be violated. An implementation SHALL accept every program that the Rust 2024 edition borrow checker would accept for the corresponding Rust program under the mapping of 18.4. An implementation MAY accept additional programs only if it can show them sound.

*Example:*

```brio
fn longest(a: Str, b: Str) -> Str from a | b =
    if a.len() >= b.len() { a } else { b }

fn header(src: [U8]) -> View from src {
    return View(data: ref src[0 ..< 16])
}
```

### 9.7 Interior mutability

**\[OWN-28\]** The core types `Cell[T]`, `RefCell[T]`, `Mutex[T]`, `RwLock[T]` and the atomic integer types permit mutation through a shared borrow, with the semantics of their Rust counterparts. A `RefCell` borrow conflict traps. `RefCell` is forbidden under the `critical` profile.

### 9.8 Destruction

**\[OWN-29\]** When a scope ends, the values still owned by its bindings are dropped in the reverse order of declaration. Temporaries are dropped at the end of the statement that created them, in reverse order of creation.

**\[OWN-30\]** Dropping a value of a type that implements `Drop` first calls its `drop(inout self)` method, then drops its fields in declaration order. Values of other types drop their fields in declaration order. Moved-from places are not dropped.

**\[OWN-31\]** `Drop.drop` SHALL NOT be called explicitly. The core function `drop[T](x: take T)` ends ownership early.

### 9.9 Closure captures

**\[OWN-32\]** A closure without `move` captures each binding it uses by shared borrow, or by exclusive borrow if its body mutates the binding. A `move` closure captures each binding it uses by moving it, or copying it if its type is `Copy`.

**\[OWN-33\]** A closure implements `Fn` if it only reads its captures, `FnMut` if it mutates them, and `FnOnce` if it moves out of them, following the rules of the Rust Reference. A closure that captures by borrow has origins, and SHALL NOT outlive the captured bindings.

## 10 Errors, traps and failure handling

Brio distinguishes two kinds of failure. *Errors* are expected conditions, such as a sensor timeout, represented as values of type `T ! E` and handled by the program. *Traps* are violations of the program's own rules, such as a failed contract or an out-of-bounds index; they mean the program's state can no longer be trusted, and they stop the program.

### 10.1 Error values

**\[ERR-1\]** Any type MAY be used as an error type. Error types SHOULD be enums that implement the core trait `Error`, which requires `Debug` and `Display` and provides `source(self) -> (ref any Error)?`.

**\[ERR-2\]** Functions report errors only by returning `.Err`. Brio has no exceptions and no unwinding.

### 10.2 The try operator

**\[ERR-3\]** `try e` is permitted only inside a function or closure whose return type is a `Result` or an `Option`.

**\[ERR-4\]** If e has type `T ! E1` and the enclosing function returns `U ! E2`, then `try e` evaluates e; if the result is `.Ok(v)` the value of `try e` is v, and if it is `.Err(x)` the function returns `.Err(x)` when E1 is E2, or `.Err(E2(x))` using `From` otherwise. If no such `From` implementation exists, the expression is ill-formed.

**\[ERR-5\]** If e has type `T?` and the enclosing function returns `U?`, then `try e` evaluates to the payload of `.Some`, or returns `.None` from the function.

**\[ERR-6\]** Applying `try` to an `Option` inside a function that returns a `Result`, or the reverse, is ill-formed. The conversion is written explicitly, for example `try reading.ok_or(.Missing)`.

**\[ERR-7\]** `try` SHALL NOT appear in a contract expression.

**\[ERR-8\]** *No silent crashes.* `Option` and `Result` provide no operation that traps without a stated reason. The core method `x.expect(reason: Str)` traps with the reason if x is `.None` or `.Err`; in prove mode, success SHALL be proven. Every `expect`, like every other explicit trap, is a *trap site* listed by \[TOOL-33\].

*Example:*

```brio
fn read_sensor(port: inout Port) -> Reading ! SensorError {
    let raw = try port.read_frame_within(100<ms>)   // bounded wait; IoError converted by From
    let reading = try Reading.decode(raw)
    if reading.rate > HeartRate.max { fail .Implausible(reading.rate) }
    return reading
}
```

### 10.3 Trap conditions

**\[TRAP-1\]** A trap occurs, in code whose verification mode is `check`, when:

1. integer arithmetic overflows (\[TYPE-4\]);
2. a division or remainder by zero, or `MIN / -1`, is evaluated (\[TYPE-6\]);
3. a shift amount is out of range (\[TYPE-7\]);
4. an index or slice range is out of bounds (\[EXPR-20\]);
5. a range check or checked conversion fails (\[RNG-5\], \[TYPE-26\], \[UNIT-8\]);
6. a `requires`, `ensures`, `invariant`, `decreases` or `assert` condition evaluates to `false` (Clause 11);
7. `unreachable` or the core function `trap(reason: Str) -> Never` is called;
8. a `RefCell` borrow conflicts (\[OWN-28\]);
9. a `bounded` loop without an `else` block exceeds its bound (\[CON-15\]).

**\[TRAP-2\]** A trap also occurs, in every verification mode, when:

1. the stack of a task is exhausted;
2. a heap allocation fails;
3. a Rust panic reaches a bridge boundary (18.8).

**\[TRAP-3\]** An implementation SHALL detect stack exhaustion and turn it into a trap. Stack exhaustion SHALL NOT cause undefined behavior.

**\[TRAP-4\]** In prove mode, the conditions in \[TRAP-1\] SHALL be proven never to occur, so conforming code in prove mode cannot trap for those reasons.

**\[TRAP-10\]** *Restricted trap handler.* The trap handler runs on a stack reserved at startup. It SHALL NOT allocate, acquire locks or wait without a deadline; it receives no references into the frames that trapped; and it holds only the capabilities registered for it at startup, for example an output that puts a device into its safe state. In programs containing `critical` modules, the safe state that the handler establishes SHALL be stated, and its reachability within a stated time after any trap is part of the timing analysis of Clause 34.

**\[TRAP-11\]** Rust code in a bridge SHALL be compiled so that a Rust panic can be caught at the bridge, and C++ code so that exceptions are caught in a C++ shim; each is converted to a trap. Anything that cannot be caught at a boundary traps.

### 10.4 What happens when a trap occurs

**\[TRAP-5\]** When a trap occurs:

1. no further user code runs in the trapping task, except the trap handler;
2. the trap handler runs on the trapping thread, with a `TrapInfo` describing the trap;
3. when the handler ends, or if there is none, the process terminates abnormally with an implementation-defined status.

Destructors are not run, and other tasks do not continue. A trap cannot be caught or recovered from.

*Note:* Brio stops instead of unwinding because, after a trap, the program's invariants may not hold, and running further code, including destructors, could make matters worse.

**\[TRAP-6\]** An executable MAY declare one *trap handler*: a function marked `@trap_handler` with signature `fn name(info: TrapInfo) -> Never`. Declaring more than one is ill-formed.

**\[TRAP-7\]** The trap handler, and every function it calls, SHALL NOT allocate, SHALL NOT acquire a lock, SHALL NOT `await`, and SHALL NOT access `Rc`, `RefCell` or `static var` state. It SHOULD bring the system into a safe state, record diagnostic information, and end by calling the core function `terminate(status: I32) -> Never`.

**\[TRAP-8\]** A trap that occurs while the trap handler is running terminates the process immediately.

**\[TRAP-9\]** `TrapInfo` provides at least: the kind of trap, a message, the source location (file, line and column), the identifier of the rule that was violated where one applies (for example `TYPE-4`), and the identifier of the trapping task. Messages longer than an implementation-defined limit are truncated; building a `TrapInfo` SHALL NOT allocate.

**\[TRAP-10\]** Without a declared handler, the implementation's default handler writes the `TrapInfo` to the standard error stream, if one exists, and then terminates.

**\[TRAP-11\]** An executable containing a module with the `critical` profile SHALL declare a trap handler.

*Example:*

```brio
@trap_handler
fn fail_safe(info: TrapInfo) -> Never {
    pump_hw.close_valves()
    pump_hw.sound_alarm()
    black_box.record(info)
    terminate(70)
}
```

## 11 Contracts and verification

### 11.1 Contract forms

*Syntax:*

```text
Contract   ::= "requires" Expr
             | "ensures" ("on" "fail")? Expr
             | "decreases" Expr ("," Expr)*
Quantified ::= ("forall" | "exists") Pattern "in" Expr ":" Expr
```

**\[CON-1\]** The contract forms are: preconditions (`requires`), postconditions (`ensures`, `ensures on fail`), function variants (`decreases` on a function), type invariants (`invariant` in a record, \[ITEM-14\]), loop invariants, loop variants and loop bounds (11.5), and assertions (`assert`, \[STMT-6\]).

**\[CON-2\]** Several clauses of the same form are combined by conjunction and evaluated in the order written.

### 11.2 Contract expressions and pure functions

**\[CON-3\]** A contract expression SHALL have type `Bool` and SHALL be *effect-free*: its evaluation SHALL NOT write any place that exists outside the evaluation, perform input or output, acquire a lock, spawn or await a task, or access a `static var`. It MAY call only functions marked `@pure`.

**\[CON-4\]** A function marked `@pure` SHALL have only default-mode and `take` parameters, and its body SHALL satisfy the restrictions of \[CON-3\]. A pure function MAY allocate memory that it releases before returning, and MAY trap. Pure functions MAY be called from ordinary code.

**\[CON-5\]** Inside contract expressions, and only there:

| Construct | Meaning | Permitted in |
| --- | --- | --- |
| `result` | The returned value (the `.Ok` payload for a `Result` function) | `ensures` |
| `error` | The returned error | `ensures on fail` |
| `old(e)` | The value of e on entry to the function | `ensures`, `ensures on fail` |
| `forall p in c: e` | e holds for every element of c | Every contract, `assert` |
| `exists p in c: e` | e holds for at least one element of c | Every contract, `assert` |

**\[CON-6\]** In `old(e)`, e SHALL have a type that implements `Clone`; in check mode its value is cloned on entry. A quantifier's collection c SHALL be a range or implement `IntoIterator` with a finite number of elements. A quantified expression extends as far to the right as possible.

**\[CON-7\]** A contract SHALL NOT refer to a `take` parameter after it may have been moved, nor to an `out` parameter in a precondition.

### 11.3 Preconditions and postconditions

**\[CON-8\]** A precondition states an obligation of every caller. In check mode it is evaluated in the callee on entry, after parameters are bound, so that callers written in any language are checked. In prove mode, it SHALL be proven at every call site, and the callee MAY assume it.

**\[CON-9\]** `ensures P` states an obligation of the callee on every normal return. For a function returning `T ! E`, it applies only to `.Ok` returns, and `result` denotes the `.Ok` payload. `ensures on fail P` applies only to `.Err` returns, and `error` denotes the error; it is ill-formed in a function that does not return a `Result`.

**\[CON-10\]** In a postcondition, a default-mode parameter denotes its (unchanged) value, and an `inout` or `out` parameter denotes its value at return. `old(x)` denotes a parameter's value on entry.

*Example:*

```brio
fn set_rate(pump: inout Controller, rate: MlPerHour) -> () ! PumpError
    requires rate <= pump.max_rate
    ensures  pump.rate == rate
    ensures on fail pump.rate == old(pump.rate)
```

### 11.4 Type invariants

**\[CON-11\]** A type invariant SHALL hold:

1. for every value produced by the record's memberwise initializer;
2. for every value of the type returned by a function declared in the record's module;
3. for every value of the type in an `inout`, `out` or `take` parameter (including `self`) when a function declared in the record's module and visible outside it returns.

**\[CON-12\]** Code outside the record's module MAY assume the invariant holds for every value of the type. Code inside the module MAY break it temporarily between the points listed in \[CON-11\].

### 11.5 Loop contracts

**\[CON-13\]** A loop `invariant P` SHALL hold at the start of every iteration and when the loop exits normally.

**\[CON-14\]** A loop or function variant `decreases e1, …, en` SHALL consist of integer-valued expressions. At the end of each iteration, or at each recursive call, the tuple of values SHALL be lexicographically smaller than at the start of the iteration or the call, and every component SHALL be non-negative. A variant therefore proves termination.

**\[CON-15\]** `bounded n` evaluates n, of type `USize`, once when the loop is entered. When the loop would begin iteration n + 1:

1. if `bounded n else B` was written, the loop exits and B is executed; B has type `()` or `Never`;
2. otherwise the loop has exceeded its bound, which traps in check mode and SHALL be proven impossible in prove mode.

*Example:*

```brio
while pressure.rising()
    bounded 1_000 else { alarms.raise(.PressureTimeout) }
{
    valve.step_down()
}

for i in 0 ..< n
    invariant total == sum(doses[0 ..< i])
{
    total += doses[i]
}
```

### 11.6 Verification modes

**\[CON-16\]** Each item is compiled in one *verification mode*, which governs its contracts and assertions. Implicit safety checks (overflow, division, shifts, bounds, range checks and checked conversions) are always proven or checked, whatever the mode (\[CON-27\]):

| Mode | Meaning |
| --- | --- |
| `prove` | Every obligation SHALL be discharged at compile time; nothing is checked at runtime |
| `check` | Every obligation is evaluated at runtime and traps if violated |
| `observe` | Every obligation is evaluated at runtime and a violation is reported as a telemetry event, without trapping (\[OBS-11\]) |
| `assume` | Obligations are neither proven nor evaluated; they are used as proof hypotheses by other code |

**\[CON-17\]** The mode of an item is the first of the following that applies: the item's own `@verify` attribute; the module's `@verify` attribute; the rules of its profile (16.2); the package manifest's `[verify]` setting for the build configuration; `check`.

**\[CON-18\]** `@verify(assume, reason)` SHALL give a non-empty justification string. Assumed obligations SHALL NOT be used by the optimizer, so an assumption that is false does not cause undefined behavior; it only invalidates proofs that relied on it. Every assumption SHALL be listed in the `brio ship` report (19.6).

**\[CON-19\]** A package manifest MAY set contracts of non-public items to `assume` for release builds. Preconditions of `public` functions SHALL remain in `check` or `prove` mode unless an item-level `@verify(assume, reason)` says otherwise.

**\[CON-20\]** If every contract and implicit check holds, a program's observable behavior is the same in every verification mode. Changing a mode can only add or remove traps.

### 11.7 Proof

**\[CON-21\]** In prove mode, the implementation SHALL generate proof obligations for: preconditions at each call site, postconditions at each return, type invariants at the points of \[CON-11\], loop invariants on entry and preservation, variants, loop bounds, and every trap condition of \[TRAP-1\].

**\[CON-22\]** Verification is *modular*. At a call, the prover SHALL use the callee's contracts, the types and ranges of its parameters and result, and its type invariants, and SHALL NOT depend on the callee's body. The bodies of `@pure` functions MAY additionally be used.

**\[CON-23\]** The verification procedure SHALL be sound: an obligation reported as proven SHALL hold in every execution, assuming the contracts of `assume`-mode code, the correctness of bridged Rust and C code, and the correctness of the hardware.

**\[CON-24\]** An obligation that the implementation cannot prove, whether because it is false or because the procedure is inconclusive, makes the program ill-formed. The diagnostic SHOULD include a counterexample when one is found.

**\[CON-25\]** Proof results SHALL be deterministic. Resource limits for the prover SHALL be expressed in deterministic units, such as solver steps, and not in wall-clock time. Limits are set in the package manifest and are part of the build inputs.

**\[CON-26\]** Proof obligations are translated to SMT-LIB 2.6. In `critical` builds, every obligation SHALL be discharged by a solver that produces a proof certificate in a standard format such as Alethe, and the certificate SHALL be validated by an independent proof checker; agreement between solvers is not sufficient. In other builds, agreement of two independent solvers MAY be used instead. In addition:

1. obligations for the trap conditions of \[TRAP-1\] SHALL be generated as separate queries whose hypotheses exclude every fact from `assume` and `observe` mode, and the checker SHALL confirm from the certificate that no such fact was used;
2. for every function, the implementation SHALL check that its preconditions and the assumptions in scope are satisfiable, and SHALL report a function whose proofs hold only vacuously;
3. the generator of verification conditions is part of the trusted base (\[TRUST-12\]) unless it is covered by \[MODEL-4\] or its output is validated for each build.

The solvers, checker, axioms and versions used are recorded in the assurance ledger.

### 11.8 Safety checks, assumptions and entry checks

**\[CON-27\]** *Safety checks are never assumed.* The implicit checks of \[CON-16\] (integer overflow, division, shifts, bounds, range and validity checks, checked conversions and `RefCell` borrows) are always in prove mode or check mode, whatever mode applies to the item's contracts. The modes `assume` and `observe` apply only to contracts and assertions written by the programmer.

**\[CON-28\]** A safety check MAY be omitted only when it is proven from facts that are themselves proven or checked at runtime. Assumed contracts, bridge contracts (\[RUST-15\]), API-notes contracts (\[FFI-12\]) and `@bounded` claims SHALL NOT be used to omit a safety check. A false assumption can therefore make a functional property fail, but can never cause undefined behavior.

**\[CON-29\]** *Checked entry.* When a function in prove mode can be called from code that does not prove its preconditions (code in check, observe or assume mode, another package, foreign code, or a call through a function value or dynamic value), its preconditions SHALL be evaluated on entry for those calls. The implementation MAY provide a separate entry without these checks for call sites that prove the preconditions. A proven function can therefore rely on its preconditions whoever calls it.

**\[CON-40\]** *Optimizer facts.* Every fact an implementation gives its optimizer, such as value ranges, validity, non-null, alignment, dereferenceability, absence of aliasing, termination and absence of side effects, SHALL be one of: a consequence of the rules of this specification for safe code, proven in the mechanization of \[MODEL-4\]; a proven obligation that used no assumed or observed fact (\[CON-26\]); or the result of a check at a trust boundary (\[CON-41\]). Facts from `assume` mode, `@bounded` and other claims about foreign code, API notes and bridge contracts SHALL NOT be given to the optimizer. A false assumption therefore cannot cause undefined behavior.

**\[CON-41\]** *Trust boundaries.* A value that enters Brio code from foreign code, a Rust bridge, Python, a WebAssembly component, device memory (`Volatile`), decoding, or an `unsafe` dereference SHALL be checked against every validity constraint of its type (range, enum discriminant, niche, `Char`, `Str` encoding, alignment, non-null) before it is used as that type, unless the check is proven redundant under \[CON-40\]. \[RUST-10\] and \[FFI-5\] are instances of this rule. Checks of large device or network buffers MAY be performed once, producing a value whose type records that it has been checked.

**\[CON-42\]** *Hardened builds.* In `critical` modules built with `hardened = true` in `[verify]`, facts about memory that is not local to a function (statics, `Shared` values, `Volatile` memory and memory reachable from foreign code) SHALL be re-checked when loaded instead of being relied on, so that corruption by a hardware fault traps instead of causing undefined behavior.

**\[CON-43\]** *Checked return.* When a function in prove mode calls a function whose postconditions are not proven, such as code in check, observe or assume mode, foreign code or a bridge claim, those postconditions SHALL be evaluated when the call returns, unless the callee is listed in the assurance ledger as trusted.

**\[CON-44\]** Code evaluated only because of observe mode (\[OBS-11\]) SHALL have no effect other than reporting, and SHALL NOT change the control flow of the program.

**\[CON-30\]** In `old(e)`, the type of e SHALL implement `Copy`, or implement `Clone` with a `@pure` implementation. Derived `Clone` implementations are `@pure`.

**\[CON-31\]** Contract expressions are evaluated with the ordinary semantics of Brio, including overflow checks. Every contract SHALL be well-defined: in prove mode, the absence of traps during its evaluation is itself an obligation; in check mode, a trap during its evaluation is reported as a violation of that contract.

## 12 Generics, traits and dispatch

### 12.1 Generic parameters

*Syntax:*

```text
GenericParams ::= "[" GenericParam ("," GenericParam)* ","? "]"
GenericParam  ::= IDENT (":" Bounds)? ("=" Type)?
                | "const" IDENT ":" Type ("=" ConstExpr)?
                | "unit" IDENT
                | "origin" IDENT
Bounds        ::= TypePath GenericArgs? ("+" TypePath GenericArgs?)*
WhereClause   ::= "where" WherePred ("," WherePred)*
WherePred     ::= Type ":" Bounds
GenericArgs   ::= "[" GenericArg ("," GenericArg)* ","? "]"
```

**\[GNR-1\]** A generic item is parameterized by type parameters, constant parameters (of integer, `Bool` or `Char` type), unit parameters (5.1) and origin parameters (9.6).

**\[GNR-2\]** Generic arguments MAY be written explicitly (`List[I32]`, `max[I32](a, b)`) or inferred at call sites. Constant arguments SHALL be constant expressions.

**\[GNR-3\]** Inside a generic item, a value of a type parameter T may be used only through the operations provided by T's bounds. A generic item is type-checked once, against its bounds, and not per instantiation.

### 12.2 Traits

*Syntax:*

```text
TraitDecl   ::= "trait" IDENT GenericParams? (":" Bounds)? WhereClause? "{" TraitMember* "}"
TraitMember ::= Attribute* ( FnDecl
                           | "type" IDENT (":" Bounds)? ("=" Type)?
                           | "const" IDENT ":" Type ("=" ConstExpr)? )
```

**\[TRAIT-1\]** A trait declares methods (with or without default bodies), associated types and associated constants. Inside a trait, `Self` denotes the implementing type, and `Self.Name` denotes an associated type.

**\[TRAIT-2\]** Bounds after `:` are *supertraits*: every type implementing the trait SHALL also implement them.

**\[TRAIT-3\]** Contracts declared on a trait method apply to every implementation. An implementation SHALL NOT add `requires` clauses. It MAY add `ensures` clauses, which strengthen the guarantee.

*Note:* This rule guarantees that code written against a trait remains correct for every implementation.

### 12.3 Implementations

*Syntax:*

```text
ImplDecl ::= "impl" GenericParams? (TypePath GenericArgs? "for")? Type WhereClause?
             "{" ImplMember* "}"
```

**\[TRAIT-4\]** `impl T { … }` is an *inherent implementation* that adds associated functions to T. It is permitted only in the package that declares T.

**\[TRAIT-5\]** `impl Tr for T { … }` implements trait Tr for type T. It SHALL provide every trait member that has no default, with a signature matching the trait's, including parameter modes.

**\[TRAIT-6\]** At most one implementation of a given trait may apply to a given type. Overlapping implementations are ill-formed.

**\[TRAIT-7\]** An implementation of Tr for T is permitted only if Tr or T is declared in the current package, following the orphan rules of the Rust Reference, with Rust crates and Brio packages both counting as foreign when they are not the current package.

### 12.4 Method resolution

**\[TRAIT-8\]** In `x.m(…)`, the method is found by trying, for the type of x and then for each type reached by dereferencing it through references and owning pointers: inherent methods first, then methods of traits that are in scope. If two traits in scope supply a method of the same name for the same type, the call is ill-formed and SHALL be written in qualified form `Tr.m(x, …)`.

**\[TRAIT-9\]** User-defined types cannot take part in automatic dereferencing. Only references and the core owning pointers of \[TYPE-19\] are dereferenced automatically.

### 12.5 Static and dynamic dispatch

**\[TRAIT-10\]** *Static use of a trait.* A trait's name, or a sum of bounds such as `Sensor + Display`, MAY be written as a type in a parameter type, including inside one (as in `[Sensor]` or `Sensor?`), in a return type, and in the type annotation of a local binding. A path in type position that resolves to a trait is a static use; one that resolves to a type denotes that type. In parameter types, each occurrence introduces its own anonymous type parameter with those bounds: `fn poll(s: Sensor)` means `fn poll[S: Sensor](s: S)`, and `fn f(a: Sensor, b: Sensor)` accepts two possibly different types. Parameters that must share one type are written with a named type parameter. In `[Sensor]`, all elements have the same type.

**\[TRAIT-11\]** In a return type, a trait name denotes one concrete type, chosen by the function body and hidden from callers, who may use only the bounds; every return path SHALL produce the same concrete type. In the annotation of a local binding, it denotes the inferred concrete type, used only through the bounds. A trait name SHALL NOT be used as a type in any other position, including record and enum fields, type aliases, statics, constants, and the parameter and result types of function types and `Fn` bounds. There the diagnostic SHALL suggest a named type parameter or `any`.

**\[TRAIT-12\]** *Dynamic values.* `any Bounds` is a sized type whose values are *dynamic values*: a value of some type implementing the bounds, together with a table of its methods. Method calls on a dynamic value are dispatched at runtime. `any` MAY appear wherever a type may, including fields, collections such as `[any Sensor]`, and return types.

**\[TRAIT-13\]** Only *dispatchable* traits may be used with `any`. A trait is dispatchable if every method has a receiver, has no type parameters, and does not use `Self` except behind a reference or as the receiver, and if every supertrait is dispatchable. A value of `any A` converts implicitly to `any B` when B is a supertrait of A.

**\[TRAIT-20\]** A dynamic value stores the contained value inline when its size and alignment fit the *inline capacity*, which is at least three pointer-sized words and is part of the Brio ABI (\[OWN-38\]); otherwise the contained value is stored on the heap. The size of `any Bounds` never depends on the contained type.

**\[TRAIT-21\]** Converting a value of type T to `any Bounds` allocates exactly when T does not fit the inline capacity, which is known at compile time at the conversion. An allocating conversion counts as heap allocation for `@no_alloc` (\[PROF-9\]) and is ill-formed in `@alloc(fallible)` modules (\[ALLOC-4\]), where the core function `try_any(x) -> any Bounds ! AllocError` is used instead. `brio perf` SHALL list every allocating conversion.

**\[TRAIT-22\]** Passing a value to a default-mode or `inout` parameter of type `any Bounds` SHALL NOT allocate: the callee receives a borrowed dynamic value referring to the caller's value. `ref any Bounds` and `ref mut any Bounds` likewise never allocate.

**\[TRAIT-23\]** Dropping a dynamic value drops the contained value and frees any heap storage. `any Bounds` implements `Send`, `Sync` or `Clone` only when the bounds include that trait.

### 12.6 Instantiation strategy

**\[TRAIT-14\]** The meaning of a generic item does not depend on how it is compiled. An implementation MAY compile each instantiation separately (monomorphization), compile one shared version that receives method tables at runtime (dictionary passing), or mix the two. Whether two instantiations have distinct addresses is unspecified.

*Note:* The reference toolchain uses dictionary passing in development builds, to compile faster, and monomorphization in release builds, to run faster (Clause 19).

### 12.7 Requested specializations

**\[TRAIT-15\]** `@specialize(A1, …, An)` on a generic function with one type parameter, or `@specialize((A1, B1), …)` for several, requests that the implementation compile the listed instantiations ahead of time and export them from the package's compiled code. A client calling the function with a listed instantiation SHALL use the precompiled version rather than a dictionary-passing version.

### 12.8 Callable traits

**\[TRAIT-16\]** The core traits `Fn`, `FnMut` and `FnOnce` are written with call syntax: `Fn(I32, inout Buffer) -> Bool`. Closures and functions implement them as in the Rust Reference. `fn(…) -> R` function types implement all three.

### 12.9 Deriving

**\[TRAIT-17\]** `@derive(Tr, …)` generates an implementation of each listed trait. The implementation provides derivers for `Copy`, `Clone`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `Hash`, `Debug` and `Default`, which operate field by field in declaration order (variant order, then fields, for enums), and, for distinct types, for the operator traits of 12.10. Other traits are derived by `comptime` derivers (13.5).

### 12.10 Operator traits

**\[TRAIT-18\]** For types other than the primitive types, these operators are defined by the corresponding core traits:

| Operators | Traits |
| --- | --- |
| `+` `-` `*` `/` `%` | `Add`, `Sub`, `Mul`, `Div`, `Rem` |
| unary `-`, `~` | `Neg`, `BitNot` |
| `&` `\|` `^` `<<` `>>` | `BitAnd`, `BitOr`, `BitXor`, `Shl`, `Shr` |
| `+=` and the other compound assignments | `AddAssign` and the corresponding traits, or else the operator trait followed by assignment |
| `==` `!=` / `<` `<=` `>` `>=` | `PartialEq` / `PartialOrd` |
| `a[i]` | `Index`, `IndexMut` |
| `x in c` | `Contains` |

**\[TRAIT-19\]** The wrapping and saturating operators, `and`, `or`, `not`, `implies`, `??`, `?.`, `.*`, `as` and `is` cannot be defined for user types.

## 13 Compile-time execution and metaprogramming

Brio replaces macros with ordinary Brio code that the compiler runs during compilation. Generated code is checked like hand-written code, and errors point at the code that requested the generation.

### 13.1 Constant expressions

**\[CT-1\]** A *constant expression* is an expression the implementation evaluates at compile time. Constant expressions are required in: `const` and `static` initializers, array lengths, constant generic arguments, default parameter values, range type bounds, unit scale factors, enum discriminants and attribute arguments.

**\[CT-2\]** An expression is a valid constant expression if its evaluation, performed as specified in 13.2, completes without error. It MAY use literals, constants, operators, control flow, local bindings, and calls to `comptime` functions and to ordinary functions whose bodies satisfy 13.2.

### 13.2 Compile-time evaluation

**\[CT-3\]** Compile-time evaluation SHALL produce the same results as evaluating the same code at runtime on the target, including the target's integer sizes and exact IEEE 754 floating-point results.

**\[CT-4\]** A trap during compile-time evaluation makes the program ill-formed, and the diagnostic SHALL identify both the trapping code and the constant expression that required it.

**\[CT-5\]** Compile-time evaluation SHALL be deterministic and hermetic. It SHALL NOT perform input or output, read the clock, read environment variables, generate random numbers, call bridged Rust or C functions, or execute `unsafe` operations, except that it MAY read files through `comptime.read_file(path)` when path is listed as a compile-time input in the package manifest.

**\[CT-6\]** Compile-time evaluation is subject to limits on steps and memory. The limits are implementation-defined, MAY be raised in the manifest, and exceeding them is an error.

### 13.3 Compile-time functions and blocks

**\[CT-7\]** A function declared `comptime fn` can be called only during compile-time evaluation. It MAY take and return compile-time-only values: `type`, `Code`, reflection values, and collections of these.

**\[CT-8\]** A `comptime { … }` block inside a function is evaluated at compile time and its value is embedded in the compiled code. Its type SHALL NOT be a compile-time-only type.

**\[CT-9\]** A `comptime { … }` block at item level SHALL evaluate to a value of type `Code` containing items. Those items are added to the module as if written in its place. Generated `public` items are part of the module's interface (17.4).

### 13.4 Reflection

**\[CT-10\]** During compile-time evaluation, a value of type `type` supports reflection queries, including at least: its name and path; for records, its fields (name, type, visibility, attributes, documentation); for enums, its variants and their fields; its generic parameters; its size and alignment; and whether it implements a given trait.

**\[CT-11\]** Reflection exposes only the members visible from the module in which the compile-time evaluation was requested. A deriver invoked by `@derive` on a type is requested from that type's module and therefore sees all of its fields.

### 13.5 Quotation and code generation

*Syntax:*

```text
Quote   ::= "quote" "{" QuoteToken* "}"
Splice  ::= "$" IDENT | "$(" Expr ")" | "$(" "for" Pattern "in" Expr ("join" STRING)? ")" "{" QuoteToken* "}"
```

**\[CT-12\]** `quote { … }` produces a value of type `Code` representing the enclosed tokens. `$name` and `$(expr)` insert a compile-time value: an identifier, a type, a literal or another `Code` value. `$(for x in e join "sep") { … }` inserts the body once per element, separated by sep.

**\[CT-13\]** Quotation is *hygienic*. An identifier written literally inside a quote resolves in the scope where the quote appears and cannot capture or be captured by names at the place where the code is inserted. An identifier inserted with `$`, or created with `comptime.ident("name")`, resolves at the insertion point.

**\[CT-14\]** Code produced by quotation SHALL be well-formed when inserted. If it is not, the diagnostic SHALL be reported at the attribute or expression that caused the insertion, and SHALL include a note locating the generating code.

**\[CT-15\]** A `comptime fn` marked `@deriver(Tr)` with signature `comptime fn name(t: type) -> Code` is a *deriver* for trait Tr. `@derive(Tr)` on a type calls the deriver for Tr visible at that point and adds the resulting implementation. At most one deriver for a trait may be visible at a use of `@derive`.

*Example:*

```brio
trait Checksum { fn checksum(self) -> U32 }

@deriver(Checksum)
comptime fn derive_checksum(t: type) -> Code {
    return quote {
        impl Checksum for $t {
            fn checksum(self) -> U32 {
                var c: U32 = 17
                $(for f in t.fields) { c = c *% 31 +% self.$(f.name).checksum() }
                return c
            }
        }
    }
}

@derive(Checksum, Debug)
record Reading { patient: PatientId, rate: HeartRate, at: Instant }
```

### 13.6 No textual macros

**\[CT-16\]** Brio has no textual or token-substitution macros. Rust macros are used only through the bridge (18.6).

## 14 Concurrency and the memory model

### 14.1 Tasks

**\[CONC-1\]** A *task* is an independent thread of execution. How tasks are mapped to operating-system threads, and how they are scheduled, is implementation-defined, subject to the rules of this clause and Clause 16.

**\[CONC-2\]** The program begins as a single task that runs `main`.

### 14.2 Asynchronous functions

**\[CONC-3\]** A function declared `async fn` may suspend. A call to an `async` function SHALL be the operand of `await` or of `spawn`. `await e` is permitted only inside an `async` function; it suspends the current task until e completes and evaluates to e's result.

**\[CONC-4\]** `try await f()` awaits f and then applies `try` to its result.

**\[CONC-5\]** Non-`async` code starts asynchronous work only through `tasks` blocks (14.4) or the core function `block_on`, which SHALL NOT be called from `async` code.

**\[CONC-6\]** An `async` function corresponds to a Rust `async fn`. The executor that runs Brio tasks is implementation-defined; when a Brio package is linked into a Rust program that has an async runtime, the bridge SHALL run Brio `async` functions on that runtime (18.7).

### 14.3 Data-race freedom

**\[CONC-7\]** A value may be moved to another task only if its type implements `Send`. A shared borrow may be used by another task only if the borrowed type implements `Sync`.

**\[CONC-8\]** A program that uses no `unsafe` code, other than code generated by the bridge, SHALL be free of data races.

### 14.4 Structured concurrency

*Syntax:*

```text
TasksExpr ::= "tasks" Block
SpawnExpr ::= "spawn" CallExpr
```

**\[CONC-9\]** `spawn f(args)` is permitted only lexically inside a `tasks` block, and not inside a closure within it. The callee and arguments are evaluated in the spawning task, left to right. The call itself then runs as a new task. `spawn` evaluates to a handle of type `Task[T]`, where T is the call's result type; `await h` waits for the task and yields its result.

**\[CONC-10\]** A `tasks` block does not complete until every task spawned within it has completed. No task spawned in a `tasks` block outlives the block.

**\[CONC-11\]** Because spawned tasks cannot outlive their block, their arguments MAY borrow from the enclosing function. For the exclusivity rules of 9.4, every borrow passed to a spawned task is live until the end of the `tasks` block.

**\[CONC-12\]** If one or more spawned calls have type `T ! E` and their handles are not awaited within the block, the `tasks` block has type `() ! E`; otherwise it has type `()`. When such an un-awaited task fails, the other tasks of the block are cancelled (14.5), and the block evaluates to the first such error in order of completion. All such tasks SHALL have the same error type E.

**\[CONC-13\]** A trap in any task traps the whole program (10.4).

**\[CONC-14\]** In a non-`async` function, a `tasks` block blocks the current task until it completes. In an `async` function, it is a suspension point.

*Example:*

```brio
unbounded fn run(sensor: inout Sensor, bus: Bus) -> () ! MonitorError {
    try tasks {
        spawn monitor(mut sensor)     // borrows sensor until the block ends
        spawn alarm_loop(bus)
    }
}
```

### 14.5 Cancellation

**\[CONC-15\]** Cancellation is cooperative and explicit. Cancelling a task sets its cancellation flag; the task keeps running. Cancellable waiting operations then complete promptly with the error `Cancelled`, and `task.is_cancelled()` reports the flag. A task therefore observes cancellation only as ordinary error values, at points it can see, and runs its own cleanup code. Futures are never discarded in the middle of an `await`. A task that ignores cancellation is not stopped; if it is bounded, it still finishes (\[BND-2\]).

### 14.6 Synchronization

**\[CONC-16\]** Tasks communicate through the core types `Channel[T]` (bounded, with a capacity fixed at creation), `Mutex[T]`, `RwLock[T]` and the atomic types. Their semantics are those of the corresponding Rust types.

### 14.7 Memory model

**\[CONC-17\]** Atomic operations take an ordering of `Relaxed`, `Acquire`, `Release`, `AcqRel` or `SeqCst`, with the meaning given by the memory model of ISO/IEC 14882:2020, clause 6.9.2.

**\[CONC-18\]** A data race, which can occur only through `unsafe` code or foreign code, is undefined behavior.

**\[CONC-19\]** On hosted targets, asynchronous operating-system signals SHALL NOT run Brio code in the interrupted context, other than the trap handler. The standard library delivers them as events to a task through `std.signal`.

### 14.8 Deadlock freedom and async discipline

**\[CONC-20\]** *Lock levels.* Every `Mutex` and `RwLock` has a *level*: `Mutex[T, level: n]` for an integer constant n, or, by default, the level *leaf*. While a task holds a lock of level n, it SHALL acquire only locks of a strictly greater level, and SHALL acquire no lock while holding a leaf lock. The compiler SHALL check this rule, including across function calls through the call graph of \[BND-12\]. Programs whose locks all follow it cannot deadlock on locks.

**\[CONC-21\]** A lock guard SHALL NOT be live across an `await`. Code that needs to hold a lock across suspension uses the core type `AsyncMutex`, which follows \[CONC-20\] as well.

**\[CONC-22\]** A function that blocks the current thread, such as a lock acquisition without a timeout or a synchronous file read, is marked `@blocking` in the core library, and the mark propagates to its callers. Calling a `@blocking` function from an `async` function is ill-formed, except inside a `blocking { … }` block, which runs its contents on a separate pool of threads.

**\[CONC-23\]** Atomic operations with an ordering other than `SeqCst` SHALL carry `@ordering("reason")` stating why the weaker ordering is correct. Each such use is recorded in the assurance ledger.

**\[CONC-24\]** *Systematic concurrency testing.* A test marked `@interleavings(max: n)` is run by `brio test` under every schedule of its tasks' synchronization operations and atomic accesses, up to n preemptions, using a technique such as dynamic partial-order reduction. A failing schedule is reported with a seed that replays it exactly (\[OBS-21\]).

**\[CONC-25\]** Channels can still form waiting cycles, which \[CONC-20\] does not cover. Waits without a deadline are `unbounded` (\[BND-16\]); in `critical` code they are forbidden, so every wait there completes or times out.

**\[CONC-26\]** In programs containing `critical` modules and running on more than one core, every lock SHALL declare a priority ceiling, and schedulability analysis (Clause 34) SHALL use a named multiprocessor locking protocol, including blocking times.

**\[CONC-27\]** In `critical` code, the longest time between two cancellation points of every task SHALL be bounded and included in timing analysis.

**\[CONC-28\]** In `critical` code, every atomic operation with an ordering other than `SeqCst` SHALL be covered by a test under \[CONC-24\] that explores all interleavings of the operations involved.

## 15 Unsafe code and C foreign functions

### 15.1 Unsafe blocks

*Syntax:*

```text
UnsafeExpr ::= "unsafe" "(" STRING ")" Block
```

**\[UNS-1\]** An *unsafe operation* (15.2) is permitted only inside an `unsafe` block. The string SHALL be a non-empty literal without interpolation that states why the enclosed operations are sound. An `unsafe` block is an expression whose value is that of its block.

**\[UNS-2\]** The body of an `unsafe fn` is not itself an unsafe context: unsafe operations inside it still require `unsafe` blocks with justifications.

**\[UNS-3\]** `unsafe` blocks, `unsafe fn` declarations and `unsafe impl` declarations are forbidden under the `critical` profile.

*Example:*

```brio
let sample = unsafe("DMA buffer is pinned and fully written before the IRQ fires; see HAZ-112") {
    dma_buf.read()
}
```

### 15.2 Unsafe operations

**\[UNS-4\]** The unsafe operations are:

1. calling a function declared `unsafe fn`, including the core raw-pointer operations such as `Ptr.read`, `MutPtr.write` and `Ptr.offset`;
2. calling a function declared in an `extern` block;
3. reading a field of a `union`;
4. reading or writing a `static var`;
5. calling the core function `unsafe_cast[T, U](x: take T) -> U`, which reinterprets the bits of a value;
6. calling implementation-provided intrinsics that are documented as unsafe, such as inline assembly.

Creating a `Ptr` or `MutPtr` from a reference, comparing pointers and converting them to integers are safe.

### 15.3 Unsafe functions and implementations

**\[UNS-5\]** `unsafe fn` declares a function whose callers must uphold conditions the compiler cannot check. Every `unsafe fn` SHALL have at least one `requires` clause stating those conditions, or a documentation comment containing a paragraph that begins with `Safety:`.

**\[UNS-6\]** `unsafe impl Send for T` and `unsafe impl Sync for T` assert thread-safety properties the compiler cannot derive. They are the only permitted manual implementations of auto traits.

### 15.4 Undefined behavior

**\[UNS-7\]** Code that performs unsafe operations SHALL NOT cause any of the following. Doing so is undefined behavior:

1. reading or writing through a pointer that is null, dangling, misaligned or out of bounds of its allocation;
2. reading uninitialized memory;
3. producing an invalid value: a `Bool` other than 0 or 1, a `Char` that is not a Unicode scalar value, an enum with an invalid discriminant, a range-type value outside its range, a `Str` that is not valid UTF-8, a null or dangling reference, or a value violating a core type's documented invariant;
4. violating the aliasing rules: mutating a place while a shared reference to it is live, or accessing it other than through an exclusive reference while that reference is live, as defined for Rust references in the Rust Reference;
5. a data race (\[CONC-18\]);
6. writing to a `static` that is not a `static var`;
7. calling a function through a pointer of a different ABI or signature;
8. unwinding a foreign exception or panic through a Brio stack frame.

### 15.5 C foreign functions

*Syntax:*

```text
ExternBlock ::= "extern" STRING "{" (Attribute* "unsafe"? "fn" IDENT "(" ParamList? ")" ("->" Type)?)* "}"
```

**\[UNS-8\]** `extern "C" { … }` declares functions that follow the platform's C calling convention. Other ABI strings are implementation-defined. Parameters of foreign functions SHALL NOT have modes; all arguments are passed by value.

**\[UNS-9\]** The parameter and return types of foreign functions SHALL be *FFI-safe*: primitive integer and floating-point types, `Bool`, `()`, `Ptr[T]`, `MutPtr[T]`, `Option[ref T]`, `Option[ref mut T]`, `extern "C" fn(…)` pointers, and records, enums and unions with `@repr(C)`, `@repr(transparent)` or an integer `@repr` whose fields are all FFI-safe.

**\[UNS-10\]** `@export(c, "symbol")` on a function makes it callable from C under the given symbol. Its signature SHALL satisfy \[UNS-8\] and \[UNS-9\]. A trap inside an exported function behaves as in 10.4; control never returns to the foreign caller.

**\[UNS-11\]** Linking foreign libraries is configured in the package manifest; the mechanism is implementation-defined.

### 15.6 Audit

**\[UNS-12\]** A conforming toolchain SHALL be able to list every `unsafe` block, `unsafe fn`, `unsafe impl` and `extern` declaration in a package and its dependencies, with location and justification (19.6).

## 16 Profiles

### 16.1 Declaring a profile

**\[PROF-1\]** Every module has a profile, `standard` or `critical`. The default is `standard`. A module selects `critical` with `@profile(critical)` on its module header or its first item. The optional `level:` argument records the intended safety classification (for example `"IEC62304-C"`) in build metadata and reports; it has no other effect.

**\[PROF-2\]** The profile applies to every item in the module, including items generated by `comptime` code.

**\[PROF-3\]** A function in a `critical` module SHALL call only: functions in `critical` modules, core library items that the implementation documents as *critical-compatible*, and `@trusted` functions (16.4). The implementation SHALL publish the list of critical-compatible core items.

### 16.2 Rules of the critical profile

**\[CRIT-1\]** *Verification.* Every item is in prove mode. `@verify(check)` and `@verify(assume)` are ill-formed, except as permitted in 16.4.

**\[CRIT-2\]** *Allocation.* Only `@init` functions (16.3) may allocate heap memory. Every other function SHALL satisfy the requirements of `@no_alloc` (16.5).

**\[CRIT-3\]** *Recursion.* The call graph of the program's `critical` functions SHALL be acyclic.

**\[CRIT-4\]** *Indirect calls.* Dynamic values (`any`) and values of function type are forbidden. Closures are permitted only when passed as parameters whose type is an `Fn…` trait used statically (\[TRAIT-10\]), so every call target is known at compile time.

**\[CRIT-5\]** *Termination.* Every function in a `critical` module SHALL be bounded (Clause 39). The one exception is a task's entry function, which MAY be declared `unbounded` if its only unbounded construct is its outermost `loop`, every iteration of which is bounded and reaches a suspension point or a call to a core periodic-wait function.

**\[CRIT-6\]** *Floating point.* Every floating-point operation SHALL be proven not to produce NaN or an infinity. `@fp(fast)` is forbidden.

**\[CRIT-7\]** *Value semantics.* Record and enum fields SHALL NOT have reference types, types SHALL NOT declare origin parameters, and reference values SHALL NOT be stored in collections. References exist only as parameters, pattern bindings, local bindings and `from` results.

**\[CRIT-8\]** *Forbidden constructs.* `unsafe` blocks and functions, `static var`, `union`, `RefCell`, `Rc`, `extern` declarations and direct calls to bridged Rust items are forbidden.

**\[CRIT-9\]** *Static task set.* `spawn` SHALL appear only in `tasks` blocks in `main` or in `@init` functions, and not inside loops, so the number of tasks is known at compile time. Channel capacities SHALL be constant expressions. The implementation SHALL use a bounded-priority-inversion protocol for `Mutex` in critical programs and SHALL document which one.

**\[CRIT-10\]** *Stack bound.* The implementation SHALL compute a worst-case stack usage for each task entry function and for the trap handler, and SHALL report it. `@max_stack(n)` on a function makes a computed bound greater than n bytes an error.

**\[CRIT-11\]** *Trap handler.* The executable SHALL declare a trap handler (\[TRAP-11\]).

### 16.3 The initialization phase

**\[PROF-4\]** A function marked `@init` MAY allocate and MAY `spawn` within `tasks` blocks. `@init` functions SHALL be called only from `main` or from other `@init` functions.

**\[PROF-5\]** In a program containing `critical` modules, `main` SHALL call every `@init` function it uses before its first statement that is not a call to an `@init` function. After that point, the program is in its *operational phase*, and no `critical` code allocates.

### 16.4 Trusted boundaries

**\[PROF-6\]** A function in a `critical` module marked `@trusted(reason)` is a *trusted boundary*. Its body is exempt from 16.2 and MAY call `standard` modules and bridged Rust items. Its reason string SHALL be non-empty.

**\[PROF-7\]** Callers of a trusted boundary SHALL prove its preconditions. Its postconditions are assumed by callers and are checked at runtime at every return from it, so that a violation traps instead of passing unnoticed into verified code.

**\[PROF-8\]** Every trusted boundary SHALL be listed, with its reason, in the `brio ship` report.

### 16.5 Performance attributes

These attributes are available under both profiles.

**\[PROF-9\]** `@no_alloc`: the function, and every function it can call, SHALL NOT allocate heap memory. Calls to bridged or foreign functions are permitted only if they are declared `@no_alloc` in their bridge or `extern` declaration.

**\[PROF-10\]** `@no_dyn`: the function, and every function it can call, SHALL NOT make calls through dynamic values or function values.

**\[PROF-11\]** `@hot`: every implicit check in the function (overflow, bounds, range and conversion checks) SHALL be proven or removed at compile time, and the function SHALL NOT call any generic function through dictionary passing in release builds. Violations are errors.

**\[PROF-12\]** `@fp(fast)` permits the implementation to reassociate floating-point operations and to use fused operations in the function or module. `@fp(strict)`, the default, forbids this (\[TYPE-8\]).

*Example:*

```brio
@hot @no_alloc @no_dyn
fn filter_ecg(samples: inout [F32; 512], coeffs: [F32; 32]) {
    for i in 32 ..< 512 {
        samples[i] = dot(samples[i - 32 ..< i], coeffs)   // bounds proven from the range
    }
}
```

## 17 Modules, packages, interfaces and private source

### 17.1 Packages and manifests

**\[PKG-1\]** A package is a directory containing a `Brio.toml` manifest in TOML 1.0 format. A package is a *library* or an *executable* (one that declares `main`).

**\[PKG-2\]** The manifest has the following tables. Unknown keys are errors.

| Table | Contents |
| --- | --- |
| `[package]` | `name` (lowercase letters, digits and hyphens), `version` (Semantic Versioning 2.0.0), `edition` (`"2026"`), `source` (`"public"`, `"private"` or `"escrow"`), `escrow-agent` (required when source is `"escrow"`), `license`, `tier` (\[EMB-1\]) |
| `[dependencies]` | Brio packages, version requirements and registries (\[WS-2\]) |
| `[rust-dependencies]`, `[rust-companion]` | Rust crates and the companion crate (Clause 18) |
| `[python-dependencies]` | Python packages (\[SCR-16\]) |
| `[foreign]` | Target languages for bindings (\[FFI-2\]) |
| `[extensions]` | Tool extensions (\[EXT-1\]) |
| `[verify]` | Verification modes per build configuration (\[CON-17\], \[CON-19\]) and prover limits (\[CON-25\]) |
| `[comptime]` | Compile-time inputs (\[CT-5\]) and limits (\[CT-6\]) |
| `[lints]`, `[lints.<name>]` | Lint levels (\[LANG-17\], \[EXT-9\]) |
| `[permissions]` | Permissions of the program and its dependencies (Clause 41) |
| `[audit]` | Vulnerability severity threshold (\[SEC-7\]) |
| `[preview]` | Enabled preview features (\[EVO-2\]) |
| `[workspace]`, `[registries]` | Workspaces and registries (\[WS-1\], \[WS-2\]) |
| `[target.memory]` | Memory regions of freestanding targets (\[EMB-5\]) |
| `[requirements]` | The requirements file (\[TOOL-16\]) |
| `[runtime]` | Runtime inspection (\[OBS-23\]) |
| `[build.dev]`, `[build.release]` | Build settings, including the minimum log level (\[OBS-7\]), fault injection (\[OBS-20\]), benchmark thresholds (\[OBS-18\]), coverage thresholds (\[TOOL-15\]) and trait resolution depth (\[BND-21\]) |
| `[links]` | Foreign libraries (\[UNS-11\]) |

**\[PKG-3\]** A `Brio.lock` file records the exact version and content hash of every dependency. When it is present, a build SHALL use exactly those versions. Cyclic dependencies between packages are ill-formed.

### 17.2 Package layout and modules

**\[PKG-4\]** A package's source files are under `src/`. The root module is `src/lib.brio` for a library or `src/main.brio` for an executable. The file `src/a/b.brio` is the module `a.b`. Directories introduce path segments only; no file is required for a directory.

**\[PKG-5\]** A source file MAY begin with `module a.b`; if present, the path SHALL match the file's location.

**\[PKG-6\]** `use` imports items by path: `use units.{Ml, MlPerHour}` or `use pump.controller.Controller as Ctl`. Glob imports are not permitted. Modules within a package MAY refer to one another cyclically.

### 17.3 Interface files

**\[PKG-7\]** For every module, the implementation generates an *interface*: a `.brioh` file in Brio syntax that contains exactly the information other code needs to compile against the module. The *public interface* contains `public` items; the *package interface* additionally contains `public(package)` items.

**\[PKG-8\]** An interface file begins with `interface a.b` and the interface format version, and contains, in declaration order and in canonical `brio fmt` formatting:

1. the `use` declarations needed to name the types it mentions;
2. every exported function's signature, including generic parameters and bounds, parameter labels and modes, origins, contracts, attributes that affect callers, and the `@specialize` list;
3. the bodies of `@inlinable` functions and the values of exported constants;
4. every exported record, with its fields if it is frozen, or as an `opaque record` listing only its public fields otherwise, together with its type invariant;
5. every exported enum, with its variants and an indication of whether it is frozen;
6. every exported trait in full, and the signatures of implementations of exported traits for exported types;
7. exported units, type aliases, range types and distinct types;
8. documentation comments.

**\[PKG-9\]** Generation is deterministic: the same source and toolchain produce byte-identical interface files.

**\[PKG-10\]** The *interface hash* of a module is a cryptographic hash of its interface file with documentation comments removed.

### 17.4 Interface lock

**\[PKG-11\]** *Public API guard.* Interface files are never part of a package's source. When the API guard is enabled (`api-guard = true` in `[package]`, the default for libraries), `Brio.lock` records, for each public item, a hash of its interface declaration. A build SHALL fail when the public interface differs from the recorded hashes, unless it is run with `--accept-interface-change`, which updates them. The diagnostic SHALL list every added, removed and changed item and, when the previous interface can be regenerated from version control or from the previously published artifact, SHALL show the textual difference.

### 17.5 Incremental compilation

**\[PKG-12\]** A change to a module that does not change its interface hash (public or package) SHALL NOT require any other module to be recompiled or re-verified.

*Note:* Because `@inlinable` bodies are part of the interface, changing one causes dependents to be rebuilt; changing any other function body does not.

### 17.6 Frozen and opaque types

**\[PKG-13\]** Unless marked `frozen`, a `public` record is *opaque* to other packages: they SHALL NOT use its memberwise initializer, record update or record patterns, and they SHALL NOT depend on its size or layout. They access its public fields as if through accessor functions.

**\[PKG-14\]** A `frozen` record's field names, types and order are part of its public interface. Other packages MAY construct it and match on it when all its fields are public.

**\[PKG-15\]** Unless marked `frozen`, a `public` enum MAY gain variants in later versions, and other packages SHALL match it with a wildcard arm (\[PAT-15\]). A `frozen` enum's variant set is fixed.

**\[PKG-16\]** The body of an `@inlinable` function SHALL refer only to `public` items. `@inlinable` makes the body part of the public interface.

**\[PKG-17\]** The rules of 17.6 apply to every dependency between packages, whether the dependency's source is available or not, so that a package behaves the same when distributed with or without source. An implementation MAY optimize across package boundaries when it has the source or intermediate code, provided the program's meaning does not change.

### 17.7 Semantic versioning

**\[PKG-18\]** Before a package is published, the toolchain SHALL compare its public interfaces with those of the previous published version and SHALL refuse to publish if the version number does not reflect the most significant change:

| Change | Minimum version increase |
| --- | --- |
| Removing or renaming an exported item | Major |
| Changing an exported signature, parameter mode, label or origin | Major |
| Adding or strengthening a precondition, or weakening a postcondition | Major |
| Adding a variant to a frozen enum, or changing a frozen record's fields | Major |
| Making a frozen type non-frozen | Major |
| Adding an exported item, trait implementation or `@specialize` entry | Minor |
| Adding a variant to a non-frozen enum, or a field to an opaque record | Minor |
| Weakening a precondition or strengthening a postcondition | Minor |
| Making a non-frozen type frozen | Minor |
| Changes to documentation, to non-inlinable bodies, or to `@inlinable` bodies that keep all contracts | Patch |

**\[PKG-19\]** When the toolchain cannot decide whether a contract change strengthens or weakens a contract, it SHALL treat the change as major.

### 17.8 Package artifacts and private source

**\[PKG-20\]** A published package artifact contains: the manifest; the public interface files; compiled code for each supported target; build metadata; and, depending on the `source` setting, the source.

| `source` | Source in the artifact |
| --- | --- |
| `"public"` | Full source under `src/` |
| `"private"` | None |
| `"escrow"` | Source encrypted for the named escrow agent; the toolchain never decrypts it |

**\[PKG-21\]** Compiled code for a target consists of object code, and MAY also include LLVM bitcode or Brio intermediate code to allow optimization across the package boundary. The artifact SHALL NOT be required to contain source for any consumer to compile, link, verify or test against it.

**\[PKG-22\]** Build metadata SHALL record the toolchain version, the qualification status (\[GEN-15\]), the hash of all build inputs, the profiles and verification modes used, and a summary of the verification results.

**\[PKG-23\]** Package artifacts SHALL be reproducible: building the same inputs with the same toolchain SHALL produce bit-identical artifacts. Artifacts SHOULD be signed, and a consumer MAY require valid signatures.

**\[PKG-24\]** The implementation SHALL document which toolchain versions can link compiled code produced by which others.

### 17.9 The build directory and the interface cache

**\[PKG-25\]** All files that a build generates (interfaces, compiled code, caches, the assurance ledger and reports) are written to the *build directory*, `.brio/` at the workspace root by default. A package's source SHALL NOT be required to contain any generated file: a source tree containing only `.brio` files, `Brio.toml` and `Brio.lock` is complete. `brio new` SHALL add the build directory to the version-control ignore file.

**\[PKG-26\]** *Content addressing.* Every cache entry is keyed by a hash of all its inputs: the module's source, the interfaces of the modules it depends on, the toolchain version, the target and the build settings. Entries are never selected by timestamps or file names. Before an entry is used, its content SHALL be verified against its key; a missing, stale or corrupted entry is regenerated. Deleting the build directory therefore changes only build time, and the rebuilt artifacts are identical (\[PKG-23\]).

**\[PKG-27\]** *Interface first.* The compiler SHALL be able to produce a module's interface after checking only its signatures, type declarations, contracts, `@inlinable` bodies and item-level `comptime` expansions, without checking or compiling any other function body. This is possible because every signature is written explicitly (\[TYPE-22\]). A module SHALL be able to start compiling as soon as the interfaces it uses exist, while the bodies of its dependencies are still being compiled.

**\[PKG-28\]** *Early cutoff.* When a module is rebuilt and its interface hash is unchanged, no other module is recompiled or re-verified (\[PKG-12\]); only the module itself and the final link are redone.

**\[PKG-29\]** *Shared caches.* Cache entries MAY be shared between branches, machines and continuous integration through a remote cache, subject to these rules:

1. only *trusted writers*, builders designated in the cache's server configuration such as continuous integration on protected branches, MAY upload entries; all other builds, including those of pull requests from forks and developers' machines, only read. A client setting cannot override this;
2. the cache key is a domain-separated hash of a canonical, versioned encoding of an enumerated list of inputs, including the content hash of the toolchain itself, the target, the build settings, declared compile-time files and license identities; adding an input changes the encoding version;
3. every entry is verified as in \[PKG-26\] and, when the cache is not fully trusted, SHALL carry the signature of a trusted writer;
4. every path in an entry SHALL be checked to lie inside its destination before anything is written.

A build that uses remote entries is identical to one that does not. The conformance suite SHALL test key completeness by rebuilding with every undeclared input changed and requiring identical results.

**\[PKG-30\]** The interfaces of dependencies, including those from published artifacts (\[PKG-20\]), are stored in the build directory, never in the consumer's source tree. The language server SHALL use the same interfaces and cache as the compiler (\[TOOL-18\]).

### 17.10 Licensed packages

A *licensed package* is a published package whose layers are readable only by the parties the publisher chooses.

**\[PKG-31\]** *Scope and limit.* Encryption keeps licensed layers confidential from every party without a licensee key, including registries, mirrors and network observers, and guarantees their integrity to everyone. It does not prevent a licensee from reading content its toolchain has decrypted, and an implementation SHALL NOT claim otherwise. Obfuscation is not part of this mechanism.

**\[PKG-32\]** *Layers.* The manifest table `[distribution]`, which extends \[PKG-2\], assigns each layer of a published version an audience of `"public"`, `"licensed"`, `"escrow"` or `"none"`. The layers are `interfaces` (interface files), `docs`, `code` (compiled code per target), `bitcode` (\[PKG-21\]) and `source`. The `source` layer's audience follows the `source` key (\[PKG-20\]). Public metadata is always unencrypted: name, version, dependencies, public interface hashes, declared permissions (\[CAP-8\]), advisories and the public part of the license terms, so that resolution, `brio audit` and permission policies work without any key.

**\[PKG-33\]** *Encryption.* Each non-public layer of a published version SHALL be encrypted with a fresh random content key using a *key-committing* authenticated encryption: either a committing algorithm from a documented set, or an algorithm from a documented set that includes AES-256-GCM and XChaCha20-Poly1305 whose encryption key and a *commitment tag* are both derived from the content key with HKDF, the tag being recorded in the signed manifest (\[PKG-35\]). A ciphertext therefore decrypts under exactly one key. The associated data SHALL bind the package name, version, layer and target, so that a ciphertext or key cannot be moved to another package, version, layer or target.

**\[PKG-34\]** *Key wrapping.* Each content key SHALL be wrapped separately for each recipient's public key with HPKE (RFC 9180) in base mode. The HPKE `info` input SHALL bind the recipient's key fingerprint, the license identifier and the associated data of \[PKG-33\]. Every wrapped key SHALL be signed by the party that produced it, the publisher or its key custodian (\[PM-28\]), and a wrapped key without a valid signature SHALL NOT be used. Implementations SHALL support a classical suite based on X25519 and a hybrid post-quantum suite combining X25519 with ML-KEM-768 (FIPS 203) through a combiner that binds both key-encapsulation ciphertexts and both public keys. A publisher MAY require the hybrid suite, and a licensee MAY refuse artifacts that do not use it. Wrapped keys are stored in the artifact, addressed by the recipient's key fingerprint.

**\[PKG-35\]** *Signatures and integrity.* The publisher SHALL sign a manifest listing, for each layer and target, the hash of its ciphertext, its commitment tag (\[PKG-33\]), the hash of its plaintext, its suite and its recipients (\[SEC-14\]). Before any tool reads a licensed layer, including the compiler, compile-time evaluation, the language server and the assistant server, the toolchain SHALL verify, in this order, the manifest signature, the ciphertext hash, the wrapped key's signature, the commitment tag, and after decryption the plaintext hash. Any failure is an error, no part of a layer that failed verification is used, and every failure SHALL be reported to remote parties with the same response, so that no service can be used to test keys.

**\[PKG-36\]** *Untrusted registries.* Registries and mirrors store ciphertext, wrapped keys, signatures and public metadata, and SHALL NOT require any decryption key. A compromised registry discloses no licensed content.

### 17.11 Keys and licenses

**\[PKG-37\]** *Licensee keys.* A licensee key pair is identified by its fingerprint. The private key is held outside the source tree, in the operating system's key store, a hardware security key, a key-management service or a continuous-integration secret, through `brio license` commands. The toolchain SHALL NOT write private keys or unwrapped content keys to the source tree, the build log or any cache shared with other users.

**\[PKG-38\]** *License tokens.* A publisher issues each licensee a signed *license token* stating the licensee, its key fingerprints, the packages and version range, the permitted layers and targets, and an optional expiry date. The toolchain SHALL verify tokens offline, without contacting any service. A build that needs a licensed layer SHALL fail with a diagnostic when no valid token covers the package, version, layer and target, or when the token has expired and grants no build rights after expiry for the version concerned (\[PM-32\]). Because an offline clock can be set back, expiry is enforced authoritatively by the key custodian, which wraps no content key after the license has ended; the toolchain additionally records the latest registry time it has seen and rejects tokens that expired before it.

**\[PKG-39\]** *No kill switch.* License checks happen only at build time. A built program SHALL NOT check, contact or depend on any license, so a program already built and shipped never stops working because a license expired or was withdrawn.

**\[PKG-40\]** *Recipients.* The publisher MAY add recipients to an existing version by adding wrapped keys and re-signing the manifest, without re-encrypting any layer. Removing a recipient applies to future versions only; content a recipient could already decrypt cannot be withdrawn, and `brio license revoke` SHALL say so. After a key compromise, the publisher publishes a new version with new content keys and MAY mark earlier versions exposed with an advisory (\[SEC-17\]).

### 17.12 Building with licensed packages

**\[PKG-41\]** *Decryption cache.* Decrypted layers are cached in the build directory, keyed by their plaintext hash (\[PKG-26\]), so each layer is decrypted once per machine. On hosts that support it, the cache is readable only by the building user, and the implementation MAY keep it encrypted at rest with a machine-local key.

**\[PKG-42\]** *Shared caches.* A cache entry whose inputs include a licensed layer SHALL be encrypted, as in \[PKG-33\] and \[PKG-34\], to the keys of the licensees holding that license, SHALL be uploaded only to a remote cache (\[PKG-29\]) configured for that license, and SHALL NOT be served to any other user.

**\[PKG-43\]** *Reproducibility and locking.* `Brio.lock` records the ciphertext and plaintext hash of every licensed layer. Because decrypted layers are deterministic, builds by the same licensee are reproducible (\[PKG-23\]).

**\[PKG-44\]** *Leak tracing.* A publisher MAY request a per-licensee identifier in the `code` layer, embedded in a dedicated section of the licensee's copy. It SHALL NOT change the behavior of any program, is recorded in the assurance ledger, and `brio license trace` SHALL report the identifier found in a binary.

**\[PKG-45\]** *Visibility.* The assurance ledger SHALL list every licensed dependency, the layers received, the license token used and its expiry, and whether leak tracing is present. Package permissions (\[CAP-8\]) are computed from the decrypted code and compared with the permissions the publisher declared; a difference is an error.

**\[PKG-46\]** *Cryptographic agility.* Suites are identified by a version number. The toolchain SHALL decrypt every suite used by artifacts of the editions it supports, SHALL publish a date after which a weakened suite is refused for new publications, and SHALL warn when a dependency uses a suite scheduled for refusal.

**\[PKG-47\]** Leak-tracing identifiers (\[PKG-44\]) SHALL be encoded so that combining the copies of several licensees does not remove them. They attribute a leak; they do not prevent one, and the documentation SHALL say so.

**\[PKG-48\]** For the rules of 17.7, a new public trait implementation, and a change in whether a public type is `Send`, `Sync` or `Freeze`, are breaking changes unless shown otherwise by the compatibility checker.

## 18 Rust interoperability

### 18.1 Model

**\[RUST-1\]** Brio code uses Rust crates, and Rust code uses Brio packages, through *bridges*. For each Rust crate used, the implementation generates Brio interface files describing the crate's public API, and a Rust *shim crate*, compiled by rustc, that connects the two sides. For each Brio package used from Rust, it generates a Rust binding crate.

**\[RUST-2\]** The implementation obtains a Rust crate's API from the crate's compiled metadata or rustdoc output; the mechanism is implementation-defined. The version of the Rust toolchain used is a build input and SHALL be recorded in build metadata.

**\[RUST-3\]** A Rust item that cannot be represented under this clause is omitted from the generated interface and listed, with the reason, in the *bridge report*. Referring to an omitted item is an error whose diagnostic SHALL give that reason.

### 18.2 Declaring and importing Rust crates

**\[RUST-4\]** Rust crates are declared in `[rust-dependencies]` using Cargo's dependency syntax, including features. Their exact versions are recorded in `Brio.lock`.

**\[RUST-5\]** Rust items are imported with `use rust crate.path.{Items}`. Rust's `::` separator becomes `.`. Rust names that are Brio keywords are written as raw identifiers, such as `` `type` ``. Rust names are otherwise unchanged.

**\[RUST-6\]** A Brio package MAY include a companion Rust crate, declared with `[rust-companion] path = "…"` in the manifest, which is bridged automatically. Companion crates hold Rust code that cannot be expressed in Brio, such as invocations of function-like Rust macros.

### 18.3 Type mapping

**\[RUST-7\]** Rust types map to Brio types as follows:

| Rust | Brio | Crossing |
| --- | --- | --- |
| `bool`, `char`, integer and float primitives | `Bool`, `Char`, `I8`…`U128`, `ISize`, `USize`, `F32`, `F64` | Direct |
| `()`, `!`, tuples, `[T; N]`, `[T]`, `str` | `()`, `Never`, tuples, `[T; N]`, `[T]`, `Str` | Direct |
| `&T`, `&mut T` | Default-mode or `inout` parameter; `ref T`, `ref mut T` elsewhere | Direct |
| `Box<T>`, `Rc<T>`, `Arc<T>` | `Box[T]`, `Rc[T]`, `Arc[T]` | Direct |
| `Vec<T>`, `String` | `List[T]`, `String` | Direct |
| `HashMap<K, V>`, `HashSet<T>` | Imported as the Rust types themselves; `Map[K, V]` and `Set[T]` convert from them explicitly | Direct as Rust types; conversion takes linear time |
| `Option<T>`, `Result<T, E>` | `T?`, `T ! E` | Converted in constant time |
| `&dyn Trait`, `&mut dyn Trait`, `Box<dyn Trait>` | `ref any Trait`, `ref mut any Trait`, `any Trait` | Converted in constant time; the shim translates method tables |
| `impl Trait` in argument or return position | The trait name, used statically (\[TRAIT-10\]) | Direct |
| `fn(A) -> R`, `unsafe fn`, `async fn` | Function type, `unsafe fn`, `async fn` | Direct |
| Struct or enum with `#[repr(C)]`, `#[repr(transparent)]` or integer `repr` | Brio record or enum with the same `@repr` | Direct |
| Other struct | Opaque record (18.4) | Direct |
| Other enum | Opaque type with a generated view (18.4) | Direct |

**\[RUST-8\]** The implementation SHALL give the core types in the *Direct* rows the same size, alignment and representation as their Rust counterparts, so that values cross without conversion.

**\[RUST-9\]** Rust lifetimes map to origins. Elided Rust lifetimes are made explicit using Rust's elision rules before mapping. The borrow rules of Clause 9 apply to bridged items exactly as the Rust borrow checker would apply them (\[OWN-27\]).

**\[RUST-10\]** Range types, unit-annotated types and distinct types cross to Rust as their base, numeric or underlying types. Every value entering Brio from Rust into a type with validity constraints (range types, enums, `Char`, `Str`, non-null references) SHALL be checked at runtime in every verification mode, unless the check is proven from facts that are not assumed (\[CON-28\]). A bridge contract may state the range, but it is never trusted in place of the check.

### 18.4 Rust structs, enums and traits

**\[RUST-11\]** A Rust struct without a C-compatible `repr` is imported as an opaque record with the size and alignment rustc computes. Brio code MAY hold, move and pass it by value; moves are bitwise, as in Rust. Its public fields are accessed through generated accessors.

**\[RUST-12\]** A Rust enum without a C-compatible `repr` is imported as an opaque type with a generated method `view(self)` that returns a Brio enum whose variants mirror the Rust variants and whose payloads are shared borrows of the Rust payloads. Matching is written `match value.view() { … }`.

**\[RUST-13\]** A Rust trait whose methods are all representable is imported as a Brio trait. Brio types MAY implement it, subject to \[TRAIT-7\]; the shim then contains a Rust implementation that delegates to the Brio methods. Rust types MAY implement Brio traits under the same rule.

### 18.5 Bridge declarations

*Syntax:*

```text
BridgeDecl   ::= "rust" "bridge" IDENT "{" BridgeMember* "}"
BridgeMember ::= "instantiate" RustPath GenericArgs
               | Attribute* "fn" RustPath Contract*
```

**\[RUST-14\]** A generic Rust item is usable from Brio only at instantiations the shim compiles. `instantiate Path[Args]` requests one. The implementation SHALL also instantiate automatically every combination that Brio code uses with concrete, bridgeable type arguments; `instantiate` is needed only for instantiations the implementation cannot discover.

**\[RUST-15\]** `fn RustPath Contract*` attaches contracts and attributes to a Rust function. Its preconditions are checked before each call and its postconditions after each return in check mode, and assumed in prove mode. Attributes such as `@no_alloc` are claims that the implementation cannot verify; every such claim SHALL be listed in the `brio ship` report.

*Example:*

```brio
rust bridge app_shell {
    instantiate tauri.AppHandle[tauri.Wry]

    @no_alloc
    fn sensor_hal.read_raw
        ensures result.len() == 64
}
```

### 18.6 Exposing Brio to Rust

**\[RUST-16\]** The generated Rust binding crate exposes every `public` item of a Brio package as a safe Rust API. Parameter modes map to Rust reference types, `T ! E` to `Result`, and contracts to runtime checks in the Brio callee.

**\[RUST-17\]** `@rust.attr(path(args))` on a public Brio function makes the shim generate a Rust wrapper function carrying the attribute `#[path(args)]` and forwarding to the Brio function. Procedural macros therefore run on ordinary Rust code. Errors from the macro SHALL be reported at the Brio attribute.

**\[RUST-18\]** `@rust.derive(Tr, …)` on a public record or enum makes the shim generate a Rust type carrying `#[derive(Tr, …)]`. For a frozen type with a C-compatible `repr`, that Rust type has the same layout and no conversion occurs; otherwise the shim converts between the two representations.

*Example:*

```brio
@rust.derive(Serialize, Deserialize)
public frozen record Vitals { public hr: HeartRate, public spo2: Percent }

@rust.attr(tauri.command)
public fn get_vitals(id: PatientId) -> Vitals ! AppError { … }
```

### 18.7 Asynchronous interoperation

**\[RUST-19\]** A Brio `async fn` is exposed to Rust as a function returning a Rust future, and a Rust `async fn` is imported as a Brio `async fn`. When Brio code runs inside a Rust async runtime, tasks it spawns SHALL run on that runtime. How the runtime is selected is implementation-defined.

### 18.8 Panics

**\[RUST-20\]** Every call from Brio into Rust passes through the shim, which SHALL catch any unwinding panic and turn it into a trap (\[TRAP-2\]). The bridge SHALL install a Rust panic hook that runs the Brio trap handler, so that a Rust panic configured to abort still runs the handler first.

### 18.9 Boundary cost

**\[RUST-21\]** Shim code SHALL NOT allocate, except where the bridged function itself allocates or a conversion listed as such in the bridge report requires it. The bridge report SHALL list every conversion performed at the boundary.

**\[RUST-22\]** A release build SHOULD use link-time optimization across the Brio–Rust boundary, so that shim functions can be inlined.

## 19 Toolchain requirements

This clause applies to conforming toolchains (\[GEN-14\]). Command names refer to the reference command-line tool, `brio`; an implementation MAY use other names if it documents the correspondence.

### 19.1 Diagnostics

**\[TOOL-1\]** Every diagnostic SHALL have a severity (error, warning or note), a stable code from the ranges in Annex C, a primary source location, and a message. Where a rule of this specification is violated, the message SHALL cite the rule's identifier.

**\[TOOL-2\]** Diagnostics SHOULD include secondary locations with labels and a *help* section. Where the fix is unambiguous, the diagnostic SHALL include a suggested edit, marked as machine-applicable.

**\[TOOL-3\]** The toolchain SHALL be able to emit diagnostics in a documented JSON format, including suggested edits.

**\[TOOL-4\]** `brio explain <code>` SHALL print a description of the diagnostic, with an example of code that triggers it and code that fixes it.

**\[TOOL-5\]** The language server SHALL use the same analysis as the compiler, so that the diagnostics shown in an editor are identical to those of `brio check`.

**\[TOOL-6\]** Use of an item marked `@deprecated(message)` produces a warning showing the message. Warnings MAY be controlled per item with `@lint.allow(name)` and `@lint.deny(name)`.

### 19.2 Commands

**\[TOOL-7\]** A conforming toolchain SHALL provide the following commands:

| Command | Function |
| --- | --- |
| `brio build` | Build the package in the development or release configuration |
| `brio check` | Report all diagnostics without generating code |
| `brio explain` | Explain a diagnostic (\[TOOL-4\]) |
| `brio lint`, `brio fix` | Report lints; apply fixes labelled certain (\[DIAG-3\]) |
| `brio test`, `brio bench`, `brio sim`, `brio mutate` | Tests (19.4), benchmarks, simulation and mutation testing (Clause 22) |
| `brio prove` | Run verification only, and report every obligation and its status |
| `brio interp` | Run code under the interpreter that detects undefined behavior (\[MODEL-6\]) |
| `brio fmt` | Format source files canonically (19.3) |
| `brio doc` | Generate documentation from interfaces and documentation comments |
| `brio expand` | Show the code produced by `comptime` blocks, embedded languages, derivers and bridges |
| `brio ledger` | Show the assurance ledger (23.1) |
| `brio run`, `brio repl`, `brio script` | Run scripts, the interactive shell, and script locking and promotion (Clause 20) |
| `brio bind` | Generate bindings for other languages (Clause 25) |
| `brio audit`, `brio advisory` | Audit dependencies and publish advisories (Clause 26) |
| `brio vendor` | Copy all dependencies for offline builds (\[WS-4\]) |
| `brio observe`, `brio inspect`, `brio symbolicate`, `brio flags` | Local telemetry viewer, runtime inspection, crash symbolication and flag listing (Clause 22) |
| `brio migrate` | Migrate between editions (\[EVO-3\]) and translate Rust or Python |
| `brio publish` | Check versioning (17.7) and publish a package artifact |
| `brio ship` | Produce the release evidence report and apply release gates (19.6) |
| `brio lsp`, `brio debug`, `brio bsp`, `brio mcp` | Language server, debug adapter, build server and assistant server (19.9) |
| `brio index` | Write a code-navigation index (19.9) |
| `brio lens`, `brio lens build`, `brio lens test`, `brio lens schema` | Open the Lens viewer, build with stage records, run stage tests, and export the published record schema (Clause 49) |

### 19.3 Formatting

**\[TOOL-8\]** `brio fmt` SHALL produce one canonical layout, with no configuration options. Formatting SHALL be idempotent, SHALL preserve comments, and SHALL NOT change the meaning of the program. Interface files are produced in the same layout.

### 19.4 Testing

**\[TOOL-9\]** `brio test` SHALL run each `test` item and report each as passed or failed. A trap in a test SHALL fail that test only; the remaining tests SHALL still run.

**\[TOOL-10\]** In test builds, every contract and implicit check is in check mode, except those in prove mode, whatever the release configuration says.

**\[TOOL-11\]** A `test … forall (x: T, …)` item is a *property test*. The toolchain generates values for each parameter and runs the body once per generated case:

1. generators exist for primitive types, range types, units, distinct types, tuples, arrays, `Option`, `Result`, collections, and records and enums built from these; generated values SHALL include the bounds of range types and other edge values;
2. `@gen(f)` on a type or test parameter replaces the generator with the function f;
3. the number of cases defaults to 256 and is configurable;
4. when a case fails, the toolchain SHALL shrink it to a smaller failing case and report that case and the seed;
5. the default seed is derived from the test's name, so runs are reproducible; running with a given seed SHALL reproduce the same cases.

**\[TOOL-12\]** The toolchain SHALL support coverage-guided fuzzing of property tests.

### 19.5 Coverage

**\[TOOL-13\]** `brio test --coverage` SHALL report statement, branch and modified condition/decision coverage (MC/DC) for each module.

### 19.6 Release evidence: brio ship

**\[TOOL-14\]** `brio ship` SHALL build the release configuration, run all tests, and produce a report, in human-readable and JSON forms, containing:

1. the toolchain version and qualification status (\[GEN-15\]), and the hash of all build inputs;
2. for each module, its profile and safety level, and the number of obligations proven, checked at runtime and assumed;
3. every assumption and its reason (\[CON-18\]);
4. every trusted boundary and its reason (\[PROF-8\]);
5. every unsafe block, unsafe function, unsafe implementation and foreign declaration, with its justification (\[UNS-12\]);
6. every claim and conversion in bridges (\[RUST-15\], \[RUST-21\]);
7. interface changes since the previous release and their semantic-versioning classification (17.7);
8. worst-case stack bounds for `critical` programs (\[CRIT-10\]);
9. test and coverage results;
10. the requirement traceability matrix (19.7);
11. a software bill of materials listing every Brio package and Rust crate with version, license and content hash;
12. the result of a reproducibility check, obtained by rebuilding and comparing artifacts.

**\[TOOL-15\]** `brio ship` SHALL fail when: the build has any error; any test fails; any obligation in a `critical` module is not proven; the public interface changed without being accepted (\[PKG-11\]); the package version does not satisfy 17.7; the reproducibility check fails; or a coverage threshold configured in the manifest is not met.

### 19.7 Requirement traceability

**\[TOOL-16\]** `@req("ID", …)` associates items and tests with requirement identifiers. When the manifest names a requirements file, every identifier used SHALL appear in it.

**\[TOOL-17\]** The traceability matrix lists, for each requirement: the items that implement it, the tests that exercise it, the verification status of those items, and whether the tests passed. Requirements with no implementing item or no test SHALL be flagged.

### 19.8 Build configurations

*Note:* The following is informative. The reference toolchain compiles development builds with a fast code generator and dictionary-passing generics, and release builds with an optimizing code generator, monomorphization and link-time optimization across the Rust boundary. It also provides `brio migrate`, which translates Rust modules to Brio, and `brio equiv`, which tests a migrated module against the original with generated inputs. None of this changes the meaning of any program.

### 19.9 Editor and tool integration

**\[TOOL-18\]** The language server, debug adapter, build server, assistant server and command-line tool SHALL use one analysis engine, so that they report identical diagnostics and results for the same inputs (\[TOOL-5\]).

**\[TOOL-19\]** `brio lsp` SHALL implement the Language Server Protocol, including diagnostics, completion, hover, signature help, inlay hints, go-to-definition, find references, rename, document and workspace symbols, folding, semantic tokens, formatting, code actions and code lenses, for Brio code and for embedded languages (21.5).

**\[TOOL-20\]** `brio debug` SHALL implement the Debug Adapter Protocol, supporting breakpoints (including conditional breakpoints), stepping, stack inspection, variable inspection and expression evaluation. The toolchain SHALL provide LLDB and GDB formatters that display core types, optionals, results, unit-annotated values and range types in Brio syntax. Stepping and breakpoints SHALL work in embedded-language text through source maps (\[LANG-8\]).

**\[TOOL-21\]** `brio bsp` SHALL implement the Build Server Protocol, exposing packages, modules, dependencies, build targets and test targets.

**\[TOOL-22\]** The toolchain SHALL provide test discovery, through a documented language-server extension and through `brio test --list --format json`, covering `test` items, property tests and `bench` items.

**\[TOOL-23\]** Features that the standard protocols cannot express, including proof status, the assurance ledger, flag usage and test discovery, SHALL be provided through documented, versioned protocol extensions. A client that does not support an extension SHALL still receive all standard features.

**\[TOOL-24\]** The Brio project SHALL publish and maintain, in step with each edition:

1. a Tree-sitter grammar, with injection rules that give embedded literals the grammar of their language;
2. a TextMate grammar;
3. editor packages or configurations for at least VS Code (published to both the Visual Studio Marketplace and Open VSX), JetBrains IDEs, Zed, Neovim, Emacs, Helix and Sublime Text.

**\[TOOL-25\]** The toolchain SHALL be able to emit: diagnostics as SARIF 2.1.0; test results as JUnit XML; coverage as LCOV and Cobertura; and, with `brio index`, a code-navigation index in SCIP format.

**\[TOOL-26\]** `brio fmt --stdin` SHALL read source from standard input and write the formatted result to standard output.

**\[TOOL-27\]** The language server SHALL also be available compiled to WebAssembly, for use in browser-based editors and a playground.

**\[TOOL-28\]** `brio mcp` SHALL implement the Model Context Protocol, giving AI assistants read-only access to types, symbols, diagnostics, proof results, the assurance ledger and `brio explain` content. Any tool it offers that writes files SHALL apply only edits allowed by \[EXT-7\].

**\[TOOL-29\]** The toolchain installer SHALL install the compiler, language server, debug adapter and debugger formatters as one matched version. A package MAY pin its toolchain version in `brio-toolchain.toml`, and every command and editor integration SHALL use the pinned version.

**\[TOOL-30\]** Code blocks marked as Brio in documentation comments are *documentation tests*. `brio test` SHALL compile and run them, except blocks marked `no_run`, which are only compiled, and blocks marked `compile_fail(code)`, which SHALL be rejected with the given diagnostic code. `brio lint` SHALL warn about public items without documentation.

**\[TOOL-31\]** `brio build --timings` SHALL report the time spent per module and per compilation phase, including verification, `comptime` evaluation and embedded-language expansion, so that regressions in build speed can be found.

**\[TOOL-32\]** `brio build --deny-warnings`, or `warnings = "deny"` in the manifest table `[lints]`, SHALL turn every warning into an error.

**\[TOOL-33\]** *Trap map.* `brio ledger --traps` SHALL list every place in the program where a trap can still occur in production: unproven overflow, bounds, range and conversion checks; contracts in check mode; `expect` calls; `unbounded` waits; allocations outside `@alloc(fallible)` modules; recursion without a stack bound; and calls into foreign code. Each entry shows its location, the condition, and the change that would remove it, such as a proof, a `get` instead of an index, or a `try_` form. `brio ship` MAY be configured to fail when the list for a module is not empty.

**\[TOOL-34\]** *Workspace trust.* Until the user marks a workspace as trusted, the language server and editor integrations SHALL provide only services that execute no code from the workspace or its dependencies, such as parsing, highlighting and formatting. Compile-time evaluation, embedded languages, tool extensions and the assistant server are disabled until then.

**\[TOOL-35\]** *Assistant server boundaries.* The assistant server (\[TOOL-7\]) SHALL never return decrypted licensed layers, or values of `Secret` or `Sensitive` types. It SHALL mark all text originating in dependencies as untrusted data together with its source. Its tool descriptions are fixed by the toolchain release and signed, and a change to them requires the user's approval again.

**\[TOOL-36\]** *Isolated compile-time code.* Compile-time evaluation, embedded languages and tool extensions SHALL run in a separate, confined process without network access, with a read-only view of the files they declare, and with deterministic limits on steps, memory and output.

## 20 Scripts

A *script* is a single Brio source file that is run directly, without a package. Scripts use the same language as packages; this clause defines only what differs.

### 20.1 Script files

*Syntax:*

```text
ScriptFile  ::= Shebang? Frontmatter? (Item | Stmt)*
Shebang     ::= "#!" any characters up to the end of the first line
Frontmatter ::= "---" NEWLINE TOML-text NEWLINE "---" NEWLINE
```

**\[SCR-1\]** A `.brio` file is compiled as a script when it is run with `brio run`, executed directly, or entered in the REPL, and it is not inside the `src/` directory of a package. Source files of packages SHALL NOT contain top-level statements.

**\[SCR-2\]** If the first line of any `.brio` file begins with `#!`, the lexer treats that line as a comment.

**\[SCR-3\]** A script MAY contain, after the optional shebang line, a *frontmatter* block delimited by lines consisting only of `---`. Its content is TOML and MAY contain the manifest tables `[dependencies]`, `[rust-dependencies]`, `[python-dependencies]`, `[verify]` and `[comptime]` with the meanings of 17.1 and 20.5. A `[package]` table is ill-formed in frontmatter.

**\[SCR-4\]** A script SHALL NOT import other source files. Code shared between scripts is placed in a package and declared as a dependency, which MAY be a local path dependency.

**\[SCR-5\]** Dependencies are referred to by their package name with each hyphen replaced by an underscore; for example, package `ward-registry` is the module path `ward_registry`. This rule applies equally in packages.

### 20.2 Top-level statements

**\[SCR-6\]** Items and statements MAY be interleaved at the top level of a script. The statements, in order, form the body of an implicit function `main` whose return type is `() ! AnyError`, or which is `async` if any top-level statement contains `await`. A script SHALL NOT declare `main`.

**\[SCR-7\]** Items in a script are visible throughout the script (\[ITEM-1\]). Bindings made by top-level statements are local variables of the implicit `main`: items SHALL NOT refer to them.

*Note:* This keeps functions in a script free of hidden global state, so they can be moved into a package unchanged.

**\[SCR-8\]** Apart from 20.1 and 20.2, a script has exactly the meaning it would have as the root module of an executable package whose `main` contains its top-level statements. A script SHALL NOT use `@profile(critical)`.

### 20.3 AnyError

**\[SCR-9\]** The core type `AnyError` holds any value whose type implements `Error`, together with an optional chain of context messages. `From` converts every such type to `AnyError`, so `try` works on any `Result` in a function that returns `T ! AnyError`. The method `.context(message)` on a `Result` adds a message to its error. `AnyError` MAY be used in packages as well as scripts.

**\[SCR-10\]** When the implicit `main` of a script returns `.Err(e)`, the implementation SHALL write e's context messages and cause chain to the standard error stream and terminate with exit status 1.

### 20.4 Running scripts

**\[SCR-11\]** `brio run <file> [-- args…]` compiles and runs a script. Executing a script file that has a shebang line naming `brio` has the same effect.

**\[SCR-12\]** The implementation SHALL cache compiled scripts, keyed on the script's source, its resolved dependencies, its frontmatter and the toolchain version. Running an unchanged script SHALL NOT recompile it.

**\[SCR-13\]** `brio script lock <file>` writes the resolved dependency versions to `<file>.lock` next to the script. When that file exists, it is used as `Brio.lock` is used in \[PKG-3\].

**\[SCR-14\]** `brio repl` evaluates each entry as script statements or items, in order, in one implicit `main`. Bindings persist between entries. An entry that is ill-formed is rejected without affecting earlier entries.

**\[SCR-15\]** `brio script promote <file>` creates a package whose manifest contains the script's frontmatter and whose `src/main.brio` contains the script's items and a `main` function containing its statements.

### 20.5 Python interoperability

**\[SCR-16\]** `use python m.{names}` imports names from the Python module m. After `use`, `python` is a contextual keyword. Python packages are declared in `[python-dependencies]` using PEP 508 requirement strings. By default, Python code runs in a separate, confined worker process that holds only the capabilities passed to it explicitly, so that Python cannot exceed the program's permissions (\[CAP-10\]). Running Python inside the program's own process requires `python.in-process = true` in `[permissions]` and the `ffi` package permission, is recorded in the assurance ledger as unrestricted authority, and is forbidden in programs containing `critical` modules. How the Python interpreter and its version are selected is implementation-defined.

**\[SCR-30\]** The Python bridge SHALL be built on a version of its binding library with no known soundness advisory, recorded in the assurance ledger, and SHALL use a Python runtime with a global interpreter lock unless the bridge has been reviewed for free-threaded Python.

**\[SCR-31\]** A script run without an interactive terminal SHALL carry, in its frontmatter, a lock section pinning the content hashes of all its dependencies. Without one, `brio run` resolves interactively and offers to add it. The compiled-script cache is private to the user and keyed as in \[PKG-26\].

**\[SCR-17\]** Python values have the core type `PyObject`. Attribute access, calls, indexing and operators on `PyObject` values are resolved at runtime and return `PyObject ! PyError`, or `T ! PyError` for a conversion to a Brio type T. A Python exception never propagates as a trap; it becomes a `PyError`.

**\[SCR-18\]** Conversions between `PyObject` and Brio types are explicit, with `T.from_py(obj)` and `value.to_py()`, and exist for primitive types, `String`, `List`, `Map`, `Set`, `Option`, tuples, and records and enums that derive `python.Convert`.

**\[SCR-19\]** `@python.export` on a `public` function or type of a package makes the toolchain build a Python extension module exposing it. Contracts of exported functions are checked in check mode, and a violation raises a Python exception in place of a trap. Traps other than contract violations terminate the process as in 10.4.

**\[SCR-20\]** Python interoperability is forbidden under the `critical` profile. Every `use python` and `@python.export` SHALL be listed in the `brio ship` report.

## 21 Embedded languages

An *embedded language* is a language, such as GraphQL, SQL, JSON or UI markup, whose text appears inside Brio source and is turned into Brio code at compile time by a library, not by the compiler.

### 21.1 Language items and literals

*Syntax:*

```text
LanguageDecl ::= "language" IDENT "=" Expr
EmbeddedLit  ::= "#" IDENT ( "{" EmbeddedText "}" | "\"" EmbeddedText "\""
                           | "\"\"\"" EmbeddedText "\"\"\"" )
```

**\[LANG-1\]** A `language` item binds a name, in the value namespace, to the value of a constant expression whose type implements the core trait `Language`. That value is the language's *configuration*.

**\[LANG-2\]** An *embedded literal* is `#` followed, with no whitespace, by the name of a language item and then by `{` or `"`. `#` is a punctuator used only for this purpose. The literal is an expression.

**\[LANG-3\]** The text of `#name{ … }` extends to the `}` that balances the opening `{`. Braces inside holes, and inside the string and comment forms declared by the language's `lexical` configuration, are not counted. The text of `#name"…"` extends to the first `"` not preceded by `\`, and `#name"""…"""` follows the rules of \[LEX-22\]. No Brio escape sequences are processed: the language receives the text as written.

**\[LANG-4\]** The language's `lexical` configuration declares its *hole* syntax: `${ … }` or `{ … }`. The content of a hole is a Brio expression, parsed and type-checked by the compiler. Holes MAY contain further embedded literals.

### 21.2 Expansion

**\[LANG-5\]** To compile an embedded literal, the implementation calls, at compile time, the configuration's `parse`, then `check`, then `expand` methods. `expand` returns a value of type `Code` containing one expression, which replaces the literal.

**\[LANG-6\]** The hole expressions of a literal are evaluated exactly once each, left to right, before the expansion's code runs, and the expansion refers to their values. A language cannot change the evaluation order of, duplicate or omit a hole.

**\[LANG-7\]** The code produced by an expansion SHALL NOT contain `unsafe` blocks, `static var` accesses or `extern` declarations. It is subject to hygiene (\[CT-13\]) and to every rule of this specification, including the profile of the enclosing module, exactly as if it had been written by hand.

**\[LANG-8\]** `Code` values produced by a language carry *source maps* to positions in the embedded text. Diagnostics, trap locations and debugger positions in expanded code SHALL be reported at those positions.

### 21.3 The language context

**\[LANG-9\]** Language methods are `comptime` functions and are subject to 13.2. Through the `LangContext` argument they MAY: obtain the type of each hole and the expected type of the literal; read files that the configuration declares through its `inputs` method, which are thereby compile-time inputs (\[CT-5\]); report diagnostics; and build source-mapped `Code`.

**\[LANG-10\]** A language item whose configuration names files, and whose language implements `expand_file`, produces one module per file, named after the language item and the file's stem.

### 21.4 Failures, diagnostics and caching

**\[LANG-11\]** If a language method traps, exceeds a compile-time limit, or returns malformed `Code`, the implementation SHALL report a diagnostic at the literal that names the language's package and version. It SHALL NOT fail in any other way.

**\[LANG-12\]** Diagnostics reported by a language carry codes of the form `package:CODE` and follow Clause 23. A language MAY implement `explain`, which `brio explain` then uses.

**\[LANG-13\]** The result of an expansion SHALL depend only on the language package and version, the configuration value, the literal's text, the types of its holes, its expected type and the contents of its declared inputs. The implementation SHALL reuse the result while these are unchanged.

### 21.5 Editor services and stability

**\[LANG-14\]** A language MAY implement the services `lint`, `format`, `highlight`, `complete`, `hover`, `signature_help`, `inlay_hints`, `definition`, `references`, `rename`, `symbols`, `folding`, `code_actions` and `fixes`. The compiler, `brio lint`, `brio fmt`, `brio fix` and the language server SHALL use every service a language provides, inside its embedded literals and files, under the same sandbox and with time limits.

**\[LANG-15\]** The `Language` trait carries an API version. A conforming implementation SHALL support every API version published for the editions it supports, so that languages keep working across compiler releases.

*Example:*

```brio
language db = sql.Postgres(migrations: "db/migrations")

let rows = try await conn.query(#db"SELECT name, ward FROM patients WHERE id = ${id}")
```

### 21.6 Lints and formatting

**\[LANG-16\]** A language declares its lints in its `lints` constant, each with an identifier, a default level (`allow`, `warn` or `deny`) and a summary. Its `lint` method reports diagnostics only for declared lints.

**\[LANG-17\]** Lint levels MAY be set per language item in the manifest table `[lints.<item>]`, and per item or statement with `@lint.allow(item: lint)`, `@lint.warn(item: lint)` and `@lint.deny(item: lint)`. A `deny` diagnostic is an error. Lint diagnostics follow Clause 23.

**\[LANG-18\]** When `format` is provided, `brio fmt` SHALL replace the embedded text with its result, re-indented to the enclosing Brio code, leaving hole contents to be formatted as Brio. The implementation SHALL re-parse the formatted text and SHALL reject the result, with a diagnostic naming the language, if the parsed tree differs from that of the original text or if formatting the result again changes it.

**\[LANG-19\]** Every service and requirement in this clause applies equally to every embedded language, whatever language it defines. A language package MAY also ship a Tree-sitter grammar and a TextMate grammar; the Brio grammars' injection rules (\[TOOL-24\]) SHALL use them for its literals, so that editors highlight any embedded language even without the language server.

**\[LANG-20\]** A language MAY report, for each hole, the Brio type it expects, derived from its configuration and inputs: for example, the column type of a SQL parameter or the argument type of a GraphQL field. The compiler SHALL check each hole against that type and SHALL use it for completion inside the hole (\[COMP-6\]).

## 22 Observability, testing and reliability

### 22.1 Principles

**\[OBS-1\]** No feature of this clause SHALL require a proprietary service. Every signal SHALL be exportable in the open formats named in this clause, and every feature SHALL work with local storage only. `brio observe` SHALL display locally all the signals this clause defines.

**\[OBS-2\]** Every flag, experiment, metric, fault point and telemetry export in a program is declared in source, so that the toolchain can list and test all of them.

### 22.2 Telemetry

*Syntax:*

```text
MetricDecl ::= "metric" IDENT ":" Type ("(" ArgList ")")?
```

**\[OBS-3\]** `@traced(p1, …)` on a function creates a span for each call, named after the function's path, with the listed parameters as attributes. `log.trace`, `log.debug`, `log.info`, `log.warn` and `log.error` record structured events whose field names and values are checked at compile time; every recorded value SHALL implement the core trait `TelemetryValue`.

**\[OBS-4\]** A `metric` item declares a metric of kind `Counter`, `UpDownCounter`, `Gauge` or `Histogram`. Its label type SHALL be a tuple of types with a finite set of values (payload-free enums, `Bool` and integer range types), with at most 1,000 combinations, unless the item carries `@unbounded_labels(reason)`.

**\[OBS-5\]** Trace context SHALL propagate automatically across `await`, `spawn` and `tasks` blocks, and across the Rust bridge into and out of spans of the Rust `tracing` crate. Across processes it SHALL use W3C Trace Context.

**\[OBS-6\]** Traces, metrics and logs SHALL be exportable with OTLP, configured at runtime through the standard OpenTelemetry environment variables or configuration file. Recording SHALL NOT block for an unbounded time, SHALL use bounded memory, and SHALL NOT trap; data that cannot be exported is dropped and counted in a self-monitoring metric.

**\[OBS-7\]** A manifest MAY set a minimum log level per build configuration; calls below it SHALL be removed at compile time. In `critical` modules, telemetry SHALL write to a fixed-size ring buffer and SHALL NOT allocate.

### 22.3 Sensitive data

**\[OBS-8\]** Values of the core type `Sensitive[T]` format as `‹redacted›` under `Display` and `Debug` and do not implement `TelemetryValue`. `s.expose(reason: Str)` returns the inner value; every call to `expose` SHALL be listed in the assurance ledger (23.1).

### 22.4 Crash reports

**\[OBS-9\]** When a trap occurs, the default trap handler, and a custom handler that calls the core function `crash.write(info)`, SHALL write a crash report without allocating. It contains the `TrapInfo`, the stack of the trapping task, the most recent spans and events from a bounded buffer, the values of all flags, and the build identifier. Its format is documented JSON.

**\[OBS-10\]** On the next start, a program configured to do so SHALL export stored crash reports as OTLP log events. `brio ship` SHALL archive debug information by build identifier, and `brio symbolicate` SHALL turn a crash report's addresses into source locations.

### 22.5 Observe mode

**\[OBS-11\]** A fourth verification mode, `observe`, evaluates obligations at runtime like `check`, but reports a violation as a telemetry event instead of trapping, and execution continues. `observe` SHALL NOT apply to obligations whose violation could cause undefined behavior, and is forbidden in `critical` modules.

### 22.6 Feature flags and experiments

*Syntax:*

```text
FlagDecl         ::= "flag" IDENT ":" Type "=" Expr
                   | "flag" IDENT ":" Type "{" FlagMember (SEP FlagMember)* SEP? "}"
FlagMember       ::= ("default" | "audience") "=" Expr
ExperimentDecl   ::= "experiment" IDENT ":" Type "=" Expr "by" Type
                   | "experiment" IDENT ":" Type "by" Type "{" ExperimentMember (SEP ExperimentMember)* SEP? "}"
ExperimentMember ::= ("control" | "metrics" | "guardrails" | "hypothesis"
                     | "audience" | "eligible" | "balance_by") "=" Expr
```

**\[OBS-12\]** A `flag` declares a runtime-configurable value of type `Bool`, an integer type, `String` or a payload-free enum, with a constant default. `f.value()` returns its current value.

**\[OBS-13\]** An `experiment` declares a payload-free enum of variants, an *assignment unit* type, and a *control* variant: the value of `=` in the short form, or of `control` in the long form. `e.variant(for: u)` returns the variant assigned to unit u under Clause 48 and records the exposure. Clause 48 defines runs, assignment, analysis and control of experiments.

**\[OBS-14\]** Flag values come from a provider implementing the core trait `FlagProvider`, whose semantics follow the OpenFeature specification. The default provider reads a local file. Evaluation never fails: on any provider error, the default value is returned and the error is recorded.

**\[OBS-15\]** `brio test --flags all` SHALL run the tests under every combination of flag values reachable from the tested code, or, with `--flags pairs`, under a set of combinations covering every pair of values. The toolchain SHALL report flags that are never read and flags whose value never varies.

**\[OBS-16\]** In a program with `critical` modules, flag values SHALL be fixed before the operational phase begins (16.3), and every obligation in `critical` code SHALL be proven for every value of every flag it reads.

### 22.7 Benchmarks

*Syntax:*

```text
BenchDecl ::= "bench" STRING Block        (* the block contains exactly one measure Block *)
```

**\[OBS-17\]** `brio bench` SHALL run each `bench` item with warm-up and repeated sampling of its `measure` block, and SHALL report the median, a 95% confidence interval, allocation counts and, where the hardware permits, instruction counts.

**\[OBS-18\]** Baselines are stored in the repository. A manifest MAY set regression thresholds, and `brio ship` SHALL fail when a benchmark exceeds its threshold (\[TOOL-15\]).

### 22.8 Faults and deterministic simulation

**\[OBS-19\]** `@fault_point("name")` on a function returning `T ! E`, where E implements the core trait `Injectable`, makes the function a *fault point*. Every standard-library input/output function and every bridge call is a fault point. When fault injection is active, a fault point MAY return an injected error, be delayed or be cancelled, as configured.

**\[OBS-20\]** Fault injection is active only in test builds, in simulation, and in builds that explicitly enable it in the manifest. In all other builds, fault points SHALL have no runtime cost. Fault injection SHALL NOT be enabled in a build containing `critical` modules.

**\[OBS-21\]** `brio sim --seed n` SHALL run the program with simulated clocks, timers, network, file system, randomness and task scheduling, so that the same seed and inputs reproduce the same execution. Input and output performed by foreign code is not simulated; each such boundary SHALL be reported as *unsimulated*.

**\[OBS-22\]** `brio mutate` SHALL apply small changes to the program and report each change that no test and no contract detects.

### 22.9 Inspection and provenance

**\[OBS-23\]** `brio inspect` SHALL attach read-only to a running program and show its tasks, locks, channels and flag values. It is available in release builds only when enabled in the manifest.

**\[OBS-24\]** `brio ship` SHALL record build provenance in SLSA format and a software bill of materials in SPDX or CycloneDX format (\[TOOL-14\]).

## 23 Assurance and diagnostics

### 23.1 The assurance ledger

**\[ASSR-1\]** Every successful build SHALL write an *assurance ledger* to `.brio/assurance.json`, in a documented format, and SHALL print a summary of it. `brio ledger` SHALL display it, and `brio ledger --diff <version>` SHALL show how it changed since a previous release.

**\[ASSR-2\]** The ledger SHALL record:

1. the properties guaranteed by the language for this build (23.2);
2. for each module, its profile and the number and kinds of obligations proven, checked, observed and assumed;
3. every assumption, trusted boundary, `unsafe` block, `unsafe fn`, `unsafe impl` and foreign declaration, with its justification;
4. every Rust crate, C library and Python module used, with version, and every bridge claim;
5. every embedded language used, with package and version;
6. every `Sensitive.expose` call, every `@unbounded_labels` metric, and every unsimulated boundary;
7. whether fault injection is enabled, and the toolchain's qualification status.

**\[ASSR-3\]** *Completeness of the ledger:* every construct whose correctness a conforming implementation neither proves nor checks at runtime SHALL appear in the ledger. A program's correctness therefore depends only on its contracts being the right ones, on the items in its ledger, and on the hardware and operating system.

**\[ASSR-4\]** The ledger is deterministic. When a release's ledger adds items to categories 3 to 7 of \[ASSR-2\] compared with the previous release, `brio ship` SHALL highlight each addition.

### 23.2 What a successful build guarantees

**\[ASSR-5\]** For a program accepted by a conforming implementation, assuming only the items in its ledger:

1. no execution has undefined behavior or a data race;
2. every value has a valid value of its type, and no reference is null or dangling;
3. every error result is handled or explicitly discarded, and every `match` is exhaustive;
4. every runtime failure is either an error value or a trap, and every trap produces a report (\[OBS-9\]);
5. in code in prove mode, no condition of \[TRAP-1\] can occur and every contract holds;
6. in `critical` code, additionally, every loop and call terminates, stack use is within the reported bound, and no memory is allocated after initialization.

**\[ASSR-6\]** A build does not guarantee, and the summary SHALL NOT claim, that contracts express the intended requirements, that ledger items behave as assumed, that hardware and operating system behave correctly, that resources suffice outside `critical` code, or that timing requirements are met.

### 23.3 Compatibility

**\[ASSR-7\]** A program that conforms to an edition SHALL be accepted, with the same meaning, by every later conforming implementation. Every conforming implementation SHALL support every edition published before its release. Changes that would reject or change the meaning of conforming programs are made only in a new edition, and the toolchain SHALL provide `brio migrate --edition` to move code to it.

### 23.4 Diagnostic style

These rules extend 19.1 and apply equally to diagnostics reported by embedded languages.

**\[DIAG-1\]** Every error diagnostic SHALL contain, in this order:

1. a *headline* that names the problem in plain words, with the source location and code;
2. a *what* statement that uses the program's own names and types;
3. the relevant source excerpt, with the problem marked;
4. a *why it matters* statement: the consequence for the running program if the code were accepted;
5. a *how to fix* section with one or more suggested fixes, ranked, the first being the recommended one, whenever the implementation can suggest one.

**\[DIAG-2\]** When a diagnostic concerns two types that differ, it SHALL show both as written in Brio syntax and mark the part that differs. When a proof obligation fails and the prover finds a counterexample, the diagnostic SHALL show the counterexample as Brio values.

**\[DIAG-3\]** Each suggested fix SHALL be labelled as either *certain*, meaning `brio fix` may apply it automatically, or *needs review*. A fix SHALL be labelled certain only if applying it yields a program that is not ill-formed for the same reason and does not change the meaning of any other code.

**\[DIAG-4\]** An implementation SHALL NOT report a diagnostic that is only a consequence of another diagnostic it reports, and SHALL order diagnostics so that causes precede their consequences.

**\[DIAG-5\]** Diagnostics SHALL use the terms defined in this specification, and SHALL NOT use terms internal to an implementation, such as "lifetime", "monomorphization", "unification", "trait object" or intermediate-representation names.

**\[DIAG-6\]** The prose of a diagnostic SHOULD NOT exceed three short paragraphs; further explanation belongs to `brio explain`.

**\[DIAG-7\]** A successful build SHALL print the summary of \[ASSR-1\], stating separately what is guaranteed, proven, checked at runtime, and taken on trust.

**\[DIAG-8\]** With `--diagnostics plain`, diagnostics SHALL use no color, no box-drawing characters and no alignment that depends on character width, and SHALL describe the marked source region in words, so that they are usable with screen readers.

**\[DIAG-9\]** The prose of diagnostics and `brio explain` pages SHALL be stored separately from the compiler's code, so that it can be translated. Diagnostic codes, rule identifiers and code excerpts are never translated.

## 24 Tool extensions

A *tool extension* is a package that adds developer tooling for Brio code: lints, code actions, editor features, explanations, documentation, commands and test support.

### 24.1 Declaring extensions

**\[EXT-1\]** Extensions are declared in the manifest table `[extensions]`, which extends \[PKG-2\], with the syntax of `[dependencies]`. They are development dependencies: they are not linked into the program.

**\[EXT-2\]** An extension's root module declares one value of a type implementing the core trait `Extension`. The trait carries an API version, and \[LANG-15\] applies to it in the same way.

### 24.2 Services

**\[EXT-3\]** An extension MAY implement any of the following services. The toolchain SHALL use every service an extension provides:

| Service | Used by |
| --- | --- |
| `lint`, with declared `lints` | The compiler, `brio lint`, the language server |
| `code_actions` | The language server and `brio fix` |
| `hover`, `inlay_hints`, `code_lenses`, `completions`, `snippets`, `highlight` | The language server |
| `explain` | `brio explain` for the extension's codes |
| `doc` | `brio doc` |
| `commands` | New `brio` subcommands |
| `generators`, `fixtures` | `brio test` and property tests (\[TOOL-11\]) |

**\[EXT-4\]** Services receive, through a `ToolContext`, a read-only view of the analyzed program: syntax trees with types, symbols and their uses, contracts, the call graph, flags and experiments, and the assurance ledger.

### 24.3 Soundness and sandboxing

**\[EXT-5\]** Extensions are analysis only. Nothing an extension does SHALL change whether a program is accepted, other than by reporting `deny` diagnostics, nor the meaning of any accepted program.

**\[EXT-6\]** Services other than `commands` are `comptime` functions subject to 13.2. A command MAY read the package's files and SHALL write only to the paths it declares; it SHALL NOT access the network unless the user passes `--allow-net`.

**\[EXT-7\]** Every edit offered by a code action or fix SHALL be analyzed before it is offered, and an edit that would make the program ill-formed SHALL NOT be offered. \[DIAG-3\] applies to the labelling of such edits.

**\[EXT-8\]** If a service traps, exceeds its limits or returns malformed results, the toolchain SHALL report a diagnostic naming the extension and version and SHALL continue without that result.

### 24.4 Lints, results and visibility

**\[EXT-9\]** Extension lints are declared and configured as in \[LANG-16\] and \[LANG-17\], using the manifest table `[lints.<extension>]` and codes of the form `extension:lint`. Their diagnostics follow Clause 23.

**\[EXT-10\]** An extension's results SHALL depend only on the extension's version, its configuration and the analyzed program, and SHALL be identical in the compiler, the language server and continuous integration. The toolchain SHALL cache them per module.

**\[EXT-11\]** The extensions used and their lint levels SHALL be recorded in the `brio ship` report.

## 25 Interoperability with other languages

This clause defines interoperability with languages other than Rust (Clause 18) and Python (20.5). It provides three channels: *native*, *component* and *service*.

### 25.1 Interfaces as the source of truth

**\[FFI-1\]** Bindings for other languages SHALL be generated from the public interface files of 17.3, and from no other description, so that a binding always matches the code it binds.

**\[FFI-2\]** A binding generator for a target language is a tool extension (Clause 24) that implements the core trait `BindingGenerator`. The toolchain SHALL use the generator for each language listed in the manifest table `[foreign]`.

### 25.2 The native channel

**\[FFI-3\]** `@foreign` on a public item exports it through the *Brio C ABI*: a documented, versioned mapping of Brio types to C-compatible representations, in which strings and slices are pointer–length pairs, opaque records are handles, errors are tagged results and traits are tables of function pointers. The Brio C ABI SHALL remain compatible within each of its major versions.

**\[FFI-4\]** `brio bind` SHALL produce, for each target language, a package in that language's usual packaging format, whose API is idiomatic for the language: errors appear as the language's error mechanism, optionals as its nullable types, `async` functions as its asynchronous functions, traits as interfaces that code in that language may implement, and opaque records as objects with deterministic release (such as `AutoCloseable`, `IDisposable` or `deinit`).

**\[FFI-5\]** Distinct types, unit-annotated types and range types SHALL be represented in the target language by named types where the language allows it. Every value of these types entering Brio SHALL be checked as in \[RUST-10\].

### 25.3 Soundness at the native boundary

**\[FFI-6\]** Every value entering Brio through a binding SHALL be validated before Brio code sees it: string encoding, enum discriminants, ranges, and non-null handles and pointers. Invalid values SHALL be reported as the target language's error, not passed on.

**\[FFI-7\]** A binding SHALL NOT allow a borrow to outlive the call that created it. Values given to foreign code are owned handles, or borrows valid only for the duration of the call or of a callback.

**\[FFI-8\]** When a foreign caller violates a precondition of an exported function, the binding SHALL report it as the target language's error, and the Brio function SHALL NOT be entered. Other traps behave as in 10.4.

**\[FFI-9\]** An exception or panic raised in foreign code called from Brio SHALL be caught at the boundary. It becomes an error value if the called function's Brio type is a `Result`, and a trap otherwise. It SHALL NOT unwind through Brio frames.

**\[FFI-10\]** A binding SHALL confine objects of types that do not implement `Send` to the thread that created them, and SHALL report use from another thread as the target language's error.

### 25.4 Importing C and C++

*Syntax:*

```text
UseDecl ::= ... | "use" ("c" | "cpp") STRING "as" IDENT ("{" FnDecl* "}")?
```

**\[FFI-11\]** `use c "header.h" as m` imports the declarations of a C header, and `use cpp "header.hpp" as m` imports those of a C++ header, as the module m. The implementation SHALL parse headers with a conforming C or C++ front end. After `use`, `c` and `cpp` are contextual keywords.

**\[FFI-12\]** Imported functions are `unsafe fn` unless the `use c` or `use cpp` declaration has an *API notes* block describing them: `use c "vendor/pump_hal.h" as hal { … }`. The block contains Brio declarations of the header's functions and types, which MAY give parameter modes, ownership, origins, units, ranges, nullability and contracts. API notes are ordinary Brio source. A function fully described by API notes is safe to call; its contracts are checked at each call in check mode and assumed in prove mode, and it is recorded as a bridge claim (\[RUST-15\]).

**\[FFI-13\]** C++ classes, methods, `std::string` and `std::vector` are imported; C++ templates are usable only at instantiations declared as in \[RUST-14\]. C++ exceptions are handled as in \[FFI-9\].

### 25.5 The component channel

**\[FFI-14\]** The toolchain SHALL be able to compile a package to a WebAssembly component targeting WASI 0.3 or later. Its exported interface SHALL be expressed in WIT, derived from its public interface files, and `use wit "package/interface"` SHALL import a WIT interface as a Brio module.

**\[FFI-15\]** A trap inside a component SHALL end only that component instance; the host observes it as an error.

### 25.6 The service channel

**\[FFI-16\]** Interoperability with services in other processes uses embedded languages (Clause 21) for schema languages such as Protocol Buffers, OpenAPI, GraphQL and AsyncAPI. Values received from a service are validated as in \[FFI-6\].

### 25.7 Visibility

**\[FFI-17\]** Every binding, imported header, API notes block, component import and service schema SHALL be recorded in the assurance ledger (\[ASSR-2\]) and in the `brio ship` report.

## 26 Supply-chain security

### 26.1 Vulnerability auditing

**\[SEC-1\]** `brio audit` SHALL check every dependency of a package, and of a script, against vulnerability advisories. This covers Brio packages, Rust crates, Python packages, C and C++ libraries named in manifests or API notes, embedded-language packages, tool extensions and binding generators.

**\[SEC-2\]** Advisories SHALL be read in the OSV format. The toolchain SHALL support at least the Brio advisory database, RustSec, the Python advisory database and the GitHub Advisory Database, and SHOULD match C and C++ libraries against CVE records. Dependencies are identified by package URLs (`pkg:brio/…`, `pkg:cargo/…`, `pkg:pypi/…`).

**\[SEC-3\]** Advisory databases MAY be mirrored locally. Given the same mirror snapshot and the same lockfiles, `brio audit` SHALL produce the same results.

**\[SEC-4\]** When an advisory names affected functions, `brio audit` SHALL use the program's call graph, including calls across bridges, to report each advisory as *reachable*, *not reachable* or *undetermined*. An advisory without function information is reported as undetermined.

**\[SEC-5\]** `brio audit` SHALL write a VEX document, in OpenVEX or CSAF format, stating for each advisory whether the package is affected, with the justification derived from \[SEC-4\]. The toolchain SHALL NOT state "not affected" for an undetermined advisory unless a waiver says so.

**\[SEC-6\]** `brio audit --release <version>` SHALL audit the software bill of materials recorded for an earlier release (\[TOOL-14\]), so that published releases can be rechecked when new advisories appear.

### 26.2 Gates and waivers

**\[SEC-7\]** `brio build` SHALL warn about reachable and undetermined advisories. `brio ship` SHALL fail when any reachable or undetermined advisory meets or exceeds the severity threshold configured in the manifest, unless a waiver covers it.

**\[SEC-8\]** A waiver is an entry in `audit.toml` giving the advisory, a reason, an owner and an expiry date. Expired waivers have no effect. Every waiver SHALL be recorded in the assurance ledger (\[ASSR-2\]).

**\[SEC-9\]** `brio audit fix` SHALL propose the smallest upgrades that resolve reachable advisories within the version requirements of the manifest, and SHALL show the resulting changes to the lockfile and the ledger before applying them.

### 26.3 Package capabilities

**\[SEC-10\]** For every dependency, the toolchain SHALL compute the package permissions it uses (\[CAP-8\]) and the capabilities it receives (\[CAP-13\]).

**\[SEC-11\]** Every package permission is denied by default and granted per package in `[permissions.packages]` (Clause 41). A dependency that uses a permission it has not been granted is an error.

**\[SEC-12\]** When a dependency update adds a capability, the toolchain SHALL report it before the lockfile changes, and `brio ship` SHALL list it (\[ASSR-4\]).

### 26.4 Integrity and provenance

**\[SEC-13\]** No code from a dependency SHALL run during dependency resolution, download or installation. Code from dependencies runs at build time only as `comptime` code, embedded languages or tool extensions, within their sandboxes (\[CT-5\], \[LANG-9\], \[EXT-6\]).

**\[SEC-14\]** Packages published to a Brio registry SHALL be signed and SHALL carry build provenance (\[OBS-24\]). The toolchain SHALL verify signatures and provenance on download and SHALL refuse a package whose content hash differs from its lockfile entry.

**\[SEC-15\]** The toolchain SHALL support recording reviews of dependency versions, stored in the repository, and MAY require a recorded review for every dependency version used in a release.

**\[SEC-16\]** When a dependency is added whose name closely resembles that of a widely used package, the toolchain SHALL warn before adding it.

### 26.5 Own advisories

**\[SEC-17\]** `brio advisory new` SHALL draft an OSV advisory for a package, with its affected versions and functions derived from the package's interface history. The `brio ship` report SHALL include the release's vulnerability status and VEX document.

## 27 Computational model and formal semantics

### 27.1 Expressive power

**\[MODEL-1\]** Brio is Turing complete. Functions declared `unbounded` (Clause 39) may run for any number of steps and use any amount of memory. Real executions are bounded by the resources of the target; exhausting them is a trap (\[TRAP-2\]), never undefined behavior. Every place where a program may run without bound is declared and listed in the assurance ledger (\[BND-19\]).

**\[MODEL-2\]** Code in `critical` modules is deliberately *not* Turing complete. Its functions are bounded (\[CRIT-5\]), non-recursive (\[CRIT-3\]) and use memory bounded at compile time (\[CRIT-2\], \[CRIT-10\]), except for the outermost task loops permitted by \[CRIT-5\], each iteration of which is bounded. It can express every computation whose running time and memory can be bounded in advance.

*Note:* No language can be both Turing complete and able to decide every property of every program. Brio keeps Turing completeness available everywhere through `unbounded`, makes every use of it visible, and requires boundedness wherever predictability matters. Functions are bounded by default, so most code carries a termination guarantee without any annotation.

### 27.2 Brio Core

**\[MODEL-3\]** *Brio Core* is a small calculus into which every Brio construct is translated (*elaborated*). The companion document *Brio Core* defines its syntax, typing rules and operational semantics, and defines the elaboration of each construct of this specification. Where a construct's dynamic semantics in this specification is given by elaboration, the Core semantics governs.

### 27.3 Machine-checked soundness

**\[MODEL-4\]** The Brio project SHALL maintain a machine-checked mechanization of Brio Core, in a proof assistant such as Rocq with the Iris separation logic or Lean 4, together with proofs of:

1. *type soundness*: a well-typed program that uses no `unsafe` operations has no undefined behavior, in the semantic, logical-relations style of RustBelt;
2. *data-race freedom* for such programs under the memory model of 14.7;
3. *borrow soundness*: the rules of Clause 9 imply the aliasing discipline of the Core memory model;
4. *verification soundness*: every obligation that the verification-condition generator reports as valid holds in the Core semantics (\[CON-23\]);
5. *contract erasure*: \[CON-20\] holds.

**\[MODEL-5\]** The Core memory model SHALL define, for `unsafe` code, exactly which aliasing patterns are undefined behavior, as a tree-structured permission model in the style of Tree Borrows. Every core library item that uses `unsafe` operations and is documented as critical-compatible SHALL have a mechanized proof that it is semantically well-typed.

**\[MODEL-6\]** The toolchain SHALL provide `brio interp`, an interpreter for Brio Core that detects every undefined behavior defined by \[MODEL-5\] at runtime. `brio test --interp` SHALL run tests under it.

**\[MODEL-7\]** For `critical` builds, the toolchain SHALL support *back-to-back testing*: running the same tests under `brio interp` and as compiled code, and reporting any difference in observable behavior.

**\[MODEL-8\]** *Mechanism independence.* The mechanization of \[MODEL-4\] SHALL include a proof that, for every choice of passing mechanism that \[OWN-9\] permits, including passing in registers, by copying, by copying in and back, and by address, every well-typed program without `unsafe` operations has the same observable behavior.

## 28 Conformance and qualification

**\[CONF-1\]** The Brio project SHALL publish a *conformance test suite* in which every normative rule of this specification is covered by at least one test that a conforming implementation passes and, for rules that make programs ill-formed, at least one test it rejects. Every test SHALL name the rules it covers.

**\[CONF-2\]** An implementation claiming conformance SHALL publish its conformance-suite results for each release and target.

**\[CONF-3\]** An implementation SHALL publish a machine-readable *known problems* list: defects that affect the meaning of programs, each linked to the rules it affects and the constructs that trigger it. `brio ship` SHALL check the program against the known-problems list of its toolchain version and SHALL fail if the program uses an affected construct, unless a waiver as in \[SEC-8\] covers it.

**\[CONF-4\]** For each safety standard it claims to support (such as IEC 62304, IEC 61508, ISO 26262 or DO-178C with DO-330), an implementation offered for safety-critical use SHALL provide a *qualification kit*: the tool classification, this specification with traceability from each rule to tests, the conformance results, the known-problems list, and the procedures for using the tool within its qualified limits.

**\[CONF-5\]** Core library items documented as critical-compatible form a *certified core subset*, whose own requirements, tests and proofs (\[MODEL-5\]) are part of the qualification kit.

**\[CONF-6\]** The artifacts of a qualified toolchain record its qualification status, qualified targets and qualified options. Using an option or target outside these limits marks the build as unqualified (\[GEN-15\]).

*Note:* This structure follows established practice: a qualified Rust toolchain has been assessed under IEC 62304 Class C, ISO 26262 and IEC 61508, based on a structured language specification and a separately certified core-library subset.

## 29 Language evolution

**\[EVO-1\]** Changes to the language are made through a public proposal process. Each accepted proposal SHALL state the rules it adds or changes, its effect on existing programs, and its effect on soundness proofs (\[MODEL-4\]).

**\[EVO-2\]** A language feature is *stable* or *preview*. A preview feature is usable only when named in the manifest table `[preview]`, produces a warning at each use, is forbidden in `critical` modules and in published packages, and MAY change or be removed in any release.

**\[EVO-3\]** A stable feature, library item or diagnostic behavior SHALL be deprecated with `@deprecated` for at least one full edition before it is removed, and SHALL be removed only in a new edition. `brio migrate --edition` SHALL rewrite code to avoid removed features wherever the rewrite is mechanical.

**\[EVO-4\]** A new edition is published no more often than every three years. Each edition SHALL have at least one *long-term support* release line, maintained with fixes for at least five years; qualified toolchains (\[CONF-4\]) derive from long-term support releases.

**\[EVO-5\]** A proposal SHALL NOT be accepted if it would make any program that conforms to an edition ill-formed, or change its meaning, under that same edition (\[ASSR-7\]).

## 30 Freestanding and embedded targets

### 30.1 Library tiers

**\[EMB-1\]** The standard library is organized in three tiers: `core`, which needs neither an allocator nor an operating system; `alloc`, which adds heap allocation; and `std`, which adds operating-system services. A package declares the tier it needs with `tier = "core" | "alloc" | "std"` in `[package]`; the default is `std`.

**\[EMB-2\]** A package SHALL NOT use items from a tier above its declared tier, and SHALL NOT depend on a package that declares a higher tier. Script mode requires `std`.

### 30.2 Targets

**\[EMB-3\]** A *target* is described by a target specification in TOML, giving at least: architecture, CPU and CPU features, endianness, pointer width, supported atomic widths, floating-point unit, ABI, and whether an operating system is present. Built-in targets have predefined specifications; custom targets are declared in the package.

**\[EMB-4\]** On a target without atomic operations of some width, atomic types of that width are unavailable. Code that uses them is ill-formed on that target, rather than emulated silently.

### 30.3 Program entry, memory layout and interrupts

**\[EMB-5\]** On a target without an operating system, the program's entry point is a function marked `@entry` with signature `fn name() -> Never`. The target specification or the package declares its memory regions in `[target.memory]`; the toolchain generates the linker configuration from it. `@section("name")` places an item in a named section.

**\[EMB-6\]** A function marked `@interrupt(name, priority: n)` is an interrupt handler. It SHALL have no parameters, SHALL NOT be `async`, SHALL satisfy `@no_alloc`, and SHALL NOT block. Its worst-case stack use is included in the bound of \[CRIT-10\].

**\[EMB-7\]** Data shared between interrupt handlers and tasks SHALL be of an atomic type or wrapped in the core type `Shared[T, ceiling: p]`. Access is written `s.lock(|v| { … })`; the compiler SHALL reject access from any handler or task whose priority exceeds p, and the implementation raises the interrupt priority to p for the duration of the access. This prevents data races between interrupts without disabling all interrupts.

### 30.4 Memory-mapped input and output

**\[EMB-8\]** The core type `Volatile[T]` denotes a memory location accessed with volatile semantics: every `read` and `write` is performed exactly once, with the width of T, in program order relative to other volatile accesses, and is never merged, split, removed or speculated.

**\[EMB-9\]** Peripheral registers SHOULD be described with an embedded language for device descriptions, such as CMSIS-SVD, which generates typed register blocks whose fields have range types and whose access rights (read-only, write-only) are enforced by the type system. Creating a register block at a raw address is an unsafe operation.

### 30.5 Execution

**\[EMB-10\]** The toolchain SHALL provide an executor for `async` tasks on targets without an operating system, driven by interrupts, with a static task set as in \[CRIT-9\]. Freestanding executables SHALL declare a trap handler (\[TRAP-6\]).

**\[EMB-11\]** On a target with more than one core, raising the interrupt priority does not exclude other cores. There, `Shared[T, ceiling: p]` SHALL also acquire a spin lock after raising the priority, unless every handler and task that accesses the value is pinned to the same core with `@core(n)`. The implementation SHALL reject a program in which two cores could access the same `Shared` value without exclusion.

## 31 Memory allocation

### 31.1 Allocators

**\[ALLOC-1\]** The core trait `Allocator` provides `allocate(layout) -> Ptr[U8] ! AllocError`, `deallocate(ptr, layout)` and `grow(ptr, old, new) -> Ptr[U8] ! AllocError`, with contracts stating the validity of pointers and layouts. Implementing it is an `unsafe impl`.

**\[ALLOC-2\]** Every allocating core type takes an allocator parameter with a default, such as `List[T, A: Allocator = Global]`. `Global` is the program's global allocator, chosen by at most one `static` marked `@global_allocator`, or by the implementation's default.

### 31.2 Fallible allocation

**\[ALLOC-3\]** Every core operation that may allocate has a fallible form that returns `T ! AllocError` instead of trapping, named with the prefix `try_`: for example `list.try_push(x)` and `String.try_with_capacity(n)`.

**\[ALLOC-4\]** In a module marked `@alloc(fallible)`, calling an allocating operation that is not a `try_` form is ill-formed. Such modules handle memory exhaustion as an ordinary error.

### 31.3 Bounded storage

**\[ALLOC-5\]** The `core` tier provides fixed-capacity types that never allocate: `BoundedList[T, N]`, `BoundedString[N]`, `BoundedMap[K, V, N]` and `BoundedQueue[T, N]`. Operations that could exceed capacity return an error or have a precondition that the capacity suffices, so that in prove mode overflow is proven impossible.

**\[ALLOC-6\]** `Pool[T, N]` is a static pool of N slots. `pool.acquire(value) -> PoolBox[T] ! PoolFull` places a value in a free slot, and the slot is released when the `PoolBox` is dropped. Pools do not allocate from the heap, so they MAY be used in the operational phase of `critical` programs.

**\[ALLOC-7\]** `Arena[A]` allocates from a region that is freed all at once. References into an arena carry the arena's origin, so they cannot outlive it.

**\[ALLOC-8\]** For `critical` programs, the toolchain SHALL report the worst-case heap use of the initialization phase when it can be bounded, and the total static storage of all pools and bounded collections.

## 32 Workspaces, registries and vendoring

**\[WS-1\]** A *workspace* is a directory whose `Brio.toml` contains a `[workspace]` table listing member packages. Members share one `Brio.lock`, one build output directory and the workspace's `[lints]`, `[verify]`, `[permissions]` and `[extensions]` settings, which members MAY override only to make them stricter.

**\[WS-2\]** A dependency MAY name a registry: `ward-registry = { version = "2", registry = "internal" }`. Registries are declared in `[registries]`, or in user configuration for credentials. The registry protocol is documented, content-addressed and signed (\[SEC-14\]); any conforming registry MAY be used, and none is privileged.

**\[WS-3\]** A registry MAY be mirrored. A mirror SHALL serve content identical to the original, which the toolchain verifies by content hash.

**\[WS-4\]** `brio vendor` SHALL copy every dependency, of every ecosystem, together with a snapshot of the advisory databases, into a directory in the workspace. `brio build --offline` SHALL then succeed without network access and produce artifacts identical to an online build (\[PKG-23\]).

**\[WS-5\]** A published version MAY be *yanked*. New resolutions SHALL NOT select a yanked version; existing lockfiles that pin it continue to build, with a warning.

## 33 Serialization and schema evolution

### 33.1 Encoding and decoding

**\[SER-1\]** The core traits `Encode` and `Decode` describe how a value maps to a format-independent data model of booleans, integers, floats, strings, bytes, sequences, maps, records and variants. They MAY be derived. Formats are libraries; the `alloc` tier SHALL provide JSON and CBOR.

**\[SER-2\]** Decoding SHALL NOT produce an invalid value. A decoded value is checked against every constraint of its type: ranges, units, distinct types, enum variants, UTF-8 validity and type invariants. A violation is a decoding error, never a trap.

**\[SER-3\]** A value of type `Sensitive[T]` or `Secret[T]` SHALL NOT be encoded unless the field carries `@encode_sensitive(reason)`, and every such attribute is listed in the assurance ledger.

**\[SER-4\]** A unit-annotated value is encoded as a number in its declared unit. A format MAY also record the unit, and decoding SHALL reject a recorded unit that differs from the declared one.

**\[SER-5\]** Types that derive `Encode` or `Decode` and cross the Rust bridge SHALL also implement serde's `Serialize` or `Deserialize` in the shim, with the same data model.

### 33.2 Schema evolution

**\[SER-6\]** A record or enum marked `@schema(version: n)` has a versioned schema. Its history is expressed in source, through the attributes of \[SER-8\] and the `@migrate` functions, so no separate schema file exists. `Brio.lock` records a hash of each schema version, and a build SHALL fail when a schema changes without its version increasing, unless it is run with `--accept-schema-change`. Compatibility (\[SER-7\]) is checked from the attributes and against the schema of the previously published version.

**\[SER-7\]** Between consecutive versions, the toolchain SHALL classify every change as *backward compatible* (new code reads old data), *forward compatible* (old code reads new data), both, or *breaking*. It SHALL reject a breaking change unless the version increases and a migration function `@migrate(from: n)` converts old values.

**\[SER-8\]** The schema attributes are: a default value for every added field; `@removed(since: n)` for removed fields, whose names are never reused; `@renamed(from: "old")`; and, for non-frozen enums, a required fallback variant marked `@unknown`, which receives variants the decoder does not recognize.

## 34 Real-time and fault tolerance

### 34.1 Timing

**\[RTF-1\]** A task entry function MAY declare `@period(p)`, `@deadline(d)` and `@wcet(budget)`, with durations as unit-annotated values, for example `@period(10<ms>)`.

**\[RTF-2\]** For `critical` programs on targets whose timing the implementation documents as analyzable, the toolchain SHALL compute a static upper bound on the execution time of each task iteration and each interrupt handler, and SHALL reject a program whose bound exceeds its declared `@wcet`. On other targets, timing evidence MAY be supplied from measurements; such evidence SHALL be marked *measured, not proven* in the assurance ledger.

**\[RTF-3\]** When every task has a period, deadline and execution-time bound, the toolchain SHALL perform response-time analysis for fixed-priority scheduling, including blocking from `Shared` ceilings (\[EMB-7\]) and mutexes (\[CRIT-9\]), and SHALL reject an unschedulable task set.

### 34.2 Safe states

**\[RTF-4\]** An enum marked `@state_machine(safe: .S)` defines a state machine whose transitions are given by a function marked `@transition` of type `fn(take Self, Event) -> Self`. In prove mode, the toolchain SHALL prove that the transition function is total and that, from every state, the event marked `@fault_event` leads to the safe state S within the number of steps declared as `max_steps`.

### 34.3 Detecting hardware faults

**\[RTF-5\]** The core type `Redundant[T]` stores a value together with a diverse copy, such as its bitwise complement, and checks them on every read. A mismatch is a trap of kind `CorruptedData`.

**\[RTF-6\]** The core function `vote(a: T, b: T, c: T) -> T ! NoMajority` returns the majority value of three independently computed results. The toolchain SHALL NOT merge the three computations when they are marked `@diverse`, so that a transient fault cannot affect all three.

**\[RTF-7\]** The core type `Watchdog` wraps a hardware or software watchdog. A `critical` program with periodic tasks SHOULD declare one, and the toolchain SHALL warn when a periodic task's deadline exceeds the watchdog timeout.

## 35 Language-level security

### 35.1 Secrets

**\[LSEC-1\]** The core type `Secret[T]` holds cryptographic secrets. It has all the properties of `Sensitive[T]` (\[OBS-8\]); in addition its storage SHALL be overwritten with zeros when the value is dropped or moved from, it implements neither `Copy` nor `Clone` except through the explicit `.duplicate()`, and its equality comparison executes in time independent of the contents.

**\[LSEC-2\]** Where the operating system allows, the implementation SHOULD prevent memory holding `Secret` values from being written to swap or core dumps.

### 35.2 Constant-time code

**\[LSEC-3\]** In a function marked `@constant_time`, values derived from `Secret` values SHALL NOT determine a branch, a loop bound, an array index, a memory address or the divisor of a division. Violations are ill-formed. The implementation SHALL generate code that preserves this property on the targets it documents as supporting constant-time code, and SHALL reject `@constant_time` on other targets.

### 35.3 Untrusted input

**\[LSEC-4\]** Standard-library functions that read from the network, files, environment variables, command-line arguments, process output or foreign callers return values wrapped in the core type `Untrusted[T]`. The only operations on an `Untrusted[T]` are `validate(f: fn(T) -> U ! E) -> U ! E`, decoding with \[SER-2\], and `.expose_unvalidated(reason)`, which is listed in the assurance ledger.

**\[LSEC-5\]** Holes in embedded literals are passed to the language as data, never as text. A language SHALL generate code that transmits hole values as parameters, such as SQL bind parameters, or escapes them for its context, such as HTML text. A language MAY declare a *raw* hole that splices text; every use of a raw hole requires `unsafe("justification")`, so that injection cannot happen by accident.

### 35.4 Cryptography

**\[LSEC-6\]** The standard library provides cryptographic primitives only through vetted implementations, and SHALL NOT implement its own. The implementations used, their versions and any certification status (for example FIPS 140-3 validation) SHALL be recorded in the assurance ledger.

## 36 Numerics and SIMD

### 36.1 Exact numeric types

**\[NUM-1\]** The `core` tier provides `Fixed[B, frac: F]`, a binary fixed-point number stored in the integer type B with F fractional bits. Addition and subtraction are exact and checked as integers are (\[TYPE-4\]). Multiplication and division compute an exact intermediate result and then round it with the rounding mode given explicitly, `a.mul(b, .HalfEven)`, or with the module's declared default `@rounding(mode)`; a module with no default SHALL state the mode at every rounding operation.

**\[NUM-2\]** The `core` tier provides `Decimal`, a 128-bit decimal floating-point type with the semantics of IEEE 754-2019 decimal128, and the `alloc` tier provides `BigInt` and `BigDecimal` of unbounded precision. `BigInt` and `BigDecimal` are not critical-compatible.

**\[NUM-3\]** Fixed-point and decimal types work with units, range types and distinct types exactly as the primitive numeric types do.

**\[NUM-4\]** Conversions that may lose precision take an explicit rounding mode: `.HalfEven`, `.HalfAwayFromZero`, `.TowardZero`, `.Up` or `.Down`.

### 36.2 Reproducible mathematics

**\[NUM-5\]** The elementary functions of the core library (`sqrt`, `exp`, `log`, `sin`, `cos`, `atan2`, `pow` and the others it defines) SHALL return the correctly rounded result for `F32` and `F64` under `@fp(strict)`, so that results are bit-identical on every target and in compile-time evaluation (\[CT-3\]).

**\[NUM-6\]** The core library's random-number generators are deterministic given their seed, and the default generator of a program is seeded from the simulation seed under `brio sim` (\[OBS-21\]).

### 36.3 SIMD

**\[NUM-7\]** The core type `Simd[T, N]`, where T is a primitive numeric type and N a supported lane count, is a vector of N lanes. Lane-wise operators have the semantics of the scalar operators, including overflow checks in check mode (\[TYPE-4\]); the wrapping and saturating operators apply lane-wise. `Mask[N]` values select lanes, and reductions such as `sum`, `min` and `max` are provided.

**\[NUM-8\]** `Simd` code SHALL produce the same results on every target, whether or not the target has vector instructions. Target-specific intrinsics are available in `core.arch` as unsafe functions.

## 37 Time, dates and text

### 37.1 Time

**\[TIME-1\]** `Duration` is the unit-annotated type `I64<ns>`. Durations are written with units, for example `250<ms>`, and convert with `as` (\[UNIT-8\]).

**\[TIME-2\]** `Instant` is a point on a monotonic clock that never goes backward; subtracting two instants gives a `Duration`. `SystemTime` is a point in UTC as reported by the system clock, which MAY jump. Timeouts and deadlines SHALL be expressed with `Instant`.

**\[TIME-3\]** `SystemTime` counts time as POSIX time does, without leap seconds. The core library documents how leap seconds on the system clock are observed.

### 37.2 Dates and time zones

**\[TIME-4\]** The `std` tier provides civil types `Date`, `Time`, `DateTime` and `ZonedDateTime`, using the IANA time zone database. The version of the database used by a build SHALL be recorded in the assurance ledger, and the database MAY be updated without recompiling.

**\[TIME-5\]** Arithmetic on civil types that can be ambiguous, such as adding a month or a local time that falls in a daylight-saving gap, SHALL require an explicit resolution policy and SHALL NOT silently choose one.

### 37.3 Text

**\[TIME-6\]** The `std` tier provides Unicode normalization, case mapping, segmentation and collation, and localized message formatting following Unicode MessageFormat 2, using versioned Unicode CLDR data recorded in the assurance ledger.

## 38 Standard library structure

**\[LIB-1\]** The companion document *Brio Standard Library* specifies every standard library item: its signature, contracts, complexity, tier (\[EMB-1\]), allocation behavior, and whether it is critical-compatible. This clause fixes its structure.

**\[LIB-2\]** The `core` tier SHALL contain at least: primitive types and their operations; `Option`, `Result` and the core traits of 4.12 and 12.10; slices, strings and formatting without allocation; bounded collections and pools (\[ALLOC-5\], \[ALLOC-6\]); cells and atomics; `Volatile` and `Shared` (\[EMB-7\], \[EMB-8\]); `Fixed`, `Decimal` and `Simd`; `Duration` and `Instant`; `Sensitive`, `Secret`, `Untrusted`, `Redundant` and `Watchdog`; and the trap and crash-report interfaces.

**\[LIB-3\]** The `alloc` tier adds `Box`, `Rc`, `Arc`, `List`, `String`, `Map`, `Set`, `Arena`, `BigInt`, `BigDecimal`, and JSON and CBOR encoding.

**\[LIB-4\]** The `std` tier adds files and paths, processes, environment, networking, threads and the default executor, `SystemTime` and civil time, text services, `AnyError`, arguments, logging and telemetry export, flags, HTTP, and cryptography through vetted implementations (\[LSEC-6\]).

**\[LIB-5\]** Every standard library item SHALL carry contracts for its preconditions, so that misuse is diagnosed as a contract violation or disproved in prove mode, never as undefined behavior.

**\[LIB-6\]** `Map[K, V]` and `Set[T]` iterate in insertion order, independent of hash values, and use keyed hashing that resists collision attacks. Their iteration order is therefore identical across runs, targets and simulations (\[OBS-21\]).

## 39 Bounded and unbounded functions

Every function in Brio either provably finishes or says, in its declaration, that it might not. This makes every place where a program could run forever visible, while keeping the language Turing complete (\[MODEL-1\]).

### 39.1 The two kinds of function

**\[BND-1\]** Every function, method and closure is *bounded* or *unbounded*. A function is unbounded if it is declared with the qualifier `unbounded`; a closure is unbounded if its body does not satisfy the rules for bounded functions. All other functions and closures are bounded. The implicit `main` of a script (\[SCR-6\]), `main` itself, and `test` and `bench` items are unbounded without being declared so.

**\[BND-2\]** *The guarantee.* Every execution of a bounded function, started in a state that satisfies its preconditions, ends after finitely many steps, by returning, failing or trapping. This holds assuming only that the clock advances, that the task running it continues to be scheduled, and that the items recorded under \[BND-19\] hold.

**\[BND-3\]** The rules of this clause are conservative: they accept only functions whose boundedness they establish. A function that is not declared `unbounded` and that the rules cannot establish as bounded is ill-formed. The diagnostic SHALL name the construct responsible and offer the remedies that apply: a `for` loop, a `bounded` or `decreases` clause, or the `unbounded` qualifier.

**\[BND-4\]** This clause does not apply to `comptime` functions, embedded-language methods or tool-extension services, whose evaluation is limited by \[CT-6\], \[LANG-11\] and \[EXT-8\].

### 39.2 Loops

**\[BND-5\]** In a bounded function, every loop SHALL be one of:

1. a `for` loop whose iterator type implements the core trait `Finite`;
2. a loop with a `bounded n` clause (\[CON-15\]);
3. a loop with a `decreases` clause (\[CON-14\]).

**\[BND-6\]** `Finite` is implemented by ranges, arrays, slices, the iterators of core collections, and bounded collections. It is preserved by iterator adapters that yield no more elements than their input, such as `map`, `filter`, `zip` and `enumerate`, and `take(n)` makes any iterator finite. A user-defined iterator implements `Finite` by providing `fn remaining(self) -> USize` and a proof, in prove mode, that every call of `next` that returns `.Some` strictly decreases `remaining()`; otherwise only by `unsafe impl Finite`.

**\[BND-7\]** A `decreases` clause that establishes boundedness is always evaluated or proven: it is never in `observe` mode, and a violation in check mode traps, which ends the execution. If the clause's item is in `assume` mode, the boundedness of the function is *assumed*.

### 39.3 Calls

**\[BND-8\]** A bounded function SHALL call only bounded functions, directly, through function values, through trait methods and through bridges, and SHALL `await` only futures of bounded `async` functions.

**\[BND-9\]** Boundedness is part of function types. `fn(A) -> R` and `Fn(A) -> R` are bounded; `unbounded fn(A) -> R` and `unbounded Fn(A) -> R` may be either. A bounded function or closure converts implicitly to the corresponding unbounded type, never the reverse.

**\[BND-10\]** A trait method is bounded unless the trait declares it `unbounded`. An implementation of a bounded trait method SHALL be bounded. An implementation of an unbounded trait method MAY be either.

**\[BND-11\]** The methods of core traits that the language calls implicitly are bounded, so their implementations SHALL be bounded: `Drop.drop`, the operator traits of 12.10, `PartialEq`, `PartialOrd`, `Index`, `IndexMut`, `Contains`, `Iterator.next`, `IntoIterator.into_iter`, `From`, `Clone`, `Hash`, `Default`, `Display`, `Debug`, `Encode`, `Decode` and `Finite.remaining`. Implicit calls therefore never hide unboundedness.

### 39.4 Recursion

**\[BND-12\]** The *conservative call graph* of a program has a node for every bounded function, method and closure, and an edge from f to g when:

1. f calls g directly;
2. f calls a trait method m through a dynamic value or a generic bound, and g is any implementation of m in the program;
3. f calls a value of function type τ, and g is any function or closure in the program that is converted to a value of a type compatible with τ, where types are compatible if they could be equal after substituting generic parameters.

Edges of kinds 2 and 3 are *indirect*. The graph over-approximates every call that can happen at runtime.

**\[BND-13\]** Every cycle in the conservative call graph SHALL consist only of direct edges. Each function on such a cycle SHALL either declare `decreases` measures, such that at every call along the cycle the callee's measure is lexicographically smaller than the caller's measure on entry and every component is non-negative, or satisfy \[BND-14\]. A cycle containing an indirect edge makes the program ill-formed; the diagnostic SHALL show the cycle and suggest writing the recursion directly, for example with a `for` loop, or declaring a function `unbounded`.

*Note:* Rule 3 of \[BND-12\] is what makes the check sound for recursion through stored callbacks and self-application: a function that calls a stored value of its own type is in a cycle with itself, whatever the storage.

**\[BND-14\]** A function needs no `decreases` clause for *structural recursion*: every recursive call passes, in place of a parameter p, a value obtained by pattern matching on p that is strictly contained in p's value through enum payloads, record fields, `Box` and core collections. Recursion through `Rc`, `Arc`, cells, locks or fields of reference type is not structural, because such data can be cyclic.

**\[BND-15\]** The implementation SHALL check \[BND-13\] over each package as far as the package's own code allows, and over the whole program no later than when building an executable, a test or a benchmark. It MAY compute the whole-program check from per-package summaries, provided the result is the same.

### 39.5 Waiting and foreign code

**\[BND-16\]** An operation that waits for an event outside the program's control, such as acquiring a lock, receiving from or sending to a channel, network input or output, waiting for a process, or waiting for user input, is bounded only when it has a deadline or timeout, as in `mutex.lock_within(50<ms>)` or `channel.recv_within(d)`. The core library declares the forms without a deadline `unbounded`. Local file-system operations are bounded under the assumption that the storage device responds; that assumption is recorded under \[BND-19\].

**\[BND-17\]** Calls to Rust, C, C++, Python, WebAssembly components and services are unbounded, unless the bridge declaration or API notes mark the called function `@bounded`. Such a mark is a claim recorded under \[BND-19\].

### 39.6 Contracts, profiles and evidence

**\[BND-18\]** `@pure` functions, and every function called from a contract expression, SHALL be bounded. This keeps the logic of contracts consistent: a non-terminating function such as `f(x) = f(x) + 1` could otherwise be used to prove anything.

**\[BND-19\]** The assurance ledger (\[ASSR-2\]) SHALL list every unbounded function, every boundedness assumed under \[BND-7\], every `unsafe impl Finite`, every `@bounded` claim on foreign code, and the number of local file-system operations assumed to complete.

**\[BND-20\]** The mechanization of \[MODEL-4\] SHALL include a proof that the rules of this clause are sound: in Brio Core, every function accepted as bounded satisfies \[BND-2\].

**\[BND-30\]** *Complete call graph.* The call graph of \[BND-12\] SHALL be computed after compile-time evaluation and monomorphization, and SHALL include implicit calls: drop at the end of a scope, operator and conversion traits, dereference, iteration, formatting, asynchronous wakers and the trap handler. Every entry point through which foreign code can call Brio code (the Brio C ABI, Python callbacks, component exports) is a node reachable from an unbounded caller unless a claim of \[BND-17\] states otherwise.

**\[BND-31\]** *Termination certificates.* For `critical` modules, the compiler SHALL produce a termination certificate, a well-founded measure for every loop and every cycle of the call graph, which the proof checker of \[CON-26\] validates.

### 39.7 Termination of the compiler

**\[BND-21\]** Trait resolution SHALL search to a fixed maximum depth, 128 by default and configurable in the manifest, and SHALL report a diagnostic when the depth is reached. Every other analysis of the compiler SHALL either use an algorithm that terminates on every input or be subject to a documented limit whose exhaustion is a diagnostic.

**\[BND-22\]** Together with \[CT-6\], \[LANG-11\], \[EXT-8\] and \[CON-25\], \[BND-21\] guarantees that every invocation of a conforming implementation ends, with a result or a diagnostic.

*Example:*

```brio
fn depth(t: Tree) -> Int = match t {             // bounded: structural recursion
    .Leaf        => 0
    .Node(l, r)  => 1 + max(depth(l), depth(r))
}

fn gcd(a: UInt, b: UInt) -> UInt                 // bounded: measure decreases
    decreases b =
    if b == 0 { a } else { gcd(b, a % b) }

fn wait_for_pump(ch: Channel[Status]) -> Status ! Timeout =
    ch.recv_within(2<s>)                         // bounded: waits with a deadline

unbounded fn serve(listener: Listener) -> () ! NetError {   // declared: runs until stopped
    loop { handle(try listener.accept()) }
}
```

## 40 Completion and suggestions

This clause applies to conforming toolchains. It covers every suggestion shown while code is written: completion lists, inline suggestions accepted with Tab, and generated code such as match arms.

### 40.1 Soundness of suggestions

**\[COMP-1\]** Every suggestion SHALL be *valid in context*: inserting it SHALL NOT introduce any name-resolution, visibility, type, parameter-mode, unit, profile or capability error. Placeholders for arguments not yet written are the only permitted incompleteness.

**\[COMP-2\]** Suggestions are *type-directed*. When the position has an expected type, only candidates of that type, or candidates that can produce it through member access or a call, are offered, and exact matches rank first. Suggestions for unit-annotated and range types SHALL respect units and ranges; for example, a literal is suggested with its unit.

**\[COMP-3\]** An accepted suggestion SHALL insert everything the language requires to make it valid: argument labels (\[FN-2\]), call-site markers `mut`, `move` and `out` according to parameter modes (\[OWN-12\]), `try` or `await` where the result type requires them, and a `use` declaration when the item is not yet imported.

**\[COMP-4\]** Items that are `unsafe`, `unbounded`, deprecated or preview-only SHALL be marked as such in the suggestion. Items that the current profile forbids, such as `unbounded` functions in a bounded context or heap allocation in `critical` code, SHALL NOT be offered.

**\[COMP-5\]** Inline suggestions from any source, including AI assistants through `brio mcp`, SHALL be analyzed before they are shown. A suggestion that would make the program ill-formed SHALL NOT be shown, and suggestions not produced by the toolchain's own analysis SHALL be visibly labelled as such.

### 40.2 Coverage

**\[COMP-6\]** Completion SHALL be provided for at least:

1. names in scope, module paths, and items importable from dependencies;
2. fields, methods and associated items of every type, including generic types, dynamic values, optional chains (`?.`), references and bridged types;
3. implicit members (`.Variant`) from the expected type;
4. labeled arguments and record initializer fields, with required fields first;
5. patterns, including generating all missing arms of an exhaustive `match`;
6. trait implementations, including stubs for all missing members with their contracts;
7. contract expressions, including `result`, `error`, `old` and parameter names;
8. attributes and their arguments, unit expressions, format specifications, and keys in `Brio.toml`, `audit.toml` and script frontmatter;
9. text inside embedded literals, through the language's `complete` service (\[LANG-14\]), and Brio expressions inside their holes, using the expected hole type the language reports;
10. Rust and C items through bridges and API notes, and Python items through type stubs where available, other Python suggestions being labelled *unverified*.

**\[COMP-7\]** Completion for dependencies SHALL use their interface files (17.3), so that it works identically for packages distributed without source.

### 40.3 Stability

**\[COMP-8\]** For the same program text, cursor position and toolchain version, completion SHALL return the same suggestions in the same order. Ranking SHALL depend only on the program and on documented factors: type match, scope nearness, and frequency of use within the workspace.

**\[COMP-9\]** While a prefix is being typed, suggestions that still match SHALL keep their relative order.

### 40.4 Speed and robustness

**\[COMP-10\]** The parser SHALL recover from every syntax error and produce a syntax tree for any input, so that completion works in incomplete code.

**\[COMP-11\]** Completion SHALL NOT wait for code generation, proofs, whole-program checks or the analysis of function bodies other than the one being edited. Analysis SHALL be incremental and demand-driven, so that editing one function re-analyzes only what depends on it.

**\[COMP-12\]** On the reference hardware documented by the implementation, completion SHOULD return its first results within 50 ms at the 95th percentile for a module of 10,000 lines in a workspace of 1,000,000 lines, and SHALL return within 200 ms, returning partial results if necessary. Embedded-language and extension services are given a time budget within this limit; results that miss the budget are omitted, not waited for.

**\[COMP-13\]** The conformance suite (\[CONF-1\]) SHALL include completion tests for \[COMP-1\] to \[COMP-9\], and implementations SHALL measure \[COMP-12\] in continuous integration and publish the results.

## 41 Capabilities and permissions

Brio code has no ambient authority: nothing can affect or observe the world outside the program unless it has been explicitly given permission. Three layers enforce this: capability values checked by the type system, package permissions checked by the toolchain, and confinement enforced by the operating system.

### 41.1 No ambient authority

**\[CAP-1\]** Every standard-library operation that reads or changes anything outside the program's own memory SHALL require a *capability value*, as an argument or as its receiver. The core capability types are: `Dir` and `File` (file system), `Net`, `Connect` and `Listen` (network), `Spawn` (processes), `Env` (environment variables), `Stdio` (standard input and output), `Clock` (reading the current time), `Entropy` (operating-system randomness) and `Device` (hardware).

**\[CAP-2\]** The following need no capability: memory allocation, creating tasks, trap and crash reporting (\[TRAP-10\]), and waiting with a deadline, whose timer the runtime provides. Recording telemetry and logs requires the core capability `Telemetry`, which is granted to every package by default and passed implicitly (\[CAP-16\]). A manifest MAY deny or narrow it per package in `[permissions.packages]`. Values derived from `Secret`, `Sensitive` or `Untrusted` values cannot be recorded through it (\[OBS-8\], \[LSEC-1\]), and only the program's configured exporters send recorded data anywhere (\[OBS-6\]).

**\[CAP-3\]** Capability values are *unforgeable*. They SHALL NOT be created except by the runtime at program start, by attenuation (41.2), or by `unsafe` code that the package is permitted to contain (\[CAP-8\]). Capability types do not implement `Copy`, `Encode`, `Decode` or `Default`, and cannot be created at compile time. They MAY be cloned explicitly to delegate authority.

**\[CAP-4\]** A program receives its capabilities as a parameter of its entry point: `fn main(caps: Caps)`. A script receives them as the implicit binding `caps` (\[SCR-6\]). Tests receive only the capabilities they declare with `@caps(…)`; the test harness provides temporary directories, loopback networking and controllable clocks.

### 41.2 Attenuation

**\[CAP-5\]** Every capability can be narrowed, and none can be widened. Examples: `dir.subdir("logs")`, `dir.read_only()`, `net.only(["api.hospital.local:443"])`, `env.only(["TZ"])`, `spawn.only(["/usr/bin/git"])`. The narrowed value authorizes a subset of what the original authorizes.

*Example:*

```brio
fn main(caps: Caps) -> () ! AnyError {
    let logs = caps.fs.subdir("/var/log/pump")
    let api  = caps.net.only(["api.hospital.local:443"])
    try monitor.run(logs, api)        // monitor can reach nothing else
}
```

### 41.3 Program permissions

**\[CAP-6\]** An executable declares the capabilities it may ever hold in the manifest table `[permissions]`, for example `net = ["api.hospital.local:443"]` and `fs.write = ["/var/log/pump"]`. A script declares them in a `[permissions]` table in its frontmatter, extending \[SCR-3\]. With no `[permissions]` table, a program holds no capabilities at all.

**\[CAP-7\]** The runtime creates the capabilities in `caps` to match the declared permissions. Deployment configuration and command-line options MAY narrow them further, but SHALL NOT widen them.

### 41.4 Package permissions

**\[CAP-8\]** Capability values cannot reach inside code that bypasses the type system. Therefore the following *package permissions* are denied to every dependency by default, and SHALL be granted per package in `[permissions.packages]`:

1. `unsafe`: containing `unsafe` blocks, functions or implementations;
2. `ffi`: calling Rust, C, C++, Python, components or services directly;
3. `comptime-read`: reading compile-time input files;
4. for foreign dependencies, each effect category the foreign code needs: `net`, `fs`, `spawn`, `env`, `device`.

A package that uses a permission it has not been granted is an error.

**\[CAP-9\]** When a dependency update requires a permission the previous version did not, the toolchain SHALL report it before changing the lockfile, and `brio ship` SHALL list it (\[SEC-12\]).

### 41.5 Operating-system confinement

**\[CAP-10\]** On targets whose operating system supports process confinement (for example Landlock and seccomp on Linux, the App Sandbox on macOS, AppContainer on Windows, or pledge and unveil on OpenBSD), the runtime SHALL confine the process at startup to the permissions of \[CAP-6\] and the foreign-code permissions of \[CAP-8\], so that foreign and `unsafe` code cannot exceed them either. Confinement SHALL use allow lists rather than deny lists; SHALL deny interfaces that bypass system-call filtering, such as `io_uring`, unless a granted capability requires them; and SHALL close or account for every file descriptor inherited before confinement. The confinement level achieved is recorded in the assurance ledger and in crash reports. Confinement is a second line of defence behind the capability rules of this clause, not a replacement for them. Where confinement is unavailable, 41.8 applies.

**\[CAP-11\]** A dependency that is not trusted at all MAY be run as a WebAssembly component (\[FFI-14\]), which can use only the capabilities passed to it.

### 41.6 Testing, simulation and audit

**\[CAP-12\]** Because every external effect passes through a capability, `brio sim` (\[OBS-21\]) and fault injection (\[OBS-19\]) SHALL operate by substituting simulated or faulty capabilities. A program that performs no foreign input or output is therefore fully simulatable.

**\[CAP-13\]** `brio caps` SHALL show, for each capability of the program, the path along which it flows from `main` to every function that uses it. The assurance ledger SHALL record the program's permissions and, for each package, the permissions granted and used.

**\[CAP-14\]** An operation attempted without the capability it needs is a compile-time error. The diagnostic SHALL name the capability required and show where the nearest available capability of that kind comes from.

**\[CAP-15\]** In the top-level statements of a script, which form its `main` (\[SCR-6\]), a standard-library call MAY omit its capability argument; the corresponding capability from `caps` is used. Functions declared in a script receive capabilities only as parameters, like all other functions.

### 41.7 Capability sugar

*Syntax:*

```text
UsesClause ::= "uses" UsesItem ("," UsesItem)*
UsesItem   ::= IDENT (":" Type)?
WithExpr   ::= ("with" WithItem)+ Block
WithItem   ::= IDENT "=" Expr
             | IDENT "." IDENT "(" ArgList? ")"      (* shorthand, [CAP-18] *)
```

**\[CAP-16\]** A function MAY declare the capabilities it needs in a `uses` clause, written after the return type and before any contracts. `uses net` is shorthand for `uses net: Net`; the shorthand names are `fs`, `net`, `spawn`, `env`, `stdio`, `clock`, `entropy` and `device`, each with its core capability type. Other capabilities are written with an explicit type: `uses logs: Dir`. Within the function, each name denotes the capability as an ordinary default-mode binding. `uses` and `with` are contextual keywords, and a NEWLINE before `uses` in a signature is ignored, as for `requires` (\[LEX-31\]).

**\[CAP-17\]** The *capability context* at a point in a program is the set of named capabilities available there: those of the enclosing function's `uses` clause, those bound by enclosing `with` blocks, and, in `main`, all capabilities granted to the program under their shorthand names. At a call to a function with a `uses` clause, each item is passed implicitly from the capability context by name and type. If the context has no capability with that name and type, the call is ill-formed (\[CAP-14\]).

**\[CAP-18\]** `with name = e { … }` evaluates e, which SHALL have the type of a capability, and makes it the capability named `name` in the context for the block, replacing any capability of that name. It does not declare a local binding, so \[STMT-4\] does not apply to it. *Shorthand:* `with name.f(args)` means `with name = name.f(args)`, and is permitted only when `name` is a capability in the context and f is a `@subset` or `@join` function of its type (\[CAP-30\], \[CAP-40\]), so that the shorthand can only limit or combine that capability. Several `with` clauses MAY precede one block, one per line, as in:

```brio
with net.only(["reports.hospital.local:443", "backup.hospital.local:443"])
with records.only_ward(icu)
{
    try nightly_report(icu)
}
```

**\[CAP-50\]** *Long stacks.* The lint `long-with-stack` SHALL warn when more than a configured number of `with` clauses (6 by default) precede one block, and SHOULD suggest a capability set (\[CAP-39\]) or a helper function that takes the capabilities it needs. It is configured in `[lints]`, for example `long-with-stack = { level = "warn", max = 6 }`. The limit of 64 stacked clauses in Annex H is independent of this lint.

**\[CAP-19\]** Implicit passing is sugar only: the program means exactly what it would mean with every capability passed explicitly, and `brio caps` (\[CAP-13\]) and the assurance ledger show the implicit flows exactly as explicit ones.

**\[CAP-20\]** A `uses` clause is part of the function's signature and interface. A trait method's implementations SHALL NOT use capabilities beyond those its trait declaration lists. Adding an item to the `uses` clause of an exported function is a major change (\[PKG-18\]).

*Example:*

```brio
fn fetch_vitals(id: PatientId) -> Vitals ! NetError
    uses net
{
    let body = try http.get("https://api.hospital.local/vitals/{id}")   // net passed implicitly
    return try Vitals.decode(body)
}

fn main() -> () ! AnyError {
    with net = net.only(["api.hospital.local:443"]),
         fs  = fs.subdir("/var/log/pump")
    {
        try monitor.run()        // monitor.run declares: uses net, fs
    }
}
```

### 41.8 When confinement is unavailable

**\[CAP-21\]** When a program is built for a target on which \[CAP-10\] cannot be applied, the compiler SHALL report a warning that names the target and states that the permissions of the program are enforced only by the type system and the toolchain.

**\[CAP-22\]** When `--deny-warnings` is given, or the manifest sets `warnings = "deny"` in `[lints]`, that warning is an error, as every warning is (\[TOOL-32\]).

**\[CAP-23\]** For a program containing `critical` modules, that condition is an error unless the build is given `--allow-unconfined "reason"`, or the manifest sets `allow-unconfined = "reason"` in `[permissions]`. The reason SHALL be non-empty and is recorded in the assurance ledger and the `brio ship` report.

**\[CAP-24\]** At startup, a program whose confinement is unavailable, or fails to apply on the host (for example because the running kernel lacks the required feature), SHALL record a warning event through its telemetry and write the warning to the standard error stream if one exists. A program containing `critical` modules that was built as confined SHALL NOT start if confinement fails at runtime; it traps before running any user code, unless it was built with the acknowledgement of \[CAP-23\].

**\[CAP-45\]** A capability SHALL NOT be stored in thread-local storage, and SHALL NOT be passed to Python or other foreign code except as an opaque handle that the foreign code cannot copy.

**\[CAP-46\]** `handle.revoke()` (\[CAP-34\]) returns only after every `with_cap` call in progress on the revoked capability has ended, so no operation begins or continues through a revoked capability.

**\[CAP-47\]** The `Clock` capability has a coarse form, with a resolution documented by the implementation, and a fine form. Functions marked `@constant_time` SHALL NOT receive the fine form.

**\[CAP-48\]** A program containing `critical` modules and built as confined SHALL state in `[permissions]` the minimum confinement features it requires, such as a minimum Landlock version, and SHALL NOT start on a host that cannot provide them (\[CAP-24\]).

### 41.9 User-defined capabilities

*Syntax:*

```text
CapabilityDecl ::= Attribute* Visibility? "capability" IDENT GenericParams? ("as" IDENT)?
                   "{" (RecordMember (SEP RecordMember)* SEP?)? "}"
```

**\[CAP-25\]** A `capability` declaration declares a nominal record type whose values are capabilities. `as name` gives it a shorthand name for `uses` and `with` (\[CAP-16\]). `capability` is a contextual keyword at the start of an item. The core capability types of \[CAP-1\] are declared this way in the core library, so built-in and user-defined capabilities follow the same rules.

**\[CAP-26\]** *Unforgeable.* A capability type is subject to \[CAP-3\]. Its memberwise initializer and fields are usable only in its own module, its *authority module*, whatever their visibility, and it SHALL NOT be constructed by `unsafe_cast`, compile-time evaluation or decoding. A type that contains a capability in a field is itself subject to \[CAP-3\].

**\[CAP-27\]** *No hiding places.* A capability, or a value containing one, SHALL NOT be stored in a `static`, a `static var` or a compile-time constant. Authority therefore flows only through parameters, `uses` clauses, `with` blocks, fields and closure captures, all of which `brio caps` shows (\[CAP-13\]).

### 41.10 Creating capabilities

**\[CAP-28\]** A capability is *minted* only in one of these ways:

1. by a function of its authority module marked `@grant`, which the runtime calls at program start, and only if the program's manifest grants the capability by its full path in `custom` in `[permissions]`. A `@grant` function's parameters SHALL be capabilities, which the runtime supplies from the program's other permissions. A call to a `@grant` function from any other code is ill-formed. It is not an initializer: the runtime calls it at most once, before `main` starts;
2. by a function of its authority module that receives, as parameters, the capabilities the new one is built on, such as `PatientRecords.open(store: Dir)`;
3. by attenuation of an existing value of the same type (\[CAP-30\]);
4. in test builds only, by a function marked `@test_grant`, which \[CAP-4\] makes available to tests.

Without a manifest grant or a capability to build on, a capability can never come into existence: user-defined capabilities are denied by default.

### 41.11 Narrowing, soundly

**\[CAP-29\]** A capability type that can be narrowed SHALL declare a *policy*: a `@pure`, bounded method `fn allows(self, action: A) -> Bool`, where A is the capability's *action type*, describing every action the value authorizes.

**\[CAP-30\]** A function marked `@subset` SHALL take `self` and return a value of the same capability type. For every `@subset` function, the compiler SHALL generate the obligation that the result authorizes nothing the original did not:

`forall a: A: result.allows(a) implies self.allows(a)`

This obligation SHALL be proven, in every build and every verification mode, before the program is accepted. It is never checked at runtime, assumed or observed. A narrowing function that could widen authority is therefore rejected at compile time.

**\[CAP-31\]** Only `@grant` functions, the functions of \[CAP-28\] item 2, `@subset` functions and `@test_grant` functions MAY return a newly minted capability.

### 41.12 Using and delegating

**\[CAP-32\]** A user-defined capability is used exactly like a built-in one: in parameters, in `uses` clauses (`uses records` with its shorthand, or `uses records: PatientRecords`), and in `with` blocks. Granted `@grant` capabilities are in the capability context of `main` under their shorthand names (\[CAP-17\]).

**\[CAP-33\]** The authority module decides what each operation requires. It SHOULD state its policy as preconditions of its public methods, such as `requires self.allows(.Read(ward))`, so that the policy is checked like any contract (\[CON-29\]).

**\[CAP-34\]** A capability type implements `Clone` only if it declares so with `@derive(Clone)`, which permits explicit delegation. The core type `Revocable[C]`, for any C that does not implement `Clone`, wraps a capability together with a separate `RevokeHandle`. `r.with_cap(|c| …) -> T ! Revoked` gives the closure a borrow of the capability that cannot escape it (\[OWN-22\]). After `handle.revoke()`, every later use fails with `Revoked`.

### 41.13 Visibility and simulation

**\[CAP-35\]** The assurance ledger SHALL record every user-defined capability type, its authority module, whether it is granted to the program, and the proof status of its `@subset` obligations. `brio caps` SHALL show user-defined capabilities alongside built-in ones.

**\[CAP-36\]** A user-defined capability built on built-in capabilities is simulated and fault-injected through them (\[CAP-12\]), with no extra code.

*Example:*

```brio
/// Access to patient records, optionally limited to one ward.
public capability PatientRecords as records {
    store: Dir
    ward:  WardId?                       // .None means every ward
}

public enum RecordAction { Read(WardId), Write(WardId) }

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
    let record = try records.read(id)
    return Summary.from(record)
}
```

*Note:* Without the precondition of `only_ward`, narrowing a capability already limited to ward 3 to ward 5 would authorize ward 5, which the original did not. The obligation of \[CAP-30\] fails, and the compiler shows that counterexample.

### 41.14 Using several capabilities

*Syntax:*

```text
CapabilitySet ::= Visibility? "capability" "set" IDENT "=" UsesItem ("," UsesItem)*
```

**\[CAP-37\]** *Resolution by name.* A function MAY list any number of capabilities in its `uses` clause, including several of the same type under different names (`uses src: Dir, dst: Dir`). Implicit passing matches by name and type only (\[CAP-17\]), and names in a capability context are unique, so the capability passed never depends on the order of items, on other capabilities in scope, or on type inference. Adding a capability to a context never changes which capability an existing call receives.

**\[CAP-38\]** *Order.* Several `with` clauses before one block apply from top to bottom, each seeing those above it, exactly as if each clause opened its own nested block. In `with a = x` followed by `with a.only_ward(w)`, the second clause therefore limits the capability the first one set.

**\[CAP-39\]** *Capability sets.* A `capability set` names a group of `uses` items: `capability set Reporting = net, records, clock`. Writing the set's name in a `uses` clause is exactly equivalent to writing its items. Sets MAY include other sets; a set that includes itself, directly or indirectly, is ill-formed. `set` is a contextual keyword after `capability`.

**\[CAP-40\]** *Joining.* A capability type MAY declare functions marked `@join` that take `self` and another value of the same type and return a value of that type. For each, the compiler SHALL prove, as for \[CAP-30\]:

`forall a: A: result.allows(a) implies (self.allows(a) or other.allows(a))`

A join therefore never authorizes more than its inputs together; it only lets one value carry authority the holder already has. `net.join(other)` is provided for every core capability type that has a policy.

**\[CAP-41\]** *Combined authority.* An operation that needs several capabilities takes them all, as parameters or through `uses`. A capability type MAY contain other capabilities as fields (for example a `ReportExporter` holding `records` and `net`); its policy then describes the combined actions, and its subset and join functions carry the obligations of \[CAP-30\] and \[CAP-40\].

**\[CAP-42\]** *Tasks and closures.* A capability passed to a spawned task SHALL implement `Send` (\[CONC-7\]). A closure that calls a function with a `uses` clause captures the named capabilities from the context where the closure is created, exactly as if they were written in its body, and `brio caps` shows the capture. The core capability types implement `Send` and `Sync`; a user-defined capability does so when all its fields do.

**\[CAP-43\]** *Revoking together.* `RevocationGroup` creates several `Revocable` capabilities that share one `RevokeHandle`; revoking it revokes all of them at once (\[CAP-34\]).

**\[CAP-44\]** *Least privilege.* The toolchain SHALL warn when a function lists a capability in `uses`, or receives one as a parameter, that it never uses or passes on. It SHOULD also suggest the narrowest form that covers the function's actual use, such as `read_only()` when a `Dir` is only read, and `brio caps --suggest` SHALL list these suggestions for the whole program.

*Example:*

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

## 42 Trusting the toolchain

No compiler can be proven free of every defect. This clause makes trust in the toolchain rest on independent evidence: each stage is checked by something other than itself, what remains trusted is small and named, and every failure is loud.

### 42.1 Checking the compiler's output

**\[TRUST-1\]** *Cross-checking code generators.* `brio test --cross-check` SHALL run the test suite, including property tests, compiled by the each development code generator (\[IMPL-2\]), compiled by the release code generator, and under `brio interp` (\[MODEL-6\]), and SHALL report any difference in observable behavior as a suspected miscompilation, with a reduced test case where possible. `brio ship` SHALL run it for every release.

**\[TRUST-2\]** *Semantic preservation for critical code.* For `critical` modules, an implementation SHALL provide at least one of:

1. a compilation path whose preservation of Brio Core semantics is proven in the mechanization of \[MODEL-4\];
2. translation validation: for each build, an independent checker confirms that each optimization step preserved the meaning of the code, and a step it cannot confirm is either replaced by an unoptimized equivalent or reported as an error.

The method used for each module is recorded in the assurance ledger. Back-to-back testing (\[MODEL-7\]) is required in addition.

**\[TRUST-3\]** *Independent proof checking.* The rules of \[CON-26\] for critical obligations apply equally to the termination obligations of Clause 39 and the capability obligations of \[CAP-30\] and \[CAP-40\].

### 42.2 Failing loudly

**\[TRUST-4\]** *Internal errors.* If the compiler detects an internal inconsistency or crashes, it SHALL NOT leave any artifact that could be mistaken for a successful build, SHALL report a diagnostic with a stable code, and SHALL write a self-contained reproduction bundle that the user can inspect before sharing.

**\[TRUST-5\]** *Self-checks.* The compiler SHALL validate its own intermediate representations between phases in every build. A validation failure is an internal error under \[TRUST-4\]; it is never ignored.

**\[TRUST-6\]** *Deterministic compilation.* Parallel and incremental compilation SHALL produce artifacts identical to a sequential clean build. `brio build --verify-incremental` SHALL rebuild from scratch and compare, and `brio ship` SHALL run it for every release.

### 42.3 Upgrades and portability

**\[TRUST-7\]** *Toolchain upgrades.* `brio upgrade-check <version>` SHALL build the package with the current and the proposed toolchain, run the tests of both builds with \[TRUST-1\], compare their assurance ledgers, and report every difference in behavior, diagnostics, proof results and ledger entries before the upgrade is adopted.

**\[TRUST-8\]** *Portability warnings.* The toolchain SHALL warn wherever the meaning of a program depends on implementation-defined or unspecified behavior listed in Annex B, such as the size of `USize` or the bits of a NaN, so that code that behaves differently between targets is never silent. `--strict` makes these warnings errors.

### 42.4 Trusting the compiler binary

**\[TRUST-9\]** *Reproducible toolchain.* Every official toolchain release SHALL be reproducible bit for bit from its source and a documented bootstrap, and SHALL be signed with build provenance (\[SEC-14\]). `brioup` SHALL verify signatures and provenance before installing.

**\[TRUST-10\]** *Independent bootstrap.* Every toolchain release SHALL be traceable to an independent root of trust: each compiler used to build the toolchain SHALL itself be reproducible from a full-source bootstrap seed through a chain that does not depend on a binary of that compiler. For the reference implementation, whose compiler is written in Rust (\[IMPL-6\]), this is the Rust toolchain, which can be rebuilt from such a seed through an independent Rust compiler. If a Brio compiler is written in Brio, each release SHALL additionally be verified by diverse double-compiling as in \[IMPL-8\]. Together these detect a compiler that inserts hidden code into itself, which reproducible builds and signatures alone cannot.

**\[TRUST-11\]** *Toolchain vulnerabilities.* The toolchain is itself a package with advisories (\[SEC-17\]). `brio audit` SHALL include the toolchain and its components, such as LLVM, Cranelift, TPDE, the linkers and the proof solvers, and SHALL report known miscompilation and soundness defects from the known-problems list (\[CONF-3\]) that affect the program.

### 42.5 The trusted base

**\[TRUST-12\]** The assurance ledger SHALL list the *trusted base* of every build: the components whose defects could make an accepted program misbehave without detection by any check in this clause. At minimum it names the parts of the compiler not covered by \[TRUST-2\] or the proofs of \[MODEL-4\], the proof checker, the linker, the operating system and the hardware. Reducing the trusted base is a stated goal of every release.

**\[TRUST-13\]** Translation validation that is bounded, for example by unrolling loops a fixed number of times, SHALL be recorded as bounded in the assurance ledger. For `critical` modules, a validation that exceeds its resources or meets an unsupported construct is a failure, not a pass.

**\[TRUST-14\]** For the compilation path of \[TRUST-2\], the trusted base SHALL name the instruction-set and calling-convention specifications, the assembler, the linker and the loader it relies on. Code in `critical` modules SHALL be released only through that path; the development code generators are never used for `critical` releases.

**\[TRUST-15\]** Runtimes that execute WebAssembly components are part of the trusted base: they are pinned in `Brio.lock`, checked by `brio audit`, and untrusted components additionally run under process confinement (\[CAP-10\]).

**\[TRUST-16\]** Diverse double-compiling (\[IMPL-8\]) SHALL use a compiler independent of the one being verified. Its result is recorded as the absence of a detected divergence, not as evidence that the toolchain is correct.

## 43 Package manager and registries

### 43.1 Resolution

**\[PM-1\]** *Complete and explained.* Dependency resolution SHALL find a set of versions satisfying every requirement whenever one exists, using a complete algorithm such as PubGrub. When none exists, the diagnostic SHALL explain the conflict as a chain of reasons in the program's own package names and versions, following Clause 23, and SHALL suggest the changes that would resolve it.

**\[PM-2\]** *Deterministic.* Given the same manifests, the same lockfile and the same registry snapshot, resolution SHALL produce the same result on every machine. Among valid solutions, the resolver prefers the lockfile's versions, then the newest versions.

**\[PM-3\]** *Semantic versioning.* A requirement such as `"2.3"` means `>=2.3.0, <3.0.0`. One version per major version of a package is selected across the whole graph; different major versions of the same package MAY coexist, and the toolchain reports each such duplication. Libraries SHALL NOT pin exact versions; a pin in a library is a warning, and an error when publishing.

**\[PM-4\]** *Toolchain requirement.* A manifest MAY state the oldest toolchain a package supports (`toolchain = ">=1.4"` in `[package]`). The resolver SHALL select only versions whose toolchain requirement the current toolchain meets, and SHALL say so when that excludes a newer version.

**\[PM-5\]** *The lockfile governs.* `brio build`, `brio test` and every other command that does not exist to change dependencies SHALL use `Brio.lock` unchanged. With `--locked`, any need to change the lockfile is an error. Only `brio add`, `brio remove` and `brio update` change it.

### 43.2 Sources of packages

**\[PM-6\]** *One registry per dependency.* Every dependency is resolved from exactly one registry: the one named in its declaration, or otherwise the workspace's default registry. A package is never looked up in a second registry after a miss. This prevents dependency confusion, in which a public package impersonates a private one.

**\[PM-7\]** *Namespaces.* Package names MAY be scoped to an organization, as in `hospital/ward-registry`. A scope belongs to one organization within a registry, and only its members publish under it.

**\[PM-8\]** *Path and Git dependencies.* A path dependency MAY name another package in the workspace. A Git dependency SHALL be pinned to a commit hash, which the lockfile records. A package with path or Git dependencies SHALL NOT be published.

**\[PM-9\]** *Features.* A package MAY declare optional *features*, each enabling optional dependencies or `comptime` configuration. Features are additive: the toolchain SHALL verify that the public interface with a feature enabled contains the interface without it, and SHALL reject a feature that removes or changes an item. The features enabled for a package are the union of those requested anywhere in the graph.

### 43.3 Speed

**\[PM-10\]** *No dependency compilation, with independent rebuilds.* When a dependency provides compiled code for the build's target (\[PKG-21\]), the toolchain SHALL link it and use its interface files without compiling the dependency's source. For packages whose source is public, a registry SHALL make a version resolvable only after at least two independent builders have reproduced its compiled code bit for bit from its source (\[PKG-23\]). Packages without public source SHALL carry the publisher's reproducibility attestation and are marked *unverifiable* in `brio info`, in permission changes on update (\[SEC-10\]) and in the assurance ledger.

**\[PM-11\]** *Shared store.* Downloaded artifacts are kept in a store shared by all projects of a user, addressed by content hash, and used through links rather than copies where the file system allows. An artifact is downloaded at most once per machine.

**\[PM-12\]** *Sparse index.* The registry index is served per package, so resolution fetches metadata only for packages in the graph, with standard HTTP caching. Downloads happen in parallel and are verified as they stream (\[SEC-14\]).

### 43.4 Registry security

**\[PM-13\]** *Metadata integrity.* Registry metadata SHALL be protected with The Update Framework (TUF) or an equivalent scheme providing signed metadata, expiry, and protection against rollback, freeze and mix-and-match attacks, in addition to per-package signatures and provenance (\[SEC-14\]).

**\[PM-14\]** *Publishing.* Publishing SHALL be possible with short-lived credentials obtained by a CI system through OpenID Connect (*trusted publishing*), so that no long-lived publishing token is needed. Interactive publishing requires phishing-resistant authentication (\[PM-23\]). `brio publish --dry-run` SHALL perform every check of 17.7 and \[PM-3\] without publishing.

**\[PM-15\]** *Registry policies.* A registry MAY enforce policies at publish and download time, including: permitted licenses, required provenance, maximum advisory severity (\[SEC-7\]), required dependency reviews (\[SEC-15\]) and permitted package permissions (\[CAP-8\]). A consuming toolchain SHALL report the policy that rejected a package.

### 43.5 Batteries-included registries

**\[PM-16\]** *Documented protocol.* The registry protocol is documented and versioned. Any server implementing it is a conforming registry, and no registry, including the public one, is privileged.

**\[PM-17\]** *Static registries.* `brio registry init --static <location>` SHALL create a registry that consists only of files, hostable on any file server, object storage or version-control repository, with no server process. Publishing to it and reading from it SHALL work with the same commands as any other registry.

**\[PM-18\]** *Registry server.* The toolchain SHALL include `brio registry serve`, a registry server that provides at least:

1. storage on the local file system or S3-compatible object storage;
2. sign-in with passkeys, email codes and single sign-on through OpenID Connect (43.7), scoped tokens, organizations and roles;
3. trusted publishing (\[PM-14\]) and the policies of \[PM-15\];
4. a web interface with search, package pages, generated documentation (\[TOOL-7\]) and advisories;
5. mirroring of other Brio registries, and caching proxies for Rust crates and Python packages, so that all dependencies of a program can come from one internal source;
6. licensed distribution (17.10 to 17.12);
7. an append-only audit log of publications, yanks, ownership changes and downloads;
8. backup, restore and replication, and operation without network access to the outside world.

**\[PM-19\]** *Scaling.* The same commands and manifests SHALL work unchanged whether a team uses the public registry, a static registry or a registry server, so a team can move between them by changing only registry configuration.

### 43.6 Everyday commands

**\[PM-20\]** The toolchain SHALL provide at least:

| Command | Function |
| --- | --- |
| `brio new`, `brio init` | Create a package or workspace, including `.gitignore` and a default manifest |
| `brio add`, `brio remove` | Add or remove a dependency, choosing the newest compatible version, and update the lockfile |
| `brio update [package]` | Move to the newest versions allowed by the manifest, and show the changes, including new permissions and advisories |
| `brio outdated` | List dependencies with newer versions, marking breaking ones |
| `brio tree`, `brio why <package>` | Show the dependency graph, and why a package is included |
| `brio search`, `brio info` | Find packages and show their metadata, permissions and advisories |
| `brio login`, `brio owner`, `brio yank` | Authenticate, manage owners, and yank versions |
| `brio registry init`, `serve`, `mirror`, `backup` | Create and operate registries (43.5) |

### 43.7 Signing in

**\[PM-21\]** *Methods.* A registry server SHALL support signing in with passkeys (WebAuthn, requiring user verification) and with email codes, and MAY additionally support single sign-on through OpenID Connect. Passwords are not supported.

**\[PM-22\]** *Email codes.* An email code SHALL be random, at least eight characters from an unambiguous alphabet, single-use, valid for at most ten minutes, and bound to the sign-in attempt that requested it. Attempts SHALL be rate-limited per account and per address. The response to a code request SHALL be the same whether or not an account exists, so that accounts cannot be discovered through it.

**\[PM-23\]** *Strength.* Passkeys and single sign-on with phishing-resistant authentication are *strong*. Email codes are *basic*, because anyone who can read the mailbox, or who tricks the user into entering the code on another site, can use them. Publishing, yanking, changing owners, managing organizations, creating tokens with publish rights, and adding or removing sign-in methods SHALL require strong authentication. A registry MAY accept basic authentication for these actions only if its operator enables it explicitly, and every such action SHALL be recorded as such in the audit log.

**\[PM-24\]** *Command line.* `brio login` SHALL use the OAuth 2.0 authorization code flow with PKCE and a redirect to a loopback address on the user's machine, opening the registry's sign-in page in the browser. The OAuth 2.0 Device Authorization Grant (RFC 8628), which attackers use to obtain tokens from users who sign in on the genuine page, MAY be used only on machines without a browser, SHALL be disabled by default for organization accounts, SHALL NOT grant publishing scopes, and SHALL show the requesting address, time and scopes on the verification page. Tokens SHALL be short-lived, limited to the scopes requested (for example `read` or `publish:hospital/*`), bound to a key held by the command-line tool so that they cannot be used from another machine, renewable without a new sign-in until they expire, and stored in the operating system's key store, never in a file in the workspace. `brio logout` revokes them. Continuous integration uses trusted publishing (\[PM-14\]) and never signs in.

**\[PM-25\]** *Account recovery.* A session established with basic authentication (\[PM-23\]) SHALL NOT add or remove sign-in methods, change the account's email address, create tokens with publishing scopes, or join a publishing role. Adding a passkey requires an existing passkey, the attestation of an administrator of the user's organization, or the registry's documented support process; the latter two start a waiting period configured by the registry (at least 72 hours by default), during which every existing sign-in method is notified and can cancel the change, and publishing rights are suspended until it ends. Accounts with publishing rights SHALL register at least two passkeys. Users are offered one-time recovery codes at sign-up. Every change to sign-in methods is recorded in the audit log and notified by email.

**\[PM-26\]** *Self-hosted registries.* `brio registry serve` SHALL provide these methods out of the box. Its configuration names the domain that passkeys are bound to and the mail server used for email codes; without a mail server, only passkeys and single sign-on are offered. Static registries (\[PM-17\]) use the access control of the storage that hosts them.

### 43.8 License management

**\[PM-27\]** *Model.* A registry server that distributes licensed packages SHALL manage: *products*, each a set of licensed packages; *plans*, each stating the layers, targets, version range and build rights a license grants; *licensees*, each an organization with members; *licensee keys*; and *licenses*, each binding a licensee to a product and plan for a period. Every operation on these SHALL be available both in the web portal and through the API (43.9), with identical effect.

**\[PM-28\]** *Key custodian.* Content keys (\[PKG-33\]) are protected by a *key custodian*: a small, isolated service, together with a *custodian backend* that holds the publisher's key-encryption keys. Content keys SHALL be stored only in wrapped form, under a key-encryption key that never leaves the backend. Storage, mirrors, content delivery and the web tier SHALL hold no content key and no access to the backend, so \[PKG-36\] continues to hold. A registry operated by someone other than the publisher SHALL use a custodian and backend controlled by the publisher.

**\[PM-29\]** *Licensee keys.* A licensee's administrators enroll their own public keys, one per developer, build machine or CI system, each with a label, through the portal, the API or `brio license key enroll`. Private keys never leave the licensee (\[PKG-37\]). A plan MAY limit the number of enrolled keys (*seats*). Removing a key stops wrapping for it from then on.

**\[PM-30\]** *Automatic wrapping.* When a license is granted, a key is enrolled, or a version is published, the custodian SHALL wrap the content keys of every covered layer, version and target for every active key of every licensee entitled to it, without re-encrypting any content (\[PKG-40\]). Wrapping SHALL complete before the change is reported as done.

**\[PM-31\]** *Lifecycle.* A license can be issued as a trial or in full, and extended, upgraded to another plan, suspended, resumed, expired and terminated. Suspension, expiry and termination stop new downloads, new wrapping and token renewal; they never affect programs already built (\[PKG-39\]).

**\[PM-32\]** *Build rights after expiry.* A plan states what a licensee may build after its license ends: nothing, or *perpetual build rights* for every version released while the license was active. For products used in regulated devices, the registry SHOULD default to perpetual build rights, so that a licensee can always rebuild the versions it shipped, for example to fix a defect. The rights are stated in the license token (\[PKG-38\]).

**\[PM-33\]** *License tokens.* While a license is active, the toolchain SHALL obtain and renew tokens automatically after `brio login`. For air-gapped licensees, the portal and API SHALL produce a signed *offline bundle* containing the ciphertext, wrapped keys and token for the licensed versions.

### 43.9 The license API

**\[PM-34\]** *API.* The registry API SHALL be documented as an OpenAPI description, versioned, and backward compatible within a major version; a major version SHALL remain available for at least twelve months after its successor is released. The portal and the `brio` command line SHALL use only this API.

**\[PM-35\]** *Safe automation.* Write operations SHALL accept an idempotency key, so that a retried request never takes effect twice. Lists SHALL be paginated. Every response SHALL identify the request in the audit log.

**\[PM-36\]** *Scoped access.* API tokens carry explicit scopes, such as `licenses:read`, `licenses:write`, `licensees:write` and `keys:enroll`, and MAY be limited to one product or organization. Creating a token with write scopes requires strong authentication (\[PM-23\]).

**\[PM-37\]** *Webhooks.* The registry SHALL send signed webhooks for at least: license issued, changed, expiring (at configurable intervals before expiry), expired and terminated; key enrolled and removed; seat limit reached; version published; and download by a licensee. Delivery is at least once, with a unique event identifier for deduplication and automatic retries.

### 43.10 Roles and oversight

**\[PM-38\]** *Separation of duties.* The registry SHALL provide distinct roles for publishing packages, managing licenses, managing licensees' own keys and members, and administering the registry. An organization MAY require a second person's approval for issuing, upgrading and terminating licenses. Every license and key operation requires strong authentication (\[PM-23\]).

**\[PM-39\]** *Audit and reports.* License, key and wrapping operations SHALL be recorded in the audit log, which SHALL be tamper-evident, each entry including the hash of the one before. The portal and API SHALL report active licenses, seats in use, upcoming expiries, and downloads per licensee and version, exportable in open formats.

**\[PM-40\]** *Personal data.* The registry SHALL store only the personal data needed to manage accounts and licenses, and SHALL support exporting and deleting it, subject to the retention the audit log requires.

### 43.11 The custodian backend

**\[PM-41\]** *Backend requirements.* A custodian backend SHALL provide: key-encryption keys that cannot be exported; wrapping bound to a context, which the custodian sets to the package, version, layer and target (\[PKG-33\]); generation of random data keys returned in wrapped form; versioned key rotation, with re-wrapping of stored keys to the newest version without revealing them, and retirement of old versions; per-operation access policies; and an audit log of every operation. The reference backend is the Transit engine of OpenBao, with derived keys and non-exportable key-encryption keys. Cloud key-management services and hardware security modules that meet these requirements MAY be used instead.

**\[PM-42\]** *Minimal exposure.* HPKE wrapping for licensees added after publication (\[PKG-34\]) is performed by the custodian. Where the deployment provides a hardware security module or an attested enclave, the wrapping SHALL take place inside it, so that no content key exists in ordinary memory. Otherwise a content key SHALL exist in plaintext only in the custodian's memory while it wraps, held as a `Secret` value (\[LSEC-1\]) that is zeroed immediately afterwards and never written to storage, logs or crash reports, and the ledger of every affected licensee records *custodian exposure*. The custodian SHALL run as a separate process, reachable only by the registry's internal work queue. Its backend credential permits only unwrapping content keys under its publisher's key-encryption keys; the backend's derivation context SHALL be computed by the custodian from the signed manifest, never supplied by a caller; and production backend policy SHALL forbid returning data keys in plaintext and SHALL forbid convergent encryption.

**\[PM-43\]** *Publishing.* When a licensed version is published, the publishing job, running under the publisher's trusted-publishing identity (\[PM-14\]), SHALL generate each layer's content key, encrypt the layer, wrap the key with HPKE for every licensee entitled at that moment, and store the key only after encrypting it with the backend under the publisher's key-encryption key. The job's backend credential SHALL permit encryption only, never decryption, and the job zeroes every content key before it ends.

**\[PM-44\]** *Correlated audit.* Every backend operation SHALL be recorded both in the backend's audit log and in the registry's audit log (\[PM-39\]), with a shared request identifier, so that every unwrap of a content key can be traced to the license or publication that required it.

### 43.12 Local development of registries

**\[PM-45\]** *Development stack.* `brio registry dev` SHALL start, on one machine, a complete registry environment for development and testing: a registry server with license management, a custodian with a development instance of the reference backend, and S3-compatible object storage and key management provided by a local cloud emulator such as Floci. The same stack SHALL be available to tests through container-based fixtures, so that license flows can be tested end to end in continuous integration.

**\[PM-46\]** *Development artifacts are marked.* Keys, tokens and packages created by a development stack SHALL be marked as such. Production registries SHALL refuse them, and the toolchain SHALL refuse to build a release (\[TOOL-14\]) that depends on them.

### 43.13 Several registries and licenses at once

**\[PM-47\]** *Package identity.* A package is identified by its registry's canonical identifier together with its name. Packages with the same name in different registries are different packages. Mirrors (\[WS-3\]) share the identifier of the registry they mirror. `Brio.lock` records the registry of every package.

**\[PM-48\]** *Renaming.* A dependency MAY be given a local name: `wr = { package = "ward-registry", registry = "internal" }`. The local name is the module path used in source (\[SCR-5\]). Two dependencies with the same package name from different registries SHALL be given different local names.

**\[PM-49\]** *One resolution.* Dependencies from every registry and ecosystem (Brio packages, Rust crates and Python packages) are resolved in one run (\[PM-1\]) and recorded in one lockfile. The toolchain SHALL fetch index data from different registries concurrently, and the shared store (\[PM-11\]) holds identical content only once, whatever registry it came from.

**\[PM-50\]** *Transitive licensed dependencies.* When a licensed package depends on another licensed package, the consumer needs a license for each licensed layer it must decrypt, unless the intermediate package's publisher holds *redistribution rights* for the dependency. Redistribution rights are a *redistribution grant*: a token signed by the dependency's publisher naming the redistributing publisher, the licensees or licensee policy covered, the package and version range, and an expiry. The dependency's custodian SHALL wrap content keys only for licensees covered by a valid grant, SHALL record each grant and wrap in the audit log, and neither publisher learns the other's content keys.

**\[PM-51\]** *All licenses checked at once.* Before downloading, `brio add`, `brio update` and every build SHALL determine every license the resolved graph requires and report, in one diagnostic, each license that is missing, expired or insufficient for a layer or target, with its publisher and how to obtain it. `brio licenses` SHALL list every license the program needs, which are held, which are covered by redistribution, and when each expires.

**\[PM-52\]** *Credentials per registry.* Sign-in tokens (\[PM-24\]) and license tokens (\[PKG-38\]) are stored and renewed per registry and per license, in the operating system's key store. A token for one registry is never sent to another.

**\[PM-53\]** *Offline first.* When every package the lockfile names is in the shared store with its wrapped keys, and every needed license token is valid, a build SHALL succeed without contacting any registry. When a registry is unreachable and a needed artifact is missing, the diagnostic SHALL name that registry, and no other registry is tried (\[PM-6\]).

**\[PM-54\]** *Vendoring licensed packages.* `brio vendor` (\[WS-4\]) SHALL copy licensed layers only in encrypted form, together with the wrapped keys for the current licensee's keys and its license tokens, so that a vendored directory never contains decrypted licensed content.

**\[PM-55\]** *Early warning.* The toolchain SHALL warn when a license the program needs expires within a period configured in the manifest (30 days by default), and `brio ship` SHALL report every license that will expire within the release's declared support period.

**\[PM-56\]** *Other ecosystems.* Packages proxied from other ecosystems keep the ecosystem in their identity, such as `crates.io:serde`, and are never merged with Brio packages. Their build scripts and install hooks SHALL run only in the isolated process of \[TOOL-36\], or be refused.

**\[PM-57\]** The TUF timestamp metadata of a registry (\[PM-13\]) SHALL expire within hours, and clients SHALL refuse metadata that has expired. Mirrors SHALL pass signed metadata through unchanged.

**\[PM-58\]** *Separate roots for development.* The development stack of \[PM-45\] SHALL use its own root keys, which production registries and toolchains never trust, so that no key, token or package created by it can be accepted in production, whatever its markings.

## 44 Performance

This clause makes Brio's safety information available to its optimizer, soundly. Every fact the optimizer receives is subject to \[CON-40\]: it is a proven consequence of the language rules, a proven obligation, or the result of a check at a trust boundary, never an assumption.

### 44.1 The performance promise

**\[PERF-1\]** *Floor.* On the Brio benchmark suite, release builds SHALL be within 3% of Rust built with overflow checks by geometric mean, and no benchmark SHALL be more than 10% slower. Results against Rust without overflow checks and against C, with and without `restrict`, are reported alongside.

**\[PERF-2\]** *Claims.* A claim that Brio is faster than another language SHALL name the workload and the benchmark that supports it, and SHALL be reproduced by the method of 44.6. Implementations and documentation SHALL NOT claim that Brio is faster than C or Rust in general, or that its safety costs nothing.

### 44.2 Using proven facts

**\[PERF-3\]** *Proven check elimination.* A check for overflow, bounds, range, division or conversion SHALL be removed only when it is proven never to fail (\[CON-40\]), in every build. A remaining check branches to a shared, cold block that traps, so that it neither unwinds nor prevents vectorization of the surrounding code more than necessary. `brio build --report checks` SHALL list the checks removed and retained in each function, with the reason a retained check could not be proven, as a guide to where a loop invariant or range type would help.

**\[PERF-4\]** *Non-aliasing.* The implementation SHALL tell its optimizer that `inout`, `take` and `out` parameters, and default-mode parameters of freeze types (\[OWN-40\]), do not alias other accessible memory, as far as \[OWN-42\] makes such aliasing undefined, and SHALL NOT do so for types containing interior mutability. Pointers entering from foreign code carry no non-aliasing fact unless a check of \[CON-41\] establishes it. Every release of the toolchain SHALL be tested with translation validation of its optimizer (\[TRUST-2\]) and with differential testing (\[TRUST-1\]) on a corpus that exercises these facts, and SHALL provide a documented option that withholds them, for diagnosis.

**\[PERF-5\]** *Effects.* A function proven bounded (Clause 39) MAY be marked to its optimizer as always returning and making progress; an `unbounded` function SHALL NOT be. A function proven `@pure` MAY be marked as not accessing memory, not synchronizing and not freeing memory. Effect annotations that are declared but not proven SHALL NOT reach the optimizer.

**\[PERF-6\]** *No unwinding.* Brio code never unwinds (Clause 11). The implementation SHALL compile every Brio function without unwinding cleanup code, and traps are calls to functions that do not return.

### 44.3 Guaranteed optimizations

**\[PERF-7\]** The following SHALL either hold in the final code of a release build or cause a compile error explaining why they do not:

1. `return tail f(x)`: the call is a tail call and uses no additional stack. `tail` is a contextual keyword immediately after `return`;
2. `@inline(always)`: the function is inlined at every call site;
3. `@vectorize` on a loop: the loop is vectorized; development builds only warn;
4. `@no_alloc`: the function and everything it calls do not allocate (\[PROF-9\]);
5. `@no_copy_larger_than(n)`: no value larger than n bytes is copied implicitly;
6. `@max_stack(n)` and `@max_frame(n)`: the function's stack use, including callees, and the size of an asynchronous frame do not exceed n bytes.

**\[PERF-8\]** *Copy elision.* Results and `out` parameters of types that are not passed in registers SHALL be constructed directly in the caller's destination, and a value moved into a container by `take` SHALL be constructed in place. In `critical` modules, an implicit copy of a value larger than a limit set in `[verify]` (256 bytes by default) is an error.

### 44.4 Layout and code generation

**\[PERF-9\]** *Layout.* Unless a type has a declared representation (`@repr`), the implementation MAY reorder its fields and SHALL use invalid values of range types and enums to represent the absence of a value in `T?`. Structure-of-arrays storage, hot and cold field splitting and bit-packing of range types change how fields are addressed and are applied only when requested, with `@layout(soa)`, `@hot_cold` and `@packed`. Small-size storage of `List` and `String` is a documented property of those library types, not a hidden transformation.

**\[PERF-10\]** *Calling convention.* Within a package, and across packages optimized together, the implementation MAY pass several results in registers and split small aggregates into scalars (\[OWN-9\]); the Brio ABI applies at package boundaries (\[OWN-38\]). The choice SHALL be a deterministic function of the program.

**\[PERF-11\]** *Processor variants and floating point.* Outside `critical` programs, a function MAY be compiled in variants for different processor features, selected once at startup from a documented table; the selection is recorded in telemetry and crash reports. `critical` programs are built for one fixed processor. Floating-point code follows IEEE 754 exactly. Reassociation is permitted only through explicit operations such as `sum_unordered` and `dot_unordered`, or inside a lexically scoped `@fp(reassoc)`, and never for a whole program.

**\[PERF-12\]** *Memory.* The implementation SHALL report the allocations that escape analysis placed on the stack or in an arena. In `critical` modules, every allocation after initialization is an error (\[PROF-9\]).

**\[PERF-13\]** *Code size.* The toolchain SHALL report the code size contributed by each generic item's instances, and MAY compile instances marked `@shared_code` once, with dictionary passing, to limit code growth. Calls through dynamic values whose concrete type is known SHALL be made directly.

### 44.5 Profiles and the critical profile

**\[PERF-14\]** *Profile-guided optimization.* Outside `critical` programs, profile-guided and post-link layout optimization MAY be used only with profiles committed to the repository and included, by content hash, in the build key (\[PKG-26\]), so that builds remain reproducible. A build SHALL NOT read a profile that is not committed.

**\[PERF-15\]** *Critical builds.* `critical` modules are compiled with a documented, restricted set of optimizations whose correctness is established by \[TRUST-2\] for every function. Profile-guided optimization, post-link optimization and processor variants are not used. Optimization across packages is permitted only with validation of every affected function and an analysis showing that all generated code is traceable to source, recorded for the qualification kit.

### 44.6 Measurement

**\[PERF-16\]** *Method.* The benchmark suite SHALL contain algorithm kernels, application workloads and `critical`-profile control loops, each in Brio, Rust and C. Timing results SHALL be measured with randomized code and data layout over at least 30 layouts, reported as geometric means with 95% confidence intervals, and a gate of \[PERF-1\] passes only when the upper bound of the interval meets it.

**\[PERF-17\]** *Continuous integration.* Every change to the toolchain SHALL be checked for regressions in instruction counts, code size, stack and frame sizes, and the number of checks removed and non-aliasing facts emitted; these measurements are deterministic. Timing gates run on dedicated hardware at least daily. Every release additionally runs the soundness checks of \[PERF-4\], the interpreter on the test suite, and fuzzing of the checks of \[CON-41\].

## 45 Accelerators

### 45.1 Kernels

**\[ACC-1\]** A function marked `@kernel` is a *kernel*: code that runs as many parallel *work items* on an accelerator such as a GPU. Kernels are compiled for the accelerator targets the implementation documents (for example SPIR-V for Vulkan compute and OpenCL, PTX for NVIDIA devices and AMDGCN for AMD devices), and SHALL also compile for the host processor with identical semantics.

**\[ACC-2\]** *Restrictions.* A kernel and every function it calls SHALL be bounded (Clause 39), SHALL NOT allocate, recurse, use dynamic values or function values, hold capabilities, or perform input and output, and SHALL NOT contain `unsafe` operations other than those of `core.arch.gpu`. Every check of \[TRAP-1\] in a kernel is proven or performed; a failed check ends the whole launch, which then completes with the error `KernelTrap` on the host, and every buffer the launch could write is discarded.

**\[ACC-3\]** *Work-item context.* A kernel's first parameter has the core type `WorkItem`, which gives the work item's global, group and local indices and the launch dimensions. Its values are range types bounded by the launch dimensions, so indices derived from them are proven in range when the buffers have the launch's size.

### 45.2 Memory and data races

**\[ACC-4\]** *Device buffers.* Data is moved to a device as a `DeviceBuffer[T]`, created from host data with `take`, and moved back with `take`. T SHALL implement the core trait `DeviceSafe`: it is freeze (\[OWN-40\]), has a declared representation, and contains no references, pointers or capabilities. While a launch is in progress, the buffers it uses are owned by the launch's handle, so host code cannot access them.

**\[ACC-5\]** *No data races by construction.* A kernel receives each buffer either as `DeviceSlice[T]`, which is read-only, or as `Partitioned[T, k]`, which gives each work item exclusive access to its own k consecutive elements, determined by its index. Writes to the same element by different work items are possible only through atomic types. Memory shared by a work group has the core type `Workgroup[T, n]`, and access to it follows the rules of \[ACC-6\].

**\[ACC-6\]** *Barriers.* A barrier SHALL be reached by every work item of a group or by none: the compiler SHALL prove, by uniformity analysis, that every barrier is in control flow that does not depend on the work item's index. Between two barriers, each element of `Workgroup` memory is written by at most one work item and not read by another, which the compiler SHALL check as for \[ACC-5\].

### 45.3 Launching, results and assurance

**\[ACC-7\]** Launching a kernel requires the capability `Accelerator` (Clause 41), obtained like other device capabilities. `launch(kernel, dims, buffers) -> Launch ! AcceleratorError` returns a handle that is awaited for the results.

**\[ACC-8\]** *Floating point.* Each accelerator target documents where its arithmetic departs from IEEE 754. A kernel marked `@fp(strict)` SHALL NOT be compiled for a target that departs. Reductions across work items combine values in a fixed order unless an operation named `_unordered` is used (\[PERF-11\]).

**\[ACC-9\]** *Testing.* `brio test` SHALL run every kernel test on the host processor and on each available accelerator and compare the results, reporting any difference beyond the documented floating-point departures of \[ACC-8\] as a suspected miscompilation (\[TRUST-1\]).

**\[ACC-10\]** Accelerator drivers, including the compilers they contain, are part of the trusted base (\[TRUST-12\]) and are recorded with their versions in the assurance ledger. Kernels SHALL NOT be used in `critical` modules.

## 46 Software updates

**\[UPD-1\]** *Update packages.* `brio ship --update` SHALL produce an update package for a deployed product: the new release, a signed manifest stating the product, the versions it may be installed over, its hash and its security version, and the release's ledger and bill of materials. Update metadata SHALL be protected as in \[PM-13\], with an offline root key and a threshold of signatures for release; frameworks designed for embedded fleets, such as Uptane, satisfy this rule.

**\[UPD-2\]** *Verification before use.* A device SHALL verify the signatures, the hash and the version constraints of an update before installing it, and SHALL refuse any update whose security version is lower than the one it runs, using a monotonic counter protected by hardware where the device has one. A delta update SHALL be verified against the hash of the complete resulting image.

**\[UPD-3\]** *Atomic installation and rollback.* An update SHALL be installed so that either the old or the new release runs, never a mixture, for example with two system slots. After starting a new release, the device runs a health check stated in the manifest; if the check fails, or the release traps before confirming, the device SHALL return to the previous release automatically.

**\[UPD-4\]** *Persistent data.* Data stored in `@schema` formats (\[SER-6\]) that a new release migrates SHALL remain readable by the release it replaced until the new release is confirmed, for example by migrating a copy, so that rollback never meets data it cannot read.

**\[UPD-5\]** *Critical devices.* In programs containing `critical` modules, an update SHALL be installed only when the program's safety state machine (Clause 34) is in a state declared safe for updating, SHALL NOT interrupt a function in progress, and SHALL complete or roll back within a stated time. The procedure is part of the program's timing analysis and of its assurance ledger.

**\[UPD-6\]** Every installed, refused and rolled-back update SHALL be recorded on the device and reported through telemetry (\[OBS-6\]), and `brio ship` SHALL keep, for each product, the history of update packages and the ledgers of their releases.

## 47 Implementation of the toolchain

The run-time speed of Brio programs is determined by the release code generator and by the facts of Clause 44 that it receives, not by the language in which the toolchain is written. The speed of the toolchain itself is determined chiefly by its architecture and its development code generators. This clause fixes both.

### 47.1 Code generators and linkers

**\[IMPL-1\]** *Release.* Release builds SHALL be compiled with LLVM, at a version pinned by the toolchain release and recorded in the assurance ledger. `critical` modules use the restricted, validated set of optimizations of \[PERF-15\].

**\[IMPL-2\]** *Development.* The implementation SHALL provide a development code generator for every supported target, and MAY provide several. The reference implementation provides Cranelift for every target, and TPDE for ELF-based x86-64 and AArch64 targets, which consumes the same LLVM intermediate representation as release builds. For each target, the default development code generator is chosen by measuring compile time and run time on the suite of Annex G, and is documented; `backend` in `[build.dev]` overrides it.

**\[IMPL-3\]** Every development code generator is subject to \[TRUST-1\]. A code generator that fails the comparison for a release SHALL NOT be offered for that release, and the failure is added to the known-problems list (\[CONF-3\]). No development code generator is used for release builds or for `critical` code (\[TRUST-14\]).

**\[IMPL-4\]** *Linkers.* Release builds that are qualified use a linker within the qualification scope of the toolchain, documented per target. Development builds MAY use faster or incremental linkers, which are recorded in the build metadata.

### 47.2 How the toolchain is written

**\[IMPL-5\]** *Memory safety.* Components of the toolchain developed by the Brio project SHALL be written in a memory-safe language, with `unsafe` code confined to audited modules listed in the assurance ledger. Third-party components that are not, such as LLVM, TPDE and the SMT solvers, SHALL be named and version-pinned, and either run in a confined process or be placed outside the trusted base (\[CON-26\]).

**\[IMPL-6\]** *One compiler, rooted outside Brio.* The reference implementation's compiler is written in Rust (\[IMPL-5\]) and built with a Rust toolchain that satisfies \[TRUST-10\]. An implementation whose compiler is not written in Brio needs no second Brio compiler, because the root of trust lies in the independent bootstrap of the language it is written in.

**\[IMPL-7\]** *Determinism.* Given identical inputs (source, options, target and toolchain identity), each compiler SHALL produce identical outputs. Uncontrolled non-determinism, including iteration order of hash tables, thread scheduling, timestamps, absolute paths and wall-clock limits on solvers, is a defect.

**\[IMPL-8\]** *If the compiler is written in Brio.* An implementation MAY write its compiler in Brio, using only a declared *bootstrap subset* of the language. It SHALL then also provide a *bootstrap compiler*: a separate program, not written in Brio, that translates the compiler's source into code a toolchain for another language can build. The bootstrap compiler need not check programs, optimize, report diagnostics or serve editors, because it only has to translate this one program, whose validity the full compiler has already established. Each release SHALL be verified by diverse double-compiling: the bootstrap compiler builds the compiler from its source, that result builds the compiler again, and the output SHALL be identical to the released binary. The bootstrap compiler's own build chain SHALL satisfy \[TRUST-10\]. A portable seed, such as a WebAssembly build of the compiler with a small interpreter written in C, MAY be provided if it can be regenerated byte for byte from source.

### 47.3 Assembly

**\[IMPL-9\]** Code written in the assembly language of a target, whether in separate files or through the inline-assembly facility of `core.arch`, SHALL be limited to: startup code, context switching of tasks, interrupt and trap entry, stack probing, fallbacks for atomic operations, and wrappers of individual processor instructions. Each unit is listed in the assurance ledger with its target, reviewer and test evidence. Cryptographic code SHOULD use formally verified assembly where hand-optimized code is needed.

### 47.4 Architecture and budgets

**\[IMPL-10\]** All compilation work SHALL be computed by memoized queries keyed by content hashes, with early cutoff, in one engine serving every interface of \[TOOL-18\]. Queries are keyed by stable identifiers, not by positions in text.

**\[IMPL-11\]** *Budgets.* On the reference hardware, the implementation SHALL measure in continuous integration, on a corpus of one million lines, the time to lex and parse (target 200 ns per line or less), to check (target 2 µs per line or less) and to complete (\[COMP-12\]). A change that makes any of them more than 5% slower SHALL NOT be merged without an explicit decision recorded with the change.

**\[IMPL-12\]** *Proof caching.* Proof results SHALL be cached by the hash of the verification condition, the prelude, the solver's identity and version, its options and its resource limit. Resource limits are counted in solver steps, not in time, so that results are deterministic. Proofs are checked in parallel, and a result is recorded as proven only as \[CON-26\] requires.

### 47.5 Runtime library

**\[IMPL-13\]** The libraries `core`, `alloc` and `std` (Clause 30) SHALL be written in Brio, with `unsafe` code confined to audited modules listed in the assurance ledger and assembly limited by \[IMPL-9\]. The subset of `core` that is certified for `critical` use is documented separately.

**\[IMPL-14\]** *Allocators.* The global allocator is replaceable. The default for hosted targets is implementation-defined and chosen by measurement on the suite of Annex G. For embedded and `critical` targets, the library SHALL provide fixed-size pools and an allocator whose worst-case time for every operation is bounded and documented.

## 48 Experimentation

This clause governs experiments from declaration to decision. An *experiment* is the item in source (\[OBS-13\]). A *run* is one execution of an experiment with a fixed plan: its allocation, targeting, metrics, statistical method and schedule. A *campaign* groups runs, across one or more experiments and programs, that serve one goal and have one owner.

### 48.1 Declaring an experiment

**\[EXP-1\]** In the long form of `experiment`, `control`, `metrics`, `guardrails` and `hypothesis` are contextual keywords. `metrics` names the `metric` items (\[OBS-4\]) that judge the experiment, the first being its *primary metric*; `guardrails` names metrics that must not get worse; `hypothesis` is a string stating the expected effect. For example:

```brio
experiment dashboard_layout: Layout by ClinicianId {
    control    = .Classic
    metrics    = [task_seconds, correction_rate]
    guardrails = [alarm_ack_seconds, crash_rate]
    hypothesis = "The compact layout shortens charting without slowing alarm response"
}
```

**\[EXP-2\]** Every variant is handled wherever `variant(for:)` is matched, because matches on enums are exhaustive. `brio test --experiments all` SHALL run the tests under every variant of every experiment the tested code reaches, as \[OBS-15\] does for flags.

**\[EXP-3\]** *Critical code.* The variant of an experiment SHALL NOT influence code in `critical` modules, which the compiler checks through the call graph and data flow. In programs containing `critical` modules, the manifest SHALL name, for every experiment, the approval under which it may run, such as that of an ethics committee, and the approval is recorded in the assurance ledger and in every run.

### 48.2 Assignment

**\[EXP-4\]** *Deterministic assignment.* A unit's variant SHALL be computed from a keyed hash of the run's salt, the experiment's name and the unit's identifier, mapped to one of 10,000 buckets, and the buckets to variants by the run's allocation. The same unit therefore receives the same variant in every process and on every machine for the whole run, without any stored state.

**\[EXP-5\]** *Stable reallocation.* Changing the allocation of a run SHALL move only the buckets needed to reach the new proportions, so units keep their variant wherever possible. Every allocation change starts a new *segment* of the run, recorded with its time, and analysis treats segments separately.

**\[EXP-6\]** *Eligibility.* A run MAY target units by rules on attributes of the evaluation context (\[OBS-14\]). An ineligible unit receives the control variant and records no exposure. Values of `Secret` and `Sensitive` types SHALL NOT be used as targeting attributes.

**\[EXP-7\]** *Layers and holdouts.* Runs in the same *layer* are mutually exclusive: each unit is in at most one of them. Runs in different layers are assigned independently, with independent salts. A campaign MAY reserve a *holdout*, a share of units that receive the control variant of all its runs, to measure their combined effect.

**\[EXP-8\]** *Exposure.* An exposure is recorded the first time a unit is assigned in a run segment, with the run, segment, variant and plan version, and never again for that unit and segment. Unit identifiers in exposure records SHALL be pseudonymized with a keyed hash, and exposure data has a retention period stated in the plan.

### 48.3 Plans and statistics

**\[EXP-9\]** *Pre-registered plans.* A run cannot start without a plan stating: the hypothesis, primary metric, secondary metrics and guardrails; the minimum effect worth detecting; the significance level and power; the resulting sample size and expected duration, computed by the toolchain; the statistical method; and the stopping rules. After the run starts, its plan can be changed only by starting a new run.

**\[EXP-10\]** *Valid at any time.* Results shown while a run is in progress SHALL use sequential methods whose error rates hold no matter how often and when they are looked at, such as always-valid confidence sequences. A run using a fixed-horizon method SHALL NOT report significance before its planned sample size is reached.

**\[EXP-11\]** *Many comparisons.* When a run has more than two variants or more than one metric, the analysis SHALL control the error rate across all comparisons, and SHALL mark every result that is not the pre-registered primary comparison as secondary.

**\[EXP-12\]** *Sample ratio mismatch.* The analysis SHALL test continuously whether the observed numbers of units per variant match the allocation. A significant mismatch means the assignment or logging is broken: the run is flagged, its results are withheld, and, if the plan says so, it is paused.

**\[EXP-13\]** *Guardrails and automatic stopping.* When a guardrail metric is shown, by the sequential method of \[EXP-10\], to be worse than the plan's threshold in any variant, that variant SHALL be stopped and its units returned to the control variant, and the owners notified. Every run also has a *kill switch* that returns all units to the control variant immediately.

**\[EXP-14\]** *Ramping.* A plan MAY define stages of allocation, such as 1%, 5% and 50%, each with a minimum duration and the guardrail checks that must pass before the next stage begins automatically.

**\[EXP-15\]** *Reproducible analysis.* Every result SHALL be reproducible from the recorded exposures and metric events, the plan and the version of the analysis engine, all of which are kept with the run. Exposures and metric events are exported over OTLP (\[OBS-6\]), so any tool can re-analyse them.

### 48.4 Control and overview

**\[EXP-16\]** *Experiment service.* The toolchain SHALL include an experiment service, served by the registry server of \[PM-18\] or on its own, that acts as the flag provider of \[OBS-14\] for experiments and provides, in a web portal and through an API with identical capabilities: every campaign, experiment and run with its status, plan, allocation, segments, exposures, sample-ratio check, results with their uncertainty, guardrails and decisions; and the operations of creating, planning, starting, ramping, pausing, stopping and concluding runs, and grouping them into campaigns.

**\[EXP-17\]** *API.* The experiment API follows \[PM-34\] to \[PM-37\]: an OpenAPI description, versioning, idempotency keys, scoped tokens, and signed webhooks for at least run started, stage advanced, guardrail stopped, sample-ratio mismatch detected, paused, stopped and concluded.

**\[EXP-18\]** *Roles and audit.* Designing plans, starting and ramping runs, and concluding them are separate roles; an organization MAY require a second person's approval to start a run or change its allocation. Every change is made with strong authentication (\[PM-23\]) and recorded in the tamper-evident audit log of \[PM-39\], with who, when and why.

**\[EXP-19\]** *Offline and local.* A run's plan and allocation can be exported as a signed file for the file-based provider of \[OBS-14\], so experiments also run where the service cannot be reached. `brio experiments` SHALL list every experiment in the program with its runs, and the development stack of \[PM-45\] includes the experiment service.

### 48.5 Concluding

**\[EXP-20\]** *Decisions.* Concluding a run records a decision: which variant ships, or none, with its reason and the results that support it. The toolchain SHALL then report the experiment's code for removal and offer a fix that replaces every `variant(for:)` with the chosen variant; `brio lint` reports experiments that have had no active run for longer than a configured period.

**\[EXP-21\]** *Records.* Plans, segments, exposures summaries, results, decisions and approvals of every run are kept for the period set by the organization, and `brio ship` includes in the release's ledger the experiments present in the release and the state of their runs.

### 48.6 Custom units, audiences and outcomes

Developers MAY define what an experiment assigns, whom it targets, how it balances its groups and what it measures, from any data the program can reach. The randomization itself is never custom (\[EXP-30\]).

**\[EXP-22\]** *Units.* The unit type of `by T` SHALL implement the core trait `AssignmentUnit`, which requires a stable identifier that can be encoded (Clause 33), equality and hashing. It SHALL NOT be or contain a `Secret`, `Sensitive` or `Untrusted` value. Users, devices, hospitals, tenants and regions are all valid units.

**\[EXP-23\]** *Audiences.* A function marked `@audience(refresh: d)`, where d is a constant `Duration`, describes the units that flags and experiments are evaluated for. It either takes one unit and returns `A ! E`, or takes a list of units and returns `Map[U, A] ! E`; the second, *batch* form SHOULD be used when attributes come from a database or a service, so that many units are computed by one query. A is a record of *audience attributes* whose fields implement `Encode` and are not `Secret` or `Sensitive`. The function MAY use any capabilities (\[CAP-16\]), for example a database. Its results form the audience's *snapshot*: a versioned, hashed and signed map from units to attributes, recomputed at least every d.

**\[EXP-32\]** *Complete audiences.* `@audience(refresh: d, units: f)` names a function f that returns every unit of the audience; each snapshot then covers all of them, which a frozen cohort (\[EXP-26\]) requires. Without `units`, a unit first seen by `variant(for:)` or `value()` is queued and included from the next refresh.

**\[EXP-33\]** *One runner.* Each refresh of a snapshot SHALL be computed exactly once, by one designated runner: either one instance of the program, designated through the experiment service, or a worker of the experiment service that runs the audience function with capabilities granted to it for that purpose in the manifest. Every other instance receives the published snapshot and verifies its signature and hash before use.

**\[EXP-24\]** *Evaluation never waits.* `variant(for:)` and `value()` read only the most recent snapshot of the audience. They never call an audience function, never block and never fail (\[OBS-14\]). A unit not in the snapshot receives the control variant or the flag's default, records no exposure, is queued under \[EXP-32\], and is counted in the overview as a unit without attributes.

**\[EXP-25\]** *Audience, eligibility and balance.* In the long form of `experiment`, `audience`, `eligible` and `balance_by` are contextual keywords. `audience` names an audience function; `eligible` is a `@pure`, bounded function from A to `Bool`; and `balance_by` is a `@pure`, bounded function from A to a type with finitely many values, such as a payload-free enum, `Bool` or an integer range type. Assignment (\[EXP-4\]) is then performed separately within each value of `balance_by`, so every group has the allocated proportions in every stratum. A run's plan MAY add eligibility rules written as typed expressions over the fields of A; the experiment service SHALL check them against the type of A, as published in the program's interface files, and SHALL refuse to start a run whose rules do not type-check.

**\[EXP-26\]** *Attributes fixed before treatment.* A unit's eligibility and stratum SHALL be computed at its first assignment in a segment, or at the start of the run when the plan asks for a *frozen cohort*, and SHALL NOT be recomputed during that segment. The snapshot version used is recorded with the exposure. The toolchain SHALL warn when an audience function reads data that an outcome or metric of the same experiment also reads, because an attribute influenced by the treatment biases the result.

**\[EXP-27\]** *Outcomes.* A function marked `@experiment_outcome(window: d)` takes a unit and two instants, `from` and `until`, and returns a number `! E`. It MAY use any capabilities. The experiment service calls it for every exposed unit, with `from` at the unit's first exposure and `until` d later or at the end of the run, and records the value with the snapshot of the data it read where the source supports it. Only data within the window counts. `metrics` and `guardrails` MAY name experiment outcomes as well as `metric` items.

**\[EXP-28\]** *Grouped units.* When outcomes or metric events are measured for individuals, such as patients, while assignment is by a group, such as a hospital, the sample size and the analysis SHALL account for the similarity of individuals within a group, and results SHALL be reported per assignment unit.

**\[EXP-29\]** *Flags.* In the long form of `flag`, `default` and `audience` are contextual keywords, and `audience` names an audience function whose attributes the provider's rules for the flag MAY use. \[EXP-23\], \[EXP-24\], \[EXP-26\], \[EXP-32\] and \[EXP-33\] apply to flags as well. Flags read by `critical` code remain fixed before the operational phase (\[OBS-16\]).

**\[EXP-30\]** *Fixed randomization.* Programs SHALL NOT supply their own assignment algorithm. Units, attributes, eligibility, balance and outcomes are open to developers; the mapping of units to variants is always that of \[EXP-4\], so every experiment keeps its statistical validity.

**\[EXP-31\]** *Testing.* `brio test` SHALL let tests supply attribute snapshots and outcome data as fixtures. `brio sim` SHALL simulate complete runs with synthetic units and a planted effect, and report whether the plan's analysis detects it and how often it reports an effect where none was planted, so that a custom experiment's design is checked before it is run.

## 49 Language instrumentation: Brio Lens

*Brio Lens* makes everything the toolchain does to a program observable: what each phase of the compiler produced and why, what the program did when it ran, and how much work each build or edit caused. It is part of the toolchain from its first phase onward, and it is how the language itself is developed, tested and explained.

### 49.1 Stage records

**\[LENS-1\]** *Stages.* With `brio lens build` or `--lens`, every phase SHALL emit a *stage record* for each module it processes: tokens; the syntax tree, including recovered errors; name resolution; types and inferred types; ownership, borrow and origin facts; contracts, proof obligations and their results; the call graph and boundedness of every function; capability flows; compile-time evaluation and embedded-language expansions; Brio Core; the intermediate code before and after each optimization pass of every code generator; machine code; and the link map.

**\[LENS-2\]** *Format.* Stage records use one documented, versioned format with a published schema, readable without the toolchain, and exportable as JSON. They are written to the build directory (\[PKG-25\]) and keyed by content hash like every other cache entry (\[PKG-26\]).

**\[LENS-3\]** *Provenance.* Every element of every stage record SHALL carry a stable identifier and links to the source text it comes from and to the elements of the previous stage it derives from. Elements the compiler creates, through inlining, monomorphization, compile-time evaluation or embedded languages, record their full chain of origin. Any element can therefore be traced back to source, and any source text forward to everything it became.

**\[LENS-4\]** *Reasons.* Every decision SHALL be recorded with its reason: the candidates considered in name lookup; the constraints that determined an inferred type; for a rejected borrow, both accesses and the rule; for a removed check, the proof that justified it, and for a retained check, the goal that could not be proven; for a bounded function, its measure, and for an unbounded one, the cycle or wait; for each capability use, where the capability came from; and for each optimization pass, what it changed.

**\[LENS-5\]** *No effect.* Recording SHALL NOT change any artifact: a build with Lens and the same build without it produce identical outputs (\[IMPL-7\]), which the conformance suite tests. When Lens is off, it costs nothing.

**\[LENS-6\]** *The compiler's own work.* Lens SHALL record, from the query engine (\[IMPL-10\]), which queries ran for a build or an edit, which were answered from the cache, where early cutoff stopped recomputation, and the time each took, so that the cause of every rebuild is visible.

### 49.2 Execution records

**\[LENS-7\]** `brio interp --lens` and `brio sim --lens` SHALL record execution: every step with its values; memory, including the tags and permissions of Annex E; tasks and the schedule that interleaved them; capability uses; and traps with their cause. A recording replays deterministically, and can be stepped backward as well as forward.

**\[LENS-8\]** When the comparison of \[TRUST-1\] or \[ACC-9\] finds a difference, Lens SHALL show the first observable event at which the executions differ, aligned across the interpreter and the code generators involved, together with the intermediate code of the functions concerned.

### 49.3 Viewing and testing

**\[LENS-9\]** *Viewer.* `brio lens` SHALL open a viewer that runs locally and works offline, providing at least: the pipeline view, in which selecting any element highlights its source and every element derived from it in every stage; graph views of the call graph, capability flows, ownership over time, tasks and the query engine; the comparison of two builds, of two toolchain versions, or of two code generators; and search across all records. Large workspaces are loaded on demand.

**\[LENS-10\]** *Programmatic access.* Stage and execution records SHALL be queryable through a documented interface, and through the assistant server within the limits of \[TOOL-35\], so that tests, tools and assistants can inspect them.

**\[LENS-11\]** *Stage tests.* `brio lens test` SHALL compare the stage records of test programs with reviewed reference records and report differences per stage. The conformance suite (\[CONF-1\]) SHALL test each rule of this specification, where its effect is visible in a stage record, through such a test, indexed by the rule's identifier.

**\[LENS-12\]** *Confidentiality.* Lens SHALL NOT record or show decrypted licensed layers to anyone not entitled to them (\[PKG-35\]), SHALL mask values of `Secret` and `Sensitive` types in execution records, and writes nothing outside the build directory unless asked.

### 49.4 From the first phase

**\[LENS-13\]** Every phase of the reference implementation SHALL emit its stage records in the same change that introduces the phase. A phase without stage records, provenance and reasons is incomplete and SHALL NOT be released. Lens is therefore available from the first compiler milestone onward, and grows with the toolchain.

## Annex A (normative): Collected grammar

This annex collects the grammar of Brio. Token classes (IDENT, INTEGER, FLOAT, NUMBER, CHAR, STRING, NEWLINE) are defined in Clause 3. NUMBER is INTEGER or FLOAT.

**\[GRAM-1\]** The grammar is to be read together with these disambiguation rules: the newline rules \[LEX-28\] to \[LEX-32\]; the unit-annotation rule \[LEX-26\]; the tuple-index rule \[LEX-27\]; the generic-argument rule \[EXPR-21\]; and the binding rule \[PAT-1\]. Because records are constructed with parentheses, a `{` after an `if`, `while`, `for` or `match` header always begins a block.

### A.1 Files and items

```text
SourceFile     ::= ModuleHeader? Item*
ModuleHeader   ::= Attribute* "module" ModulePath
InterfaceFile  ::= "interface" ModulePath Item*
ModulePath     ::= IDENT ("." IDENT)*
Path           ::= IDENT ("." IDENT)*
TypePath       ::= Path

Item           ::= Attribute* Visibility? ItemKind
Visibility     ::= "public" ("(" "package" ")")?
ItemKind       ::= UseDecl | ConstDecl | StaticDecl | TypeAlias | RangeTypeDecl
                 | DistinctDecl | RecordDecl | EnumDecl | UnionDecl | TraitDecl
                 | ImplDecl | FnDecl | UnitDecl | TestDecl | BridgeDecl
                 | ExternBlock | ComptimeBlock
UseDecl        ::= "use" "rust"? Path ("." "{" UseItem ("," UseItem)* ","? "}" | "as" IDENT)?
UseItem        ::= IDENT ("as" IDENT)?
ConstDecl      ::= "const" IDENT ":" Type "=" Expr
StaticDecl     ::= "static" "var"? IDENT ":" Type "=" Expr
TypeAlias      ::= "type" IDENT GenericParams? "=" Type
RangeTypeDecl  ::= "type" IDENT "=" Type "in" RangeBounds
DistinctDecl   ::= "distinct" "type" IDENT GenericParams? "=" Type ("in" RangeBounds)?
RangeBounds    ::= Expr ("..<" | "..=") Expr
RecordDecl     ::= "frozen"? "record" IDENT GenericParams? WhereClause?
                   "{" (RecordMember (SEP RecordMember)* SEP?)? "}"
RecordMember   ::= FieldDecl | "invariant" Expr
FieldDecl      ::= Attribute* Visibility? IDENT ":" Type ("=" Expr)?
EnumDecl       ::= "frozen"? "enum" IDENT GenericParams? WhereClause?
                   "{" Variant (SEP Variant)* SEP? "}"
Variant        ::= Attribute* IDENT ("(" VariantField ("," VariantField)* ","? ")")? ("=" Expr)?
VariantField   ::= Type | IDENT ":" Type
UnionDecl      ::= "union" IDENT GenericParams? "{" FieldDecl (SEP FieldDecl)* SEP? "}"
TraitDecl      ::= "trait" IDENT GenericParams? (":" Bounds)? WhereClause? "{" TraitMember* "}"
TraitMember    ::= Attribute* (FnDecl | "type" IDENT (":" Bounds)? ("=" Type)?
                              | "const" IDENT ":" Type ("=" Expr)?)
ImplDecl       ::= "unsafe"? "impl" GenericParams? (TypePath GenericArgs? "for")? Type
                   WhereClause? "{" ImplMember* "}"
ImplMember     ::= Attribute* Visibility? (FnDecl | "type" IDENT "=" Type | ConstDecl)
FnDecl         ::= FnQualifier* "fn" FnName GenericParams? "(" ParamList? ")"
                   ("->" Type OriginSpec?)? WhereClause? UsesClause? Contract* FnBody?
FnQualifier    ::= "unsafe" | "unbounded" | "async" | "comptime" | "extern" STRING
FnName         ::= IDENT | TypePath "." IDENT
ParamList      ::= Param ("," Param)* ","?
Param          ::= Attribute* (Mode? "self" | IDENT? IDENT ":" Mode? Type ("=" Expr)?)
Mode           ::= "inout" | "take" | "out"
OriginSpec     ::= "from" IDENT ("|" IDENT)*
FnBody         ::= Block | "=" Expr
UnitDecl       ::= "unit" UnitDef ("," UnitDef)*
UnitDef        ::= IDENT ("=" (NUMBER "*")? UnitExpr ("/" NUMBER)?)?
TestDecl       ::= "test" STRING ("forall" "(" ParamList ")")? Block
BridgeDecl     ::= "rust" "bridge" IDENT "{" BridgeMember* "}"
BridgeMember   ::= "instantiate" Path GenericArgs | Attribute* "fn" Path Contract*
ExternBlock    ::= "extern" STRING "{" ExternFn* "}"
ExternFn       ::= Attribute* "unsafe"? "fn" IDENT "(" ParamList? ")" ("->" Type)?
ComptimeBlock  ::= "comptime" Block
Attribute      ::= "@" Path ("(" (AttrArg ("," AttrArg)* ","?)? ")")?
AttrArg        ::= (IDENT ":")? Expr | Type
SEP            ::= "," | NEWLINE
```

### A.2 Generics and contracts

```text
GenericParams  ::= "[" GenericParam ("," GenericParam)* ","? "]"
GenericParam   ::= IDENT (":" Bounds)? ("=" Type)?
                 | "const" IDENT ":" Type ("=" Expr)?
                 | "unit" IDENT
                 | "origin" IDENT
GenericArgs    ::= "[" GenericArg ("," GenericArg)* ","? "]"
GenericArg     ::= Type | Expr | UnitExpr
Bounds         ::= Bound ("+" Bound)*
Bound          ::= TypePath GenericArgs? | "unbounded"? TypePath "(" (ParamType ("," ParamType)*)? ")" ("->" Type)?
WhereClause    ::= "where" Type ":" Bounds ("," Type ":" Bounds)*
Contract       ::= "requires" Expr
                 | "ensures" ("on" "fail")? Expr
                 | "decreases" Expr ("," Expr)*
```

### A.3 Types

```text
Type           ::= OptType ("!" OptType)?
OptType        ::= PrefixType "?"?
PrefixType     ::= "ref" "mut"? PrefixType OriginSpec?
                 | "any" Bounds                      (* a TypePath naming a trait is a static use, [TRAIT-10] *)
                 | PrimaryType
PrimaryType    ::= TypePath GenericArgs? UnitAnnot?
                 | "(" ")" | "(" Type ")" | "(" Type ("," Type)+ ","? ")"
                 | "[" Type "]" | "[" Type ";" Expr "]"
                 | "unbounded"? ("extern" STRING)? "fn" "(" (ParamType ("," ParamType)*)? ")" ("->" Type)?
                 | "Self" | "type"
ParamType      ::= Mode? Type
UnitAnnot      ::= "<" UnitExpr ">"
UnitExpr       ::= UnitFactor (("*" | "/") UnitFactor)*
UnitFactor     ::= IDENT ("^" "-"? INTEGER)? | "(" UnitExpr ")" | "1"
```

### A.4 Statements

```text
Block          ::= "{" (Stmt (StmtSep Stmt)*)? StmtSep? "}"
StmtSep        ::= ";" | NEWLINE
Stmt           ::= LetStmt | VarStmt | AssignStmt | AssertStmt | FnDecl | ConstDecl | Expr
LetStmt        ::= "let" Pattern (":" Type)? "=" Expr ("else" ("|" Pattern "|")? Block)?
VarStmt        ::= "var" IDENT (":" Type)? ("=" Expr)?
AssignStmt     ::= Expr AssignOp Expr
AssignOp       ::= "=" | "+=" | "-=" | "*=" | "/=" | "%=" | "&=" | "|=" | "^="
                 | "<<=" | ">>=" | "+%=" | "-%=" | "*%="
AssertStmt     ::= "assert" Expr ("," Expr)?
```

### A.5 Expressions

```text
Expr           ::= Quantified | ImpliesExpr
Quantified     ::= ("forall" | "exists") Pattern "in" Expr ":" Expr
ImpliesExpr    ::= OrExpr ("implies" ImpliesExpr)?
OrExpr         ::= AndExpr ("or" AndExpr)*
AndExpr        ::= NotExpr ("and" NotExpr)*
NotExpr        ::= "not" NotExpr | CmpExpr
CmpExpr        ::= CoalesceExpr (CmpOp CoalesceExpr | "is" Pattern)?
CmpOp          ::= "==" | "!=" | "<" | "<=" | ">" | ">=" | "in"
CoalesceExpr   ::= RangeExpr ("??" CoalesceExpr)?
RangeExpr      ::= BitOrExpr (("..<" | "..=") BitOrExpr)?
BitOrExpr      ::= BitXorExpr ("|" BitXorExpr)*
BitXorExpr     ::= BitAndExpr ("^" BitAndExpr)*
BitAndExpr     ::= ShiftExpr ("&" ShiftExpr)*
ShiftExpr      ::= AddExpr (("<<" | ">>") AddExpr)*
AddExpr        ::= MulExpr (("+" | "-" | "+%" | "-%" | "+|" | "-|") MulExpr)*
MulExpr        ::= CastExpr (("*" | "/" | "%" | "*%" | "*|") CastExpr)*
CastExpr       ::= PrefixExpr ("as" Type)*
PrefixExpr     ::= ("-" | "~" | "try" | "await" | "ref" "mut"?) PrefixExpr | PostfixExpr
PostfixExpr    ::= PrimaryExpr PostfixOp*
PostfixOp      ::= "." IDENT | "." INTEGER | ".*" | "?." IDENT
                 | "(" ArgList? ")" | "[" (GenericArg | Expr) ("," (GenericArg | Expr))* "]"
ArgList        ::= Arg ("," Arg)* ","?
Arg            ::= ".." Expr | (IDENT ":")? ("mut" | "move" | "out")? Expr
PrimaryExpr    ::= Literal | Path | "." IDENT | "self" | "result" | "error" | "old" "(" Expr ")"
                 | "(" ")" | "(" Expr ")" | "(" Expr ("," Expr)+ ","? ")"
                 | "[" (Expr ("," Expr)* ","?)? "]" | "[" Expr ";" Expr "]"
                 | IfExpr | MatchExpr | LoopExpr | WhileExpr | ForExpr | Closure
                 | "unsafe" "(" STRING ")" Block | "comptime" Block | Quote
                 | "tasks" Block | "spawn" PostfixExpr
                 | "return" ("tail" PostfixExpr | Expr)? | "fail" Expr | "break" IDENT? Expr? | "continue" IDENT?
IfExpr         ::= "if" Expr Block ("else" (IfExpr | Block))?
MatchExpr      ::= "match" ("mut" | "move")? Expr "{" MatchArm (SEP MatchArm)* SEP? "}"
MatchArm       ::= Pattern ("if" Expr)? "=>" (Expr | Block)
LoopExpr       ::= Label? "loop" LoopSpec* Block
WhileExpr      ::= Label? "while" Expr LoopSpec* Block
ForExpr        ::= Label? "for" Pattern "in" Expr LoopSpec* Block
LoopSpec       ::= "invariant" Expr | "decreases" Expr ("," Expr)* | "bounded" Expr ("else" Block)?
Label          ::= IDENT ":"
Closure        ::= "move"? "|" (ClosureParam ("," ClosureParam)*)? "|" ("->" Type)? (Expr | Block)
ClosureParam   ::= Pattern (":" Mode? Type)?
Quote          ::= "quote" "{" (TOKEN | Splice)* "}"
Splice         ::= "$" IDENT | "$(" Expr ")"
                 | "$(" "for" Pattern "in" Expr ("join" STRING)? ")" "{" (TOKEN | Splice)* "}"
Literal        ::= INTEGER UnitAnnot? | FLOAT UnitAnnot? | CHAR | STRING | "true" | "false"
```

### A.6 Patterns

```text
Pattern        ::= PatternAlt ("|" PatternAlt)*
PatternAlt     ::= PatternBase ("as" IDENT)?
PatternBase    ::= "_" | LiteralPattern | RangePattern | "var"? IDENT
                 | TuplePattern | VariantPattern | RecordPattern | SlicePattern
                 | "(" Pattern ")"
LiteralPattern ::= "-"? INTEGER UnitAnnot? | CHAR | STRING | "true" | "false" | TypePath "." IDENT
RangePattern   ::= RangeBound ("..<" | "..=") RangeBound
RangeBound     ::= "-"? NUMBER UnitAnnot? | CHAR | TypePath "." IDENT
TuplePattern   ::= "(" Pattern ("," Pattern)+ ("," "..")? ","? ")"
VariantPattern ::= ("." IDENT | TypePath "." IDENT) ("(" FieldPatterns ")")?
RecordPattern  ::= TypePath "(" FieldPatterns ")"
FieldPatterns  ::= ".." | FieldPattern ("," FieldPattern)* ("," "..")?
FieldPattern   ::= IDENT ":" Pattern | Pattern
SlicePattern   ::= "[" (SliceElem ("," SliceElem)*)? "]"
SliceElem      ::= Pattern | ".." ("as" IDENT)?
```

*Note:* In `FieldPattern`, a bare identifier is both a valid `Pattern` and the labeled shorthand of \[PAT-2\]; which applies is determined by whether the payload's fields are labeled.

### A.7 Scripts

```text
ScriptFile     ::= Shebang? Frontmatter? (Item | Stmt)*
Shebang        ::= "#!" any characters up to the end of the first line
Frontmatter    ::= "---" NEWLINE TOML-text NEWLINE "---" NEWLINE
UseDecl        ::= ... | "use" "python" Path ("." "{" UseItem ("," UseItem)* ","? "}")?
```

### A.8 Embedded languages, telemetry, flags and benchmarks

```text
ItemKind       ::= ... | CapabilityDecl | LanguageDecl | MetricDecl | FlagDecl | ExperimentDecl | BenchDecl
LanguageDecl   ::= "language" IDENT "=" Expr
MetricDecl     ::= "metric" IDENT ":" Type ("(" ArgList ")")?
FlagDecl       ::= "flag" IDENT ":" Type "=" Expr
                 | "flag" IDENT ":" Type "{" FlagMember (SEP FlagMember)* SEP? "}"
FlagMember     ::= ("default" | "audience") "=" Expr
ExperimentDecl ::= "experiment" IDENT ":" Type "=" Expr "by" Type
                 | "experiment" IDENT ":" Type "by" Type "{" ExperimentMember (SEP ExperimentMember)* SEP? "}"
ExperimentMember ::= ("control" | "metrics" | "guardrails" | "hypothesis" | "audience" | "eligible" | "balance_by") "=" Expr
BenchDecl      ::= "bench" STRING Block
MeasureStmt    ::= "measure" Block                   (* only inside a bench item *)
PrimaryExpr    ::= ... | EmbeddedLit
UseDecl        ::= ... | "use" ("c" | "cpp") STRING "as" IDENT ("{" FnDecl* "}")? | "use" "wit" STRING ("as" IDENT)?
UsesClause     ::= "uses" UsesItem ("," UsesItem)*
UsesItem       ::= IDENT (":" Type)?
CapabilityDecl ::= "capability" IDENT GenericParams? ("as" IDENT)? "{" (RecordMember (SEP RecordMember)* SEP?)? "}"
                 | "capability" "set" IDENT "=" UsesItem ("," UsesItem)*
PrimaryExpr    ::= ... | ("with" WithItem)+ Block
WithItem       ::= IDENT "=" Expr | IDENT "." IDENT "(" ArgList? ")"
EmbeddedLit    ::= "#" IDENT ("{" EmbeddedText "}" | "\"" EmbeddedText "\""
                             | "\"\"\"" EmbeddedText "\"\"\"")
```

## Annex B (normative): Implementation-defined and unspecified behavior

### B.1 Implementation-defined behavior

A conforming implementation SHALL document its choice for each of the following (\[GEN-12\]).

| Behavior | Rule |
| --- | --- |
| Size of `ISize` and `USize` on each target | 4.2 |
| Mechanism used to detect stack exhaustion | \[TRAP-3\] |
| Process exit status after a trap | \[TRAP-5\] |
| Maximum length of a `TrapInfo` message | \[TRAP-9\] |
| Textual form of units when displayed | \[UNIT-10\] |
| Default limits for compile-time evaluation | \[CT-6\] |
| Verification procedure and default prover resource limits | \[CON-24\], \[CON-25\] |
| Mapping of tasks to threads, and scheduling | \[CONC-1\] |
| Executor used for `async` code, and how a Rust runtime is selected | \[CONC-6\], \[RUST-19\] |
| Bounded-priority-inversion protocol used for `Mutex` in critical programs | \[CRIT-9\] |
| List of critical-compatible core library items | \[PROF-3\] |
| ABI strings other than `"C"` | \[UNS-8\] |
| Intrinsics, including inline assembly | \[UNS-4\] |
| Mechanism for linking foreign libraries | \[UNS-11\] |
| Mechanism for reading Rust crate APIs | \[RUST-2\] |
| Which toolchain versions can link each other's compiled code | \[PKG-24\] |

The following are also implementation-defined: the targets whose timing is analyzable (\[RTF-2\]); the targets on which `@constant_time` is supported (\[LSEC-3\]); the selection of the Python interpreter (\[SCR-16\]); the ABI of each target of the Brio C ABI mapping (\[FFI-3\]); the executor used on freestanding targets (\[EMB-10\]); and the Brio ABI of each target (\[OWN-38\]).

### B.2 Unspecified behavior

The following are unspecified. Conforming programs SHALL NOT depend on them.

| Behavior | Rule |
| --- | --- |
| How any argument is passed, and the address of a parameter, except as required by \[OWN-9\] (\[OWN-37\]) | \[OWN-9\] |
| Field order and padding of records and enums without `@repr` | \[TYPE-27\] |
| Whether generic code is monomorphized or uses dictionary passing, and whether instantiations have distinct addresses | \[TRAIT-14\] |
| Whether `@inline(always)` or `@inline(never)` is honored; neither affects meaning | \[ATTR-1\] |
| The order in which concurrently running tasks complete, and therefore which error a `tasks` block reports when several fail | \[CONC-12\] |
| Whether a check that has been proven is also executed at runtime | \[CON-16\] |

### B.3 Undefined behavior

Undefined behavior can arise only in the situations listed in \[GEN-10\] and \[UNS-7\].

## Annex C (informative): Diagnostic code ranges

Diagnostic codes have the form `B` + a category letter + four digits, for example `BO0017`. The category follows the clause of the rule that was violated. Codes are never reused once assigned.

| Letter | Category | Rules |
| --- | --- | --- |
| L | Lexical | LEX |
| S | Syntax | GRAM |
| N | Names, items and attributes | ITEM, FN, ATTR |
| T | Types, generics and traits | TYPE, GNR, TRAIT |
| U | Units, ranges and distinct types | UNIT, RNG, DIST |
| E | Expressions and statements | EXPR, STMT, CTRL, FMT |
| M | Patterns and matching | PAT |
| O | Ownership and borrowing | OWN |
| F | Errors and traps | ERR, TRAP |
| P | Contracts and proof | CON |
| K | Compile-time execution | CT |
| C | Concurrency | CONC |
| X | Unsafe code and foreign functions | UNS |
| Q | Profiles | PROF, CRIT |
| I | Interfaces and packages | PKG |
| R | Rust bridge | RUST |
| D | Toolchain and lints | TOOL |

Rules of Clauses 20 to 49 and Annexes E to H use these letters: SCR and EMB use N; LANG and EXT use K; OBS, RTF and ASSR use Q; DIAG, TOOL and COMP use D; FFI uses R; SEC, LSEC and CAP use X; ALLOC, SER, NUM and TIME use T; WS and PM use I; MODEL, CONF, EVO, LIB, BND, TRUST, PERF, ACC, UPD, IMPL, EXP, LENS, ALIAS, BENCH and LIM use G, a category for general and library rules.

*Example* of a diagnostic in human-readable form:

```text
── VALUE USED AFTER IT WAS MOVED ───────────── src/monitor.brio:42 · BO0017

`buf` was handed over to `archive` on line 41, so it holds no value here:

41 │     archive(move buf)
   │             ━━━━━━━━ ownership moves to archive
42 │     let sum = checksum(buf)
   │                        ━━━ buf is empty at this point

Why it matters: `archive` may already have freed the buffer's memory.

How to fix, pick one:
  1. Compute the checksum before archiving        [recommended · certain]
       let sum = checksum(buf)
       archive(move buf)
  2. If `archive` only reads the buffer, drop `take` from its parameter   [needs review]
       fn archive(buf: Buffer)

brio explain BO0017
```

## Annex D (informative): Changes from the design spec and open issues

### D.1 Decisions that refine or change the design spec

| Topic | This specification | Reason |
| --- | --- | --- |
| Enum variant names | UpperCamel everywhere: `.Urgent`, `.Running` | One naming rule for all variants |
| Record construction | Memberwise initializer `Reading(patient: p, …)`; labeled enum payloads also use parentheses | Removes the ambiguity between a record literal and a block after `if` or `while` (\[GRAM-1\]) |
| Derived units | Need no declaration; scaled units are declared as `unit km = 1000 * m` | Less boilerplate, exact conversions (\[UNIT-3\], \[UNIT-8\]) |
| Trait contracts | Implementations cannot add `requires` | Code written against a trait stays correct (\[TRAIT-3\]) |
| Shadowing | Forbidden within a function | Avoids review errors (\[STMT-4\]) |
| Returning errors | `fail e` | Reads clearly and applies `From` (\[CTRL-8\]) |
| Pattern tests | `e is P` replaces `if let` | One construct for tests and bindings (\[EXPR-16\]) |
| References | `ref` and `ref mut` types and expressions; `.*` dereferences | References needed a written form (\[OWN-20\]) |
| `Int` | Fixed at 64 bits | Behavior independent of target (\[TYPE-3\]) |
| `as` | Always checked; lossy conversions use named functions | Silent truncation is a common defect (\[TYPE-26\]) |
| Error postconditions | `ensures on fail` | Contracts for the error path (\[CON-9\]) |
| Bounded loops | `bounded n else B` exits gracefully; plain `bounded n` must be proven | Termination without traps (\[CON-15\]) |
| Static and dynamic trait use | A trait name alone means static use (`s: Sensor`, `[Sensor]`); `any Sensor` is a sized dynamic value, so `[any Sensor]` works directly (\[TRAIT-10\], \[TRAIT-12\]) | Trait objects are dynamically sized (\[TYPE-21\]) |
| Borrowed returns | `-> T from x` means an implicit borrow | Keeps the design spec's notation valid (\[OWN-34\]) |
| Critical-to-standard calls | Only through `@trusted` boundaries | Makes every unverified dependency visible (\[PROF-6\]) |
| Interface files | Generated into `.brio/`; public API changes guarded by per-item hashes in `Brio.lock` | Source trees contain only .brio files, and nothing generated can go stale (\[PKG-11\], \[PKG-25\]) (\[PKG-11\]) |

### D.2 Issues resolved

| Issue | Resolution |
| --- | --- |
| Proof back end | SMT-LIB 2.6; checked proof certificates for `critical` builds, with safety obligations proven without assumed facts (\[CON-26\]) |
| Reading Rust crate APIs | Each Brio release pins one rustc version and reads that version's rustdoc output; format changes are absorbed in Brio releases, never in users' builds |
| Pinning rustc and LLVM | Each Brio release pins a rustc version whose LLVM major version matches Brio's own; cross-language link-time optimization is used only when they match, and builds succeed without it otherwise |
| Standard library | Structure fixed in Clause 38; the item-by-item API is the companion document *Brio Standard Library* |
| Toolchain qualification | Clause 28: conformance suite, known-problems list, qualification kits and a certified core subset |
| Non-ASCII identifiers | Identifiers stay ASCII in edition 2026. The next edition may allow Unicode identifiers following UAX #31, with confusable-character detection |
| Performance criterion | A floor in every release: within 3% of Rust with overflow checks by geometric mean, no benchmark more than 10% slower. Claims of being faster are made only per named workload. Details, measurement method and gates: Clause 44 |
| Embedded targets | Clause 30 |
| Allocation | Clause 31 |
| Workspaces, registries, vendoring | Clause 32 |
| Serialization and schema evolution | Clause 33 |
| Language evolution | Clause 29 |
| Conformance and formal semantics | Clauses 27 and 28 |
| Timing, fault tolerance, security, SIMD, numerics, time and text, documentation tests | Clauses 34 to 37, \[TOOL-30\] |
| Accessible and translatable diagnostics, build timings | \[DIAG-8\], \[DIAG-9\], \[TOOL-31\] |

### D.3 Status of version 0.1

The following cannot be closed by writing a specification. They are work to be done, and are tracked here until they are complete.

Version 0.1 leaves no design question open: every feature, including accelerators (Clause 45) and software updates (Clause 46), is specified, and no decision is deferred to a later edition. What remains is work that carries out this specification without changing it:

1. Mechanize the theorems of Annex F and record each as proven in the ledger as its proof is checked (\[MODEL-4\]).
2. Implement the toolchain, the conformance suite of \[CONF-1\], and the benchmark suite of Annex G, and publish the first measurements of \[PERF-1\] and the named claims of \[BENCH-2\].
3. Publish the companion documents *Brio Core*, which formalizes Annexes E and F, and *Brio Standard Library*, which documents the library tiers of Clause 30.
4. Obtain the independent assessment of the toolchain for qualification (Clause 28), and submit the qualification record of \[BENCH-4\] to the certification authority of the first product that needs it.
5. Register the "Brio" trademark and publish the trademark policy.

## Annex E (normative): The aliasing model

This annex defines the aliasing model of \[OWN-42\]. It follows the structure of Tree Borrows, adapted to Brio's parameter modes.

### E.1 Tags and trees

**\[ALIAS-1\]** Every allocation has a *tag tree*. The allocation's owner holds the root tag. Creating a reference from a place, whether by passing it to a default-mode, `inout`, `take` or `out` parameter, by `ref` or `ref mut`, or by a borrowing method, creates a new tag as a child of the tag through which the place was reached. Converting a reference to `Ptr` or `MutPtr` does not create a tag: the pointer carries the tag of the reference it came from.

**\[ALIAS-2\]** For every location of the allocation, each tag has one *permission*: `Reserved`, `Active`, `Frozen` or `Disabled`. A tag created by a shared borrow of a freeze location starts `Frozen`; a tag created by an exclusive borrow (`inout`, `out`, `ref mut`) starts `Reserved`; the root starts `Active`. Locations inside an interior-mutability type (\[OWN-35\]) reached through a shared borrow are *cell locations*, to which \[ALIAS-4\] applies instead of \[ALIAS-3\].

### E.2 Accesses

**\[ALIAS-3\]** An access through tag t to location l changes the permissions at l as follows. For t and its ancestors (*local* tags): a read leaves every permission unchanged, except that a read through a `Disabled` tag is undefined behavior; a write changes `Reserved` to `Active`, leaves `Active` unchanged, and is undefined behavior if the permission is `Frozen` or `Disabled`. For every other tag (*foreign* tags): a read changes `Active` to `Frozen` and leaves the other permissions unchanged; a write changes every permission to `Disabled`.

**\[ALIAS-4\]** At cell locations, reads and writes through any tag that reached them through a shared borrow are permitted and change no permission of such tags; other tags are affected as in \[ALIAS-3\].

### E.3 Protectors

**\[ALIAS-5\]** For the duration of a call, the tags created for the call's `inout`, `take` and `out` arguments, and for default-mode arguments of freeze types, are *protected*. It is undefined behavior if, during the call, a protected tag becomes `Disabled`, a foreign write reaches a location a protected `Reserved` tag has read, or the allocation of a protected tag is deallocated. Protection ends when the call returns.

### E.4 Integers and pointers

**\[ALIAS-6\]** Converting a pointer to an integer *exposes* its tag. `Ptr.from_addr` and `MutPtr.from_addr`, which are `unsafe`, produce a pointer whose tag is any exposed tag of the allocation at that address, chosen so as to avoid undefined behavior if such a choice exists. Pointer arithmetic keeps the tag; an access outside the allocation of the tag is undefined behavior.

### E.5 Consequences

**\[ALIAS-7\]** Safe Brio code cannot perform an access that is undefined under this annex (Annex F, theorem F.3). The optimizer MAY rely on the absence of undefined behavior under this annex, and only on that, for the facts of \[PERF-4\]. `brio interp` SHALL track tags and permissions and report every access that is undefined under this annex, with the operations that created and changed the tags involved.

## Annex F (normative): Soundness theorems

This annex states the theorems that the mechanization of \[MODEL-4\] SHALL prove about Brio Core. A theorem is recorded in the assurance ledger as *proven* only when its mechanized proof has been checked by the proof assistant for the toolchain version in use; until then it is recorded as *stated*, and no document SHALL describe it as proven.

| Theorem | Statement |
| --- | --- |
| F.1 Type soundness | Every well-typed Brio Core program without `unsafe` operations either runs forever, finishes with a value, or stops with a trap of \[TRAP-1\]; it never has undefined behavior |
| F.2 Data-race freedom | Every such program has no data races under the memory model of Clause 14 |
| F.3 Aliasing | Every such program performs no access that is undefined under Annex E, and every non-aliasing fact given to the optimizer by \[PERF-4\] holds in every execution |
| F.4 Borrow and origin soundness | The rules of Clause 9, without implied bounds (\[OWN-41\]), ensure that no reference is used after the end of any borrow it depends on |
| F.5 Mechanism independence | The observable behavior of every such program is the same for every passing mechanism permitted by \[OWN-9\] (\[MODEL-8\]) |
| F.6 Verifier soundness | If every obligation of \[CON-21\] for a function is valid, no condition of \[TRAP-1\] occurs in it and every contract holds whenever its preconditions do |
| F.7 Check elimination | Removing a check proven under \[CON-40\] does not change the behavior of any execution (\[PERF-3\]) |
| F.8 Assumptions never reach the optimizer | For every program, the optimized program refines the unoptimized one even when every `assume` fact is false (\[CON-40\]) |
| F.9 Termination | Every function accepted as bounded terminates for every input satisfying its preconditions (\[BND-20\]) |
| F.10 Capability confinement | An execution performs an operation governed by a capability only through a value of that capability obtained as described in Clause 41; subset and join functions never authorize more than their inputs (\[CAP-30\], \[CAP-40\]) |
| F.11 Contract erasure | For a program all of whose obligations are proven, removing all contract checks does not change its behavior |
| F.12 Accelerator data-race freedom | Kernels that satisfy Clause 45 without atomic operations have no data races and produce the same results as their host compilation, up to the floating-point departures of \[ACC-8\] |

Theorems F.1 to F.12 are proven about Brio Core. That the implementation compiles Brio to Brio Core and Brio Core to machine code correctly is established by Clause 42, not by this annex.

## Annex G (normative): The benchmark suite and critical optimization qualification

### G.1 The suite

**\[BENCH-1\]** The benchmark suite of \[PERF-16\] SHALL contain at least the following workloads, each implemented idiomatically in Brio, in Rust (built with and without overflow checks) and in C (with and without `restrict`):

| Group | Workloads |
| --- | --- |
| Language kernels | n-body, spectral-norm, fannkuch-redux, mandelbrot, fasta, k-nucleotide, reverse-complement, binary-trees |
| Algorithms | sorting, hash map insertion and lookup, B-tree, dense matrix multiplication, FFT, CRC-32, SHA-256, ChaCha20-Poly1305, base64, UTF-8 validation |
| Applications | JSON parsing and serialization, HL7 v2 message parsing, user-space packet forwarding, an embedded key-value store, the Brio compiler's own lexer and parser, HTTP request handling |
| Critical control loops | a 1 kHz PID controller, an infusion-rate control loop, a Kalman filter, a telemetry ring buffer; each also measured by static worst-case execution time analysis |

**\[BENCH-2\]** *Named claims.* The workloads that support a claim under \[PERF-2\] are listed with the claim. Version 0.1 defines three claims, each to be published only when the method of \[PERF-16\] supports it: faster than Rust with overflow checks on array kernels whose indices are provably in range; faster than C without `restrict` on kernels with several exclusive buffers; and smaller code than Rust built to unwind on panics.

**\[BENCH-3\]** *Environment.* Timing runs use dedicated machines of documented models, with fixed processor frequency, no other workload, and at least five runs for each of at least 30 randomized layouts. Hardware, operating system, toolchain versions and raw results are published with every report.

### G.2 Optimization across packages in critical builds

**\[BENCH-4\]** Optimization across packages in `critical` modules (\[PERF-15\]) is enabled only with `lto = "qualified"` in `[build.release]` and a qualification record produced by `brio ship`, containing:

1. the translation validation result (\[TRUST-2\]) of every function whose code changed because of optimization across packages;
2. a map from every instruction of the final code to the source statements it implements, derived from the compiler's line tables and checked against the validated translation;
3. a list of all code not directly traceable to a source statement, such as inlined copies and compiler-generated helpers, each with the verification that covers it;
4. the qualification data of the linker and of every tool involved.

Without the record, `brio ship` SHALL fail for a `critical` release that uses optimization across packages. Whether the record is accepted is decided by the certification authority of each product.

## Annex H (normative): Implementation limits

**\[LIM-1\]** *Minimum limits.* A conforming implementation SHALL accept every otherwise valid program that stays within each of the following limits, subject only to the memory available to the compiler (\[LIM-3\]):

| Construct | Minimum limit |
| --- | --- |
| Characters in a source line | 65,535 |
| Lines in a source file | 1,000,000 |
| Characters in an identifier | 1,024 |
| Bytes in a string or embedded-language literal | 16 MiB |
| Nesting of blocks, statements and expressions, together | 256 |
| Stacked `with` clauses before one block (\[CAP-18\]) | 64 |
| Capabilities in a `uses` clause, after expanding capability sets (\[CAP-39\]) | 64 |
| Nesting of capability sets | 16 |
| Parameters of a function, and arguments of a call | 255 |
| Contract clauses of one function | 256 |
| Generic parameters of one item | 64 |
| Depth of generic instantiation, and of trait resolution (\[BND-21\]) | 128 |
| Fields of a record, and fields of an enum variant | 4,096 |
| Variants of an enum, and arms of a `match` | 65,535 |
| Attributes on one item, and arguments of one attribute | 64 |
| Modules in a package | 65,535 |
| Packages in a resolved dependency graph | 10,000 |

**\[LIM-2\]** *Exceeding a limit.* When a program exceeds a limit of the implementation, the implementation SHALL report a diagnostic naming the limit, its value and the location that exceeds it. It SHALL NOT crash, truncate, produce an artifact, or produce code with a different meaning (\[TRUST-4\]).

**\[LIM-3\]** *Deterministic limits.* Every limit SHALL be enforced by counting, so that whether a program is accepted never depends on the host's stack size, number of threads or timing. Exhausting the compiler's memory is reported as a diagnostic under \[TRUST-4\], with the resource that was exhausted.

**\[LIM-4\]** *Documented limits.* Each implementation SHALL document its actual limits, which are at least the minimums of \[LIM-1\], in its toolchain metadata, and `brio explain limits` SHALL list them.

**\[LIM-5\]** *Portability.* The toolchain SHALL warn when a program exceeds a minimum limit of \[LIM-1\] that the implementation itself accepts, because another conforming implementation may reject the program (\[TRUST-8\]). `--strict` makes this an error.

**\[LIM-6\]** Compile-time evaluation, embedded languages and tool extensions have the deterministic limits of \[TOOL-36\], documented as in \[LIM-4\].
