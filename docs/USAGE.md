# Usage guide

This guide describes each panel in the NullSector window and what its
buttons do.

## Menu bar

The top menu bar has a **File** menu and a theme switch.

- **Clear the Terminal** runs `clear` (or `powershell -c clear` on
  Windows). This does not clear the NullSector window. It targets the
  terminal the app was started from, if any.
- **Exit** closes the app.
- The theme switch toggles between light and dark mode.

## Side panel: Common Commands

### Hook into Term

Opens a new terminal window running your shell (`$SHELL` on Linux and
macOS, PowerShell on Windows). Use this to run commands NullSector does
not have a button for.

### Local Network Config

Shows the local network configuration in a terminal window:

- `ip addr show` on Linux
- `ifconfig` on macOS
- `ipconfig /all` on Windows

### NMAP

Scans the target address with `nmap`, using the flags you select:

- **Target** is the IP address or hostname to scan.
- **-sC** runs nmap's default scripts.
- **-sV** detects service and version information.
- **-v** turns on verbose output.
- **-p (ports)** limits the scan to specific ports. Enter a list
  (`22,80,443`) or a range (`1-1000`).
- **-oA** saves the scan output to files named after the target.

Click **Send it!** to start the scan in a terminal window. On Linux and
macOS, the app runs `sudo nmap ...`, since `-sC` and `-sV` need root. Enter
your password in the terminal window when prompted.

### Network Tools

- **Target** is the hostname to look up.
- **NSLOOKUP** row: looks up NS, MX, TXT, or ANY records for the target.
- **DIG**: runs `dig <target>`.
- **WHOIS**: runs `whois <target>`.
- **PING**: pings the target 4 times on Linux and macOS, or with the
  default count on Windows.

### Weather

- **Closest Major City** sets the location for the weather lookup. Use a
  city name, with spaces replaced by `+` (for example, `Kansas+City`).
- **Current Weather** shows a one-line summary from `wttr.in`.
- **3-Day Forecast** shows a longer forecast from `wttr.in`.

Both buttons need an internet connection and `curl`.

## Central panel

### PEAS Download

PEAS (Privilege Escalation Awesome Script) searches a machine for possible
privilege-escalation paths. See the linked
[Hack Tricks](https://book.hacktricks.xyz/) page for background.

- **Download PEAS** downloads winPEAS (Windows) or linPEAS (Linux/macOS)
  from the [PEASS-ng](https://github.com/carlospolop/PEASS-ng) releases,
  into the current working directory.
- **Run PEAS** runs the downloaded script in a terminal window. Download
  it first.

### Host Info

Shows information about the current machine and user:

- Public IP (looked up from `ipinfo.io`)
- Local IP address
- Device platform and OS distribution
- Device name, hostname, and desktop environment
- User's real name, username, and language preference

A field that NullSector cannot read shows a message instead of a value.
Most fields show this in red. The device name field is an exception. It
shows a muted "not set on this system" message when `/etc/machine-info`
does not exist. This file is optional on most Linux systems, so its
absence is not an error.

### Disclaimer

Lists the external tools that Windows users need to install manually
(`nmap`, `dig`, `whois`), with Chocolatey install commands for `dig` and
`whois`.

## Persistence

NullSector saves your target host, nmap settings, and weather location
between sessions, using eframe's built-in persistence.
