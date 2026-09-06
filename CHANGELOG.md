# Changelog

All notable changes to NullSector are documented here. This project uses
[semantic versioning](https://semver.org/). While the version stays below
`1.0.0`, minor releases may change behavior.

## [0.3.0] - 2026-09-05

### Changed

- Renamed the crate and the binary from `ns-bootcon-gui` to `nullsector`.
  The name now matches the project. Download links from earlier releases
  do not work with this version.

## [0.2.0] - 2026-09-05

### Added

- CI workflow running `cargo fmt`, `cargo clippy` (warnings denied), and
  build/test on Linux, Windows, and macOS.
- Release workflow that builds binaries for Linux x86_64, Windows MSVC,
  and macOS on tagged pushes. The workflow packages each binary with a
  SHA256 checksum and publishes a GitHub release.
- `docs/USAGE.md` walkthrough of each panel and `docs/ARCHITECTURE.md`
  covering how external commands run and how their output is displayed.

### Changed

- Rewrote the README with accurate features, requirements, and build steps.
- Updated all dependencies to current versions.

### Fixed

- The nmap flag checkboxes (`-sC`, `-sV`, `-v`) now change the command that
  the app runs. Earlier versions drew these checkboxes but ignored them.
- Diagnostic commands and terminal launch no longer fail silently.
- Removed `unwrap` and `expect` calls and denied them with a lint. A
  missing tool now logs a warning instead of crashing the app.
- Corrected stale disclaimer text about crashing on missing tools, and
  removed dead links.

## [0.1.0]

- Initial release: egui desktop GUI wrapping `nmap`, `dig`, `whois`,
  `nslookup`, and `ping`, with local network info, host info, weather from
  `wttr.in`, terminal access, and PEAS script download.
