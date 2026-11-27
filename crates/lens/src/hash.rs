// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception

//! Hashing for record keys, stable identifiers and record digests.

/// The BLAKE3 of `bytes`.
pub(crate) fn of(bytes: &[u8]) -> [u8; 32] {
    *blake3::hash(bytes).as_bytes()
}

/// `bytes` in lowercase hexadecimal.
pub(crate) fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;

    let mut text = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        // Writing to a `String` cannot fail.
        let _ = write!(text, "{byte:02x}");
    }
    text
}

/// A BLAKE3 over a sequence of fields.
///
/// Every field is written with its length, so two different sequences never
/// hash the same bytes.
pub(crate) struct FieldHasher(blake3::Hasher);

impl FieldHasher {
    /// A hasher for one purpose. Hashes for different purposes never
    /// collide, because each starts with its own `purpose`.
    pub(crate) fn new(purpose: &str) -> Self {
        let mut hasher = Self(blake3::Hasher::new());
        hasher.text(purpose);
        hasher
    }

    /// Adds a field of bytes.
    pub(crate) fn bytes(&mut self, field: &[u8]) {
        self.count(field.len());
        self.0.update(field);
    }

    /// Adds a field of text.
    pub(crate) fn text(&mut self, field: &str) {
        self.bytes(field.as_bytes());
    }

    /// Adds a number, such as a version.
    pub(crate) fn number(&mut self, number: u64) {
        self.0.update(&number.to_le_bytes());
    }

    /// Adds how many fields of a list follow.
    pub(crate) fn count(&mut self, count: usize) {
        self.number(count as u64);
    }

    /// The hash of every field added.
    pub(crate) fn finish(self) -> [u8; 32] {
        *self.0.finalize().as_bytes()
    }
}
