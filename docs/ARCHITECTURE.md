# Architecture

This document explains how NullSector structures its code, and why it runs
external commands the way it does. It is for contributors who want to add
a panel or change how a command runs.

## Crate layout

- `src/main.rs`: the native entry point. Sets up logging
  (`tracing_subscriber`) and starts the `eframe` window with a `BootCon`
  instance.
- `src/lib.rs`: the crate root. Re-exports `BootCon` and sets the two
  crate-wide lint attributes described below.
- `src/app.rs`: everything else. The `BootCon` struct (app state), the
  `eframe::App` implementation (UI layout, one `ui` call per frame), and
  every function that runs an external command.

There is no `wasm32` target. Some dependencies here, such as
`local-ip-address`, do not support it.

## No panics in application code

`lib.rs` and `main.rs` both set:

```rust
#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::expect_used))]
```

This turns a use of `.unwrap()` or `.expect()` outside test code into a
compile error. A GUI app has no terminal to print a panic message to on
most platforms. A panic there just closes the window with no explanation.
Every fallible call in `app.rs` returns a `Result` that the UI matches on.
Other calls go through `spawn_or_warn`. This function logs a warning with
`tracing::warn!` instead of panicking when a command fails to start.

## Running external commands

NullSector runs every external tool one of three ways, depending on
whether the user needs to see its output.

### `spawn_or_warn`

For commands whose result does not matter to the user, such as clearing a
terminal or downloading a file in the background. Spawns the command and
logs a warning on failure. Does not wait for the command to finish and
does not show its output.

### `spawn_terminal`

For opening an empty, interactive terminal window (the "Hook into Term"
button). On Windows, this launches PowerShell directly. A GUI process on
Windows can start a console-subsystem process. That process gets its own
window automatically. Linux and macOS have no such mechanism. This
function launches a terminal *emulator* instead, and passes the user's
shell to it as the command to run.

### `run_visibly`

For every diagnostic and scan command (`dig`, `whois`, `ping`, `nslookup`,
`nmap`, the weather `curl` calls, and the PEAS scripts). These commands
produce output the user needs to read.

`Command::spawn` inherits the parent process's stdout and stderr by
default. For a GUI app started from a desktop launcher, those streams are
not attached to any terminal. A command run this way produces output
nobody can see. `run_visibly` avoids this by running the command inside a
real terminal, the same way `spawn_terminal` opens one. It also appends a
"press Enter to close" prompt, so the output does not flash and disappear
when the command exits.

This also means a command that needs a password (`sudo nmap`, for
`-sC`/`-sV` scans on Linux and macOS) has a real terminal to prompt into.

### Finding a terminal emulator

Windows and macOS each have one terminal program NullSector can always
assume is present (PowerShell and Terminal.app). Linux has no such
default. `spawn_linux_terminal` tries the `$TERMINAL` environment variable
first. Several window managers and desktop environments set this
variable. If it is unset or fails, the function tries a fixed list of
common terminal emulators instead (`LINUX_TERMINALS` in `app.rs`), until
one of them starts.

Most terminal emulators accept `-e <command>` to run a command instead of
their default shell. Two do not:

- `gnome-terminal` deprecated `-e` in favor of `--`.
- `wezterm` has no `-e` flag. It uses the `start --` subcommand instead.

`try_spawn_terminal` handles both cases.

### Shell quoting

`run_visibly` and `spawn_terminal` build a `sh -c '...'` command line as a
single string. A terminal emulator's `-e` flag takes exactly this kind of
string. `posix_quote` wraps each argument in single quotes, and escapes
any single quote inside it. This stops user-supplied text, such as a
hostname or a port list, from breaking out of its argument and running as
separate shell syntax. `posix_command_line` builds the full line from a
program name and its arguments, quoting each part.

On macOS, `run_visibly` builds the same quoted command line, then wraps it
again for AppleScript. AppleScript is how Terminal.app is scripted to run
a command in a new window.

## Persisted state

`BootCon` derives `serde::Serialize`/`Deserialize` and marks a few fields
`#[serde(skip)]`: `host`, `weather`, and `public_ip`. These reset to their
defaults on every launch. `weather` and `host` are separate example
values, so they do not carry over from `target`, which is used for nmap
scans specifically. `public_ip` is looked up fresh on every launch instead
of persisted, since a saved IP address can go stale.

## Adding a new tool button

1. Add any input fields it needs as `BootCon` struct fields, with a
   default in `Default::default()`.
2. Add the button inside the relevant `ui.collapsing(...)` block.
3. Call `run_visibly(program, args)` if the command produces output the
   user should read, or `spawn_or_warn(Command::new(...), description)`
   for a fire-and-forget command.
4. Never call `.unwrap()` or `.expect()`. The crate-wide lint rejects
   this outside tests.
