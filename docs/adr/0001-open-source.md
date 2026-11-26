# Brio is open source

All Brio code is under Apache-2.0 with LLVM exception, and the language specification and documentation are under CC BY 4.0. Every widely adopted systems language is fully open, and the LLVM exception means programs that embed standard-library or runtime code carry no attribution obligations. LLVM, Cranelift and TPDE, which Brio builds on, use the same license.

## Considered Options

- **Plain Apache-2.0.** Rejected: without the LLVM exception, programs compiled with Brio would carry attribution obligations for the runtime code embedded in them.
- **CC BY-ND 4.0 for the spec.** Rejected: a spec no one may adapt is not open, and it would block translations and annotated editions.

## Consequences

- Publishing is irreversible: anything released under these licenses stays usable under them.
- The "Brio" name is not covered by either license. Protecting it needs a trademark and a published trademark policy.
- Outside contributions need the Developer Certificate of Origin or a contributor license agreement before the first one is accepted.
