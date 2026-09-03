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
                if cfg!(target_os = "windows") {
                    spawn_or_warn(Command::new("powershell.exe"), "PowerShell");
                } else {
                    spawn_or_warn(Command::new("zsh"), "Terminal (zsh)");
                }
            }

            if ui.button("Local Network Config").clicked() {
                if cfg!(target_os = "windows") {
                    let mut cmd = Command::new("ipconfig");
                    cmd.arg("/all");
                    spawn_or_warn(cmd, "Windows ipconfig");
                } else if cfg!(target_os = "macos") {
                    spawn_or_warn(Command::new("ifconfig"), "macOS ifconfig");
                } else {
                    let mut cmd = Command::new("ip");
                    cmd.args(["addr", "show"]);
                    spawn_or_warn(cmd, "Linux `ip addr show`");
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

                    if cfg!(target_os = "windows") {
                        let mut cmd = Command::new("nmap");
                        cmd.args(&flags)
                            .args([&self.target, "-oA", &self.target]);
                        spawn_or_warn(cmd, "NMAP (Windows)");
                    } else {
                        let mut cmd = Command::new("sudo");
                        cmd.arg("nmap")
                            .args(&flags)
                            .args([&self.target, "-oA", &self.target]);
                        spawn_or_warn(cmd, "NMAP");
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
                        let mut cmd = Command::new("nslookup");
                        cmd.args(["-type=NS", &self.host]);
                        spawn_or_warn(cmd, "nslookup (NS)");
                    }
                    if ui.button("MX").clicked() {
                        let mut cmd = Command::new("nslookup");
                        cmd.args(["-type=MX", &self.host]);
                        spawn_or_warn(cmd, "nslookup (MX)");
                    }
                    if ui.button("TXT").clicked() {
                        let mut cmd = Command::new("nslookup");
                        cmd.args(["-type=TXT", &self.host]);
                        spawn_or_warn(cmd, "nslookup (TXT)");
                    }
                    if ui.button("ANY").clicked() {
                        let mut cmd = Command::new("nslookup");
                        cmd.args(["-type=any", &self.host]);
                        spawn_or_warn(cmd, "nslookup (ANY)");
                    }
                });
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("DIG").clicked() {
                        let mut cmd = Command::new("dig");
                        cmd.arg(&self.host);
                        spawn_or_warn(cmd, "DIG");
                    }

                    if ui.button("WHOIS").clicked() {
                        let mut cmd = Command::new("whois");
                        cmd.arg(&self.host);
                        spawn_or_warn(cmd, "WHOIS");
                    }

                    if ui.button("PING").clicked() {
                        if cfg!(target_os = "windows") {
                            let mut cmd = Command::new("ping");
                            cmd.arg(&self.host);
                            spawn_or_warn(cmd, "PING (Windows)");
                        } else {
                            let mut cmd = Command::new("ping");
                            cmd.args(["-c", "4", &self.host]);
                            spawn_or_warn(cmd, "PING");
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
                    let mut cmd = Command::new("curl");
                    cmd.arg("-s")
                        .arg(format!("http://wttr.in/{}?format=3", self.weather));
                    spawn_or_warn(cmd, "Weather (current)");
                }
                if ui.button("3-Day Forcast").clicked() {
                    let mut cmd = Command::new("curl");
                    cmd.arg("-s")
                        .arg(format!("http://wttr.in/{}", self.weather));
                    spawn_or_warn(cmd, "Weather (3-day)");
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
            ui.hyperlink_to("BootCampSpot", "https://bootcampspot.com/login");
            ui.hyperlink_to("KU GitLab", "https://ku.bootcampcontent.com/");
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
                            let mut cmd = Command::new("powershell.exe");
                            cmd.arg("-c").arg(".\\winPEAS.bat");
                            spawn_or_warn(cmd, "Run WinPEAS.bat");
                        } else {
                            let mut cmd = Command::new("sh");
                            cmd.arg("./linpeas.sh");
                            spawn_or_warn(cmd, "Run LinPEAS.sh");
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
                    Ok(name) => ui.label(format!("Device's 'Pretty' Name: {name}")),
                    Err(err) => ui.colored_label(egui::Color32::RED, format!("Device name: unavailable ({err})")),
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
