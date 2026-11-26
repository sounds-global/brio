# Governance

## Who decides

Brio is led by Sounds Global B.V., which holds the copyright and the "Brio" name. The maintainers review and merge changes, and decide on language proposals. Once there are maintainers from outside the company, the project can move to a neutral foundation.

## How the language changes

- Language changes go through the public proposal process of the specification (Clause 29). Anyone may propose one by opening an issue describing the problem.
- Breaking changes happen only in new editions. Code written for an earlier edition keeps building.
- The specification in `docs/` is the definition of the language. A change to the language is a change to the specification, made in the same pull request as its implementation or before it.
- Normative rules carry stable identifiers. A removed rule's identifier is never reused.

## How decisions are recorded

Decisions that are hard to reverse are recorded as short decision records in `docs/adr/`, either for the whole workspace or for one context (see `GLOSSARY-MAP.md`).

## Trademark

The code and specification licenses do not cover the "Brio" name or logo. Their use will follow a published trademark policy.

## Code of Conduct

Everyone taking part in the project follows the [Code of Conduct](CODE_OF_CONDUCT.md).
