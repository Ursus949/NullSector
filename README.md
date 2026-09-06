# NullSector

[![CI](https://github.com/Ursus949/NullSector/actions/workflows/ci.yml/badge.svg)](https://github.com/Ursus949/NullSector/actions/workflows/ci.yml)

NullSector is a desktop GUI for common networking and security tools. It
wraps `nmap`, `dig`, `whois`, `nslookup`, `ping`, and a few others behind
buttons. A user does not need to remember the exact command syntax for
each tool. It also shows local host information and downloads the PEAS
privilege-escalation scripts.

The project started as a final project for a Cyber Security class. The goal
was to build something useful while learning Rust and GUI development. It
is a starting point, not a finished security suite. Add the tools and
panels your own workflow needs.

## Features

- **Terminal access.** Opens a terminal window using the user's shell.
- **Local network info.** Shows the local network configuration
  (`ip addr show` on Linux, `ifconfig` on macOS, `ipconfig /all` on
  Windows).
- **Nmap.** Scans a target with configurable flags (`-sC`, `-sV`, `-v`,
  custom ports, and `-oA` output files).
- **DNS and network tools.** `nslookup` (NS, MX, TXT, ANY), `dig`, `whois`,
  and `ping`.
- **Weather.** Current conditions and a 3-day forecast from `wttr.in`.
- **PEAS download and run.** Downloads and runs winPEAS or linPEAS,
  depending on the OS.
- **Host info.** Shows the public IP, local IP, OS distribution, device
  name, hostname, desktop environment, and user account details.

See [`docs/USAGE.md`](docs/USAGE.md) for a full walkthrough of each panel,
and [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) for how the app runs
external commands and shows their output.

## Requirements

NullSector runs the following external tools. Install the ones you plan to
use and put them on your `PATH`.

| Tool | Used by | Linux/macOS | Windows |
| --- | --- | --- | --- |
| `nmap` | NMAP panel | usually preinstalled or in the package manager | `choco install nmap` |
| `dig` | Network Tools panel | `bind-tools` / `dnsutils` package | `choco install bind-toolsonly` |
| `whois` | Network Tools panel | `whois` package | `choco install whois` |
| `nslookup` | Network Tools panel | usually preinstalled | usually preinstalled |
| `ping` | Network Tools panel | preinstalled | preinstalled |
| `curl` | Weather panel, public IP lookup, PEAS download | usually preinstalled | preinstalled since Windows 10 1803 |

On Windows, a missing tool makes its button log a warning instead of
crashing the app. Install the tool and try again.

On Linux, `nmap` runs the `-sC` and `-sV` scans as root, through `sudo`.
The app runs `sudo` in a real terminal, so it can prompt for your password.

The app also needs a way to open a terminal window. On Linux it checks the
`$TERMINAL` environment variable, then a list of common terminal
emulators (Alacritty, kitty, WezTerm, Konsole, GNOME Terminal, and others).
Install at least one of these if your desktop environment does not set
`$TERMINAL`.

## Build and run

NullSector needs a current stable Rust toolchain (install one from
[rustup.rs](https://rustup.rs)).

```sh
cargo run
```

For a release build:

```sh
cargo build --release
```

The binary is at `target/release/ns-bootcon-gui`.

## Development

Run these checks before you send a change:

```sh
cargo fmt
cargo clippy --all-targets -- -D warnings
cargo test
```

The crate denies `unwrap` and `expect` outside tests
(`#![deny(clippy::unwrap_used, clippy::expect_used)]` in `lib.rs` and
`main.rs`). Handle errors instead of panicking, for example by returning a
`Result` or logging a warning with `tracing::warn!`.

Generate the API documentation with:

```sh
cargo doc --no-deps --open
```

## Project layout

```
src/
  main.rs   entry point, sets up logging and starts the eframe window
  lib.rs    crate root, re-exports BootCon
  app.rs    the BootCon app: UI layout and all external-command logic
docs/       usage guide and architecture notes
build.rs    embeds the Windows executable icon
```

## License

This project has no license file yet. Contact the repository owner before
you reuse this code.
