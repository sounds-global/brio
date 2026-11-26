# Security policy

Brio is designed for systems where failure is not acceptable, so security reports matter even more here than usual. Thank you for reporting responsibly.

## Reporting a vulnerability

**Do not open a public issue.** Report privately through GitHub instead:

1. Go to the [Security tab](https://github.com/sounds-global/brio/security) of this repository.
2. Choose **Report a vulnerability**, and describe the problem, how to reproduce it, and its impact.

Only the maintainers can see the report. We aim to acknowledge it within 3 working days, and to agree with you on a fix and a disclosure date. We credit reporters in the advisory unless you prefer otherwise.

## What counts

Report anything that breaks a guarantee the specification makes, for example:

- unsoundness: a program the toolchain accepts that violates memory, type, ownership or capability safety;
- a proof, contract or check that the toolchain reports as satisfied when it is not;
- escapes from the confinement of compile-time code, embedded languages or tool extensions;
- leaks of licensed layers, `Secret` or `Sensitive` values, or anything else the specification says must stay confidential;
- vulnerabilities in the package manager, registry or update installer.

## Supported versions

Brio is in early development and has no releases yet. Fixes are made on `main`.

## Safety notice

The Brio toolchain is not qualified under any safety standard, and it must not be the sole means of assurance for safety-critical software. Until it is qualified, it marks everything it builds as *unqualified* (specification, `[GEN-15]`).
