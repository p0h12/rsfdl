# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-02-12

### Added

- Initial release
- SFDL v2 & v3 container parsing with AES-128-CBC decryption
- Async FTP downloads with configurable parallelism (1-3 threads)
- Resume support, retry logic, and hash verification
- Real-time progress display with progress bars
- Desktop GUI (Dioxus) with file selection, password dialog, and settings
- CLI for headless/scripted usage (info, list, download commands)
- Cross-platform support: macOS, Linux, Windows

[Unreleased]: https://github.com/p0h12/rsfdl/compare/v0.1.0...HEAD

[0.1.0]: https://github.com/p0h12/rsfdl/releases/tag/v0.1.0
