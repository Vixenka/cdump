# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Serialize, and deserialize traits - API to process serialization and deserialization.
- Support for shallow data - serialization support for a simple shallow/plain data via copy.
- Support for single level pointers - serialization support for a deep serialization of single pointer to T.
- Support for CString - serialization support for a CStrings ended with `\0` character.
- Support for arrays - serialization support for an arrays of T under pointer, with providen length via another field.
- Support for array of CStrings - serialization support for an arrays of CString, each CString is under pointer which make double pointer.
- Support for dynamic types - serialization support for a dynamic types which size cannot be known by compiler.
- CDebug macro - macro for implement Rust's Debug for C types.

[unreleased]: https://github.com/Vixenka/cdump/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/Vixenka/cdump/releases/tag/v0.1.0
