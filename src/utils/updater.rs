use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

pub const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const GITHUB_REPO: &str = "Saboor-hamedi/search_forge";

#[derive(Debug, Clone)]
pub enum UpdateState {
    Idle,
    Checking,
    UpdateAvailable {
        version: String,
        download_url: String,
        file_name: String,
        total_bytes: u64,
    },
    UpToDate,
    Downloading {
        progress: f32, // 0.0 to 1.0
        downloaded: u64,
        total: u64,
        file_name: String,
    },
    ReadyToRestart {
        installer_path: PathBuf,
        version: String,
    },
    Error(String),
}

pub struct UpdateManager {
    state: UpdateState,
    event_rx: Option<Receiver<UpdateEvent>>,
    event_tx: Sender<UpdateEvent>,
}

pub enum UpdateEvent {
    FoundUpdate {
        version: String,
        download_url: String,
        file_name: String,
        total_bytes: u64,
    },
    NoUpdate,
    Progress {
        downloaded: u64,
        total: u64,
    },
    Downloaded {
        path: PathBuf,
        version: String,
    },
    Failed(String),
}

impl UpdateManager {
    pub fn new() -> Self {
        let (tx, rx) = channel();
        Self {
            state: UpdateState::Idle,
            event_rx: Some(rx),
            event_tx: tx,
        }
    }

    pub fn state(&self) -> &UpdateState {
        &self.state
    }

    pub fn poll_updates(&mut self) {
        if let Some(ref rx) = self.event_rx {
            while let Ok(event) = rx.try_recv() {
                match event {
                    UpdateEvent::FoundUpdate {
                        version,
                        download_url,
                        file_name,
                        total_bytes,
                    } => {
                        self.state = UpdateState::UpdateAvailable {
                            version,
                            download_url,
                            file_name,
                            total_bytes,
                        };
                    }
                    UpdateEvent::NoUpdate => {
                        self.state = UpdateState::UpToDate;
                    }
                    UpdateEvent::Progress { downloaded, total } => {
                        let progress = if total > 0 {
                            (downloaded as f32 / total as f32).clamp(0.0, 1.0)
                        } else {
                            0.0
                        };
                        let file_name = match &self.state {
                            UpdateState::Downloading { file_name, .. } => file_name.clone(),
                            UpdateState::UpdateAvailable { file_name, .. } => file_name.clone(),
                            _ => "installer.exe".to_string(),
                        };
                        self.state = UpdateState::Downloading {
                            progress,
                            downloaded,
                            total,
                            file_name,
                        };
                    }
                    UpdateEvent::Downloaded { path, version } => {
                        self.state = UpdateState::ReadyToRestart {
                            installer_path: path,
                            version,
                        };
                    }
                    UpdateEvent::Failed(err) => {
                        self.state = UpdateState::Error(err);
                    }
                }
            }
        }
    }

    pub fn check_for_updates(&mut self) {
        self.state = UpdateState::Checking;
        let tx = self.event_tx.clone();

        thread::spawn(move || {
            let api_url = format!("https://api.github.com/repos/{}/releases/latest", GITHUB_REPO);
            let mut req = ureq::get(&api_url)
                .set("User-Agent", "SearchForge-App")
                .set("Accept", "application/vnd.github.v3+json");

            if let Ok(token) = std::env::var("GITHUB_TOKEN").or_else(|_| std::env::var("GH_TOKEN")) {
                req = req.set("Authorization", &format!("Bearer {}", token));
            }

            let resp = match req.call() {
                Ok(r) => r,
                Err(e) => {
                    let _ = tx.send(UpdateEvent::Failed(format!("Network error: {}", e)));
                    return;
                }
            };

            let json: SerdeJsonValueMinimal = match resp.into_json() {
                Ok(v) => v,
                Err(e) => {
                    let _ = tx.send(UpdateEvent::Failed(format!("Parse error: {}", e)));
                    return;
                }
            };

            let tag = json.tag_name.trim_start_matches('v');
            if is_newer_version(CURRENT_VERSION, tag) {
                // Find Windows installer or binary
                let (url, name, size) = find_best_asset(&json.assets);
                if let Some(download_url) = url {
                    let _ = tx.send(UpdateEvent::FoundUpdate {
                        version: tag.to_string(),
                        download_url,
                        file_name: name.unwrap_or_else(|| "SearchForge-Setup.exe".to_string()),
                        total_bytes: size,
                    });
                } else {
                    let _ = tx.send(UpdateEvent::NoUpdate);
                }
            } else {
                let _ = tx.send(UpdateEvent::NoUpdate);
            }
        });
    }

    pub fn start_download(&mut self) {
        if let UpdateState::UpdateAvailable {
            version,
            download_url,
            file_name,
            total_bytes,
        } = &self.state
        {
            let ver = version.clone();
            let url = download_url.clone();
            let fname = file_name.clone();
            let total = *total_bytes;
            let tx = self.event_tx.clone();

            self.state = UpdateState::Downloading {
                progress: 0.0,
                downloaded: 0,
                total,
                file_name: fname.clone(),
            };

            thread::spawn(move || {
                let temp_dir = std::env::temp_dir();
                let dest_path = temp_dir.join(&fname);

                let req = ureq::get(&url)
                    .set("User-Agent", "SearchForge-App");

                let resp = match req.call() {
                    Ok(r) => r,
                    Err(e) => {
                        let _ = tx.send(UpdateEvent::Failed(format!("Download failed: {}", e)));
                        return;
                    }
                };

                let total_size: u64 = resp
                    .header("Content-Length")
                    .and_then(|l| l.parse().ok())
                    .unwrap_or(total);

                let mut reader = resp.into_reader();
                let mut out_file = match std::fs::File::create(&dest_path) {
                    Ok(f) => f,
                    Err(e) => {
                        let _ = tx.send(UpdateEvent::Failed(format!("Cannot create file: {}", e)));
                        return;
                    }
                };

                let mut downloaded: u64 = 0;
                let mut buffer = [0u8; 16384];

                use std::io::{Read, Write};
                loop {
                    match reader.read(&mut buffer) {
                        Ok(0) => break,
                        Ok(n) => {
                            if let Err(e) = out_file.write_all(&buffer[..n]) {
                                let _ = tx.send(UpdateEvent::Failed(format!("Write error: {}", e)));
                                return;
                            }
                            downloaded += n as u64;
                            let _ = tx.send(UpdateEvent::Progress {
                                downloaded,
                                total: total_size,
                            });
                        }
                        Err(e) => {
                            let _ = tx.send(UpdateEvent::Failed(format!("Read error: {}", e)));
                            return;
                        }
                    }
                }

                let _ = tx.send(UpdateEvent::Downloaded {
                    path: dest_path,
                    version: ver,
                });
            });
        }
    }

    pub fn restart_and_install(&self) {
        if let UpdateState::ReadyToRestart { installer_path, .. } = &self.state {
            #[cfg(target_os = "windows")]
            {
                let path_str = installer_path.to_string_lossy().to_string();
                let _ = std::process::Command::new(&path_str).spawn();
            }
            std::process::exit(0);
        }
    }
}

// Minimal manual JSON model to eliminate heavy json parser dependency
#[derive(serde::Deserialize, Default)]
struct SerdeJsonValueMinimal {
    #[serde(default)]
    tag_name: String,
    #[serde(default)]
    assets: Vec<AssetMinimal>,
}

#[derive(serde::Deserialize, Default)]
struct AssetMinimal {
    #[serde(default)]
    name: String,
    #[serde(default)]
    browser_download_url: String,
    #[serde(default)]
    size: u64,
}

fn is_newer_version(current: &str, candidate: &str) -> bool {
    let parse = |v: &str| -> Vec<u64> {
        v.split('.')
            .map(|s| s.trim_matches(|c: char| !c.is_ascii_digit()).parse::<u64>().unwrap_or(0))
            .collect()
    };
    let c = parse(current);
    let n = parse(candidate);
    n > c
}

fn find_best_asset(assets: &[AssetMinimal]) -> (Option<String>, Option<String>, u64) {
    // Prefer Windows user setup installer
    for a in assets {
        if a.name.to_lowercase().contains("usersetup") || (a.name.to_lowercase().contains("setup") && a.name.ends_with(".exe")) {
            return (Some(a.browser_download_url.clone()), Some(a.name.clone()), a.size);
        }
    }
    // Fallback to any .exe
    for a in assets {
        if a.name.ends_with(".exe") {
            return (Some(a.browser_download_url.clone()), Some(a.name.clone()), a.size);
        }
    }
    // Fallback to .zip
    for a in assets {
        if a.name.ends_with(".zip") {
            return (Some(a.browser_download_url.clone()), Some(a.name.clone()), a.size);
        }
    }
    (None, None, 0)
}
