use std::process::Command;

use local_ip_address::local_ip;

/// Runs a command in the background and reports failures instead of panicking.
///
/// A failure here means the OS could not start the process (for example, the
/// binary is missing). We show a warning in the log instead of crashing the
/// whole application over an optional tool not being installed.
fn spawn_or_warn(mut command: Command, what: &str) {
    if let Err(err) = command.spawn() {
        tracing::warn!("{what} failed to start: {err}");
    }
}

/// Quotes a single argument for POSIX shells by wrapping it in single quotes.
///
/// This is what keeps user-supplied text (a hostname, a target, a port
/// list) from being interpreted as shell syntax when it is spliced into a
/// command line string for a terminal emulator to run.
fn posix_quote(arg: &str) -> String {
    format!("'{}'", arg.replace('\'', r"'\''"))
}

/// Builds a `sh`-compatible command line from a program and its arguments,
/// with every part safely quoted.
fn posix_command_line(program: &str, args: &[&str]) -> String {
    let mut line = posix_quote(program);
    for arg in args {
        line.push(' ');
        line.push_str(&posix_quote(arg));
    }
    line
}

/// Terminal emulators to try, in order.
///
/// Unlike Windows, Linux has no single terminal every install ships with, so
/// we probe a list of common ones. `$TERMINAL` (a convention several window
/// managers and desktop environments set) is tried first.
const LINUX_TERMINALS: &[&str] = &[
    "alacritty",
    "kitty",
    "wezterm",
    "konsole",
    "gnome-terminal",
    "xfce4-terminal",
    "tilix",
    "terminator",
    "foot",
    "xterm",
    "urxvt",
];

/// Tries to launch `terminal` running the given `sh -c`-style command line.
///
/// Most terminal emulators accept `-e <command>` to run a command instead of
/// their default shell, but `gnome-terminal` deprecated `-e` in favor of
/// `--`, and `wezterm` has no `-e` at all and instead uses its `start --`
/// subcommand. Returns `true` if the process was spawned successfully.
fn try_spawn_terminal(terminal: &str, sh_command_line: &str) -> bool {
    let mut cmd = Command::new(terminal);
    match terminal {
        "gnome-terminal" => {
            cmd.args(["--", "sh", "-c", sh_command_line]);
        }
        "wezterm" => {
            cmd.args(["start", "--", "sh", "-c", sh_command_line]);
        }
        _ => {
            cmd.args(["-e", "sh", "-c", sh_command_line]);
        }
    }
    cmd.spawn().is_ok()
}

/// Opens a terminal emulator on Linux running `sh_command_line`.
///
/// Tries `$TERMINAL` first (a convention several window managers and
/// desktop environments set), then falls back through [`LINUX_TERMINALS`].
/// Logs a warning instead of doing nothing if none of them are installed.
fn spawn_linux_terminal(sh_command_line: &str) {
    let term = std::env::var("TERMINAL").unwrap_or_default();
    if !term.is_empty() {
        if try_spawn_terminal(&term, sh_command_line) {
            return;
        }
        tracing::warn!("$TERMINAL ({term}) failed to start, trying known terminals");
    }

    for terminal in LINUX_TERMINALS {
        if try_spawn_terminal(terminal, sh_command_line) {
            return;
        }
    }

    tracing::warn!(
        "no terminal emulator found (tried $TERMINAL and {} known terminals)",
        LINUX_TERMINALS.len()
    );
}

/// Opens a new terminal window running an interactive shell.
///
/// On Windows this launches PowerShell directly, since Windows gives console
/// apps their own window automatically: a console-subsystem process spawned
/// by a GUI-subsystem process gets its own console window for free. On
/// Linux and macOS there is no such mechanism: a shell run directly from a
/// GUI has no terminal attached and is invisible to the user, so a terminal
/// *emulator* has to be launched instead, with the shell passed to it as
/// the command to run.
fn spawn_terminal() {
    if cfg!(target_os = "windows") {
        spawn_or_warn(Command::new("powershell.exe"), "PowerShell");
        return;
    }

    if cfg!(target_os = "macos") {
        let mut cmd = Command::new("open");
        cmd.args(["-a", "Terminal"]);
        spawn_or_warn(cmd, "Terminal.app");
        return;
    }

    let shell = std::env::var("SHELL").unwrap_or_else(|_| "sh".to_string());
    spawn_linux_terminal(&posix_quote(&shell));
}

/// Runs `program args...` inside a visible terminal so its output can
/// actually be seen, then reports the failure to start it, if any.
///
/// `Command::spawn` inherits this GUI app's own stdout/stderr, which is not
/// attached to any terminal the user is watching, so a command spawned
/// directly produces output that goes nowhere visible. This wraps the
/// command in a terminal emulator (or PowerShell/Terminal.app) the same way
/// [`spawn_terminal`] does, instead of running it headless.
fn run_visibly(program: &str, args: &[&str]) {
    if cfg!(target_os = "windows") {
        // PowerShell's own argument quoting is handled by `Command`, so the
        // program and its arguments can be passed through directly instead
        // of being assembled into one string. `-NoExit` keeps the window
        // open after the command finishes so the output can be read.
        let mut cmd = Command::new("powershell.exe");
        cmd.arg("-NoExit").arg("-Command").arg(program).args(args);
        spawn_or_warn(cmd, program);
        return;
    }

    if cfg!(target_os = "macos") {
        // AppleScript is the standard way to ask Terminal.app to run a
        // command in a fresh window; see
        // https://apple.stackexchange.com/questions/205143.
        let command_line = posix_command_line(program, args);
        let script = format!(
            "tell application \"Terminal\" to do script \"{}\"",
            command_line.replace('\\', "\\\\").replace('"', "\\\"")
        );
        let mut cmd = Command::new("osascript");
        cmd.args(["-e", &script]);
        spawn_or_warn(cmd, "Terminal.app");
        return;
    }

    // Keep the terminal open after the command finishes so the output does
    // not flash and disappear as soon as the command exits.
    let command_line = format!(
        "{}; printf '\\n[press Enter to close] '; read _",
        posix_command_line(program, args)
    );
    spawn_linux_terminal(&command_line);
}

/// Fetches the public IP address as reported by `ipinfo.io`.
///
/// Returns an error message instead of the address when the lookup fails, so
/// the UI can show the reason instead of crashing.
fn fetch_public_ip() -> Result<String, String> {
    let output = Command::new("curl")
        .arg("ipinfo.io/ip")
        .output()
        .map_err(|err| format!("could not run curl: {err}"))?;

    if !output.status.success() {
        return Err(format!("curl exited with status {}", output.status));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct BootCon {
    target: String,
    nmap_default_scripts: bool,
    nmap_service_version: bool,
    nmap_verbose: bool,
    nmap_use_custom_ports: bool,
    nmap_ports: String,
    nmap_save_output: bool,

    #[serde(skip)]
    host: String,
    #[serde(skip)]
    weather: String,
    #[serde(skip)]
    public_ip: Result<String, String>,
}

impl Default for BootCon {
    fn default() -> Self {
        Self {
            host: "example.com".to_string(),
            target: "127.0.0.1".to_string(),
            weather: "Kansas+City".to_string(),
            nmap_default_scripts: true,
            nmap_service_version: true,
            nmap_verbose: true,
            nmap_use_custom_ports: false,
            nmap_ports: String::new(),
            nmap_save_output: true,
            public_ip: fetch_public_ip(),
        }
    }
}

impl BootCon {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        cc.egui_ctx.set_visuals(egui::Visuals::dark());

        // Load previous app state (if any).
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for BootCon {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::top("top_panel").show(ui, |ui| {
            // The top panel for a menu bar:
            egui::menu::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Clear the Terminal").clicked() {
                        if cfg!(target_os = "windows") {
                            let mut cmd = Command::new("powershell");
                            cmd.arg("-c").arg("clear");
                            spawn_or_warn(cmd, "Clear Term (powershell)");
                        } else {
                            spawn_or_warn(Command::new("clear"), "Clear Term");
                        }
                    }
                    if ui.button("Exit").clicked() {
                        ui.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                egui::widgets::global_theme_preference_switch(ui);
            });
        });

        egui::Panel::left("side_panel").show(ui, |ui| {
            ui.heading("Common Commands");

            if ui.button("Hook into Term").clicked() {
                spawn_terminal();
            }

            if ui.button("Local Network Config").clicked() {
                if cfg!(target_os = "windows") {
                    run_visibly("ipconfig", &["/all"]);
                } else if cfg!(target_os = "macos") {
                    run_visibly("ifconfig", &[]);
                } else {
                    run_visibly("ip", &["addr", "show"]);
                }
            }

            ui.separator();
            ui.collapsing("NMAP", |ui| {
                ui.horizontal(|ui| {
                    ui.label("Target: ");
                    ui.text_edit_singleline(&mut self.target);
                });

                ui.checkbox(&mut self.nmap_default_scripts, "-sC (default scripts)");
                ui.checkbox(&mut self.nmap_service_version, "-sV (service/version detection)");
                ui.checkbox(&mut self.nmap_verbose, "-v (verbose)");

                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.nmap_use_custom_ports, "-p (ports)");
                    ui.add_enabled(
                        self.nmap_use_custom_ports,
                        egui::TextEdit::singleline(&mut self.nmap_ports)
                            .hint_text("e.g. 22,80,443 or 1-1000"),
                    );
                });

                ui.checkbox(&mut self.nmap_save_output, "-oA (save output to files named after the target)");

                if ui.button("Send it!").clicked() {
                    let mut flags: Vec<&str> = Vec::new();
                    if self.nmap_default_scripts {
                        flags.push("-sC");
                    }
                    if self.nmap_service_version {
                        flags.push("-sV");
                    }
                    if self.nmap_verbose {
                        flags.push("-v");
                    }
                    if self.nmap_use_custom_ports && !self.nmap_ports.trim().is_empty() {
                        flags.push("-p");
                        flags.push(self.nmap_ports.trim());
                    }

                    let mut output_args: Vec<&str> = Vec::new();
                    if self.nmap_save_output {
                        output_args.push("-oA");
                        output_args.push(&self.target);
                    }

                    let mut nmap_args: Vec<&str> = flags;
                    nmap_args.push(&self.target);
                    nmap_args.extend(&output_args);

                    if cfg!(target_os = "windows") {
                        run_visibly("nmap", &nmap_args);
                    } else {
                        // nmap needs root for the scan types this app enables by
                        // default (`-sC`/`-sV`), so it is run through `sudo`.
                        // Running it in a real terminal (via `run_visibly`) means
                        // `sudo` can actually prompt for a password instead of
                        // failing or hanging with no visible output.
                        let mut sudo_args: Vec<&str> = vec!["nmap"];
                        sudo_args.extend(&nmap_args);
                        run_visibly("sudo", &sudo_args);
                    }
                }
            });
            ui.separator();
            ui.collapsing("Network Tools", |ui| {
                ui.horizontal(|ui| {
                    ui.label("Target: ");
                    ui.text_edit_singleline(&mut self.host);
                });

                ui.horizontal(|ui| {
                    ui.label("NSLOOKUP:");
                    if ui.button("NS").clicked() {
                        run_visibly("nslookup", &["-type=NS", &self.host]);
                    }
                    if ui.button("MX").clicked() {
                        run_visibly("nslookup", &["-type=MX", &self.host]);
                    }
                    if ui.button("TXT").clicked() {
                        run_visibly("nslookup", &["-type=TXT", &self.host]);
                    }
                    if ui.button("ANY").clicked() {
                        run_visibly("nslookup", &["-type=any", &self.host]);
                    }
                });
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("DIG").clicked() {
                        run_visibly("dig", &[&self.host]);
                    }

                    if ui.button("WHOIS").clicked() {
                        run_visibly("whois", &[&self.host]);
                    }

                    if ui.button("PING").clicked() {
                        if cfg!(target_os = "windows") {
                            run_visibly("ping", &[&self.host]);
                        } else {
                            run_visibly("ping", &["-c", "4", &self.host]);
                        }
                    }
                });
                ui.separator();
            });

            ui.separator();
            ui.collapsing("Weather", |ui| {
                ui.horizontal(|ui| {
                    ui.label("Closest Major City: ");
                    ui.text_edit_singleline(&mut self.weather);
                });

                if ui.button("Current Weather").clicked() {
                    let url = format!("http://wttr.in/{}?format=3", self.weather);
                    run_visibly("curl", &["-s", &url]);
                }
                if ui.button("3-Day Forcast").clicked() {
                    let url = format!("http://wttr.in/{}", self.weather);
                    run_visibly("curl", &["-s", &url]);
                }
                ui.separator();
            });
            ui.separator();

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.label("powered by ");
                    ui.hyperlink_to(
                        "Curiosity",
                        "https://www.merriam-webster.com/dictionary/curiosity",
                    );
                    ui.label(" and ");
                    ui.hyperlink_to(
                        "Insomnia",
                        "https://www.mayoclinic.org/diseases-conditions/insomnia/symptoms-causes/syc-20355167",
                    );
                });
            });
        });

        egui::CentralPanel::default().show(ui, |ui| {
            // The central panel is the region left after adding TopPanel's and SidePanel's.
            ui.heading("KU Cybersecurity 2022");
            egui::warn_if_debug_build(ui);

            ui.separator();

            ui.collapsing("PEAS Download:", |ui| {
                ui.label(
                    "    - This button will allow you to download the PEAS, no matter what OS you are using (winPEAS, linPEAS)

        - PEAS = Privilege Escalation Awesome Script

        - PEAS searches for possible paths to escalate privileges

    - Will download the file to your $HOME DIR or the same DIR the app was ran from.",
                );
                ui.hyperlink_to("Hack Tricks", "https://book.hacktricks.xyz/");
                ui.horizontal(|ui| {
                    if ui.button("Download PEAs").clicked() {
                        if cfg!(target_os = "windows") {
                            let mut cmd = Command::new("curl");
                            cmd.arg("-L").arg("-O").arg(
                                "https://github.com/carlospolop/PEASS-ng/releases/latest/download/winPEAS.bat",
                            );
                            spawn_or_warn(cmd, "WinPEAS download");
                        } else {
                            let mut cmd = Command::new("curl");
                            cmd.arg("-L").arg("-O").arg(
                                "https://github.com/carlospolop/PEASS-ng/releases/latest/download/linpeas.sh",
                            );
                            spawn_or_warn(cmd, "LinPEAS download");
                        }
                    }
                    if ui.button("Run PEAs").clicked() {
                        if cfg!(target_os = "windows") {
                            run_visibly(".\\winPEAS.bat", &[]);
                        } else {
                            run_visibly("sh", &["./linpeas.sh"]);
                        }
                    }
                });
            });
            ui.separator();
            ui.collapsing("Host Info", |ui| {
                match &self.public_ip {
                    Ok(ip) => ui.label(format!("Public IP: {ip}")),
                    Err(err) => ui.colored_label(egui::Color32::RED, format!("Public IP: unavailable ({err})")),
                };
                match local_ip() {
                    Ok(local_ip) => {
                        ui.label(format!("Local IP: {local_ip}"));
                    }
                    Err(err) => {
                        ui.colored_label(
                            egui::Color32::RED,
                            format!("Local IP: unavailable ({err})"),
                        );
                    }
                }
                ui.label(format!("Device Platform: {}", whoami::platform()));
                match whoami::distro() {
                    Ok(distro) => ui.label(format!("OS Distro: {distro}")),
                    Err(err) => ui.colored_label(egui::Color32::RED, format!("OS Distro: unavailable ({err})")),
                };
                match whoami::devicename() {
                    Ok(name) => {
                        ui.label(format!("Device's 'Pretty' Name: {name}"));
                    }
                    Err(err) => {
                        let io_err: std::io::Error = err.into();
                        if io_err.kind() == std::io::ErrorKind::NotFound {
                            // Most Linux systems never set a pretty device
                            // name (it lives in `/etc/machine-info`, which
                            // is optional), so a missing-file error here
                            // just means nobody configured one, not a
                            // failure.
                            ui.weak("Device's 'Pretty' Name: not set on this system");
                        } else {
                            ui.colored_label(
                                egui::Color32::RED,
                                format!("Device name: unavailable ({io_err})"),
                            );
                        }
                    }
                };
                match whoami::hostname() {
                    Ok(hostname) => ui.label(format!("Hostname: {hostname}")),
                    Err(err) => ui.colored_label(egui::Color32::RED, format!("Hostname: unavailable ({err})")),
                };
                match whoami::desktop_env() {
                    Some(desktop_env) => ui.label(format!("Desktop Env: {desktop_env}")),
                    None => ui.label("Desktop Env: unknown"),
                };

                ui.separator();
                ui.heading("User Info:");
                match whoami::realname() {
                    Ok(name) => ui.label(format!("User's Name: {name}")),
                    Err(err) => ui.colored_label(egui::Color32::RED, format!("User's Name: unavailable ({err})")),
                };
                match whoami::username() {
                    Ok(name) => ui.label(format!("User's Username: {name}")),
                    Err(err) => ui.colored_label(egui::Color32::RED, format!("User's Username: unavailable ({err})")),
                };
                match whoami::lang_prefs() {
                    Ok(prefs) => ui.label(format!("User's Language: {prefs}")),
                    Err(err) => ui.colored_label(egui::Color32::RED, format!("User's Language: unavailable ({err})")),
                };
            });
            ui.separator();
            ui.collapsing("Disclaimer:", |ui| {
                ui.label("\t- On Windows, `NMAP`, `DIG`, and `WHOIS` need to be installed and on PATH. If they are missing, the button logs a warning instead of running the tool");
                ui.label("\t\t- You can use Chocolatey on Windows to install DIG and WHOIS");
                ui.label("\t\t- DIG -- `choco install bind-toolsonly`");
                ui.label("\t\t- WHOIS -- `choco install whois`");
            });

            ui.with_layout(egui::Layout::bottom_up(egui::Align::RIGHT), |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.label("Created by Ursus949");
                });
            });
        });
    }
}
