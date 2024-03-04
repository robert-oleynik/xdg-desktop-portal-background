use std::path::{Path, PathBuf};
use std::str::FromStr;

use anyhow::Context;
use configparser::ini::Ini;

/// Used to manage all background applications.
pub struct System {
    autostart_dir: PathBuf,
}

impl Default for System {
    fn default() -> Self {
        let home = std::env::var("HOME").unwrap();
        let mut home = PathBuf::from_str(&home).unwrap();
        home.push(".config");
        home.push("autostart");
        Self {
            autostart_dir: home,
        }
    }
}

pub struct App {
    pub id: String,
    pub autostart: bool,
    pub cmd: Vec<String>,
    pub flags: u32,
}

impl System {
    pub async fn add_autostart(&mut self, app: &App) -> anyhow::Result<()> {
        let path = self.app_path(&app.id);
        if tokio::fs::try_exists(&path).await.unwrap_or(false) {
            log::warn!("overiting app config. app_id={:?}", app.id);
        }
        store_app(path, &app).await
    }

    pub async fn get_autostart(&mut self, id: &str) -> anyhow::Result<Option<App>> {
        load_app(self.app_path(id)).await
    }

    pub async fn list_apps(&self) -> anyhow::Result<Vec<App>> {
        let app_dir = self.autostart_dir();
        let mut apps = Vec::new();
        let mut files = tokio::fs::read_dir(app_dir)
            .await
            .context("failed to read apps")?;
        loop {
            let entry = match files.next_entry().await {
                Ok(Some(entry)) => entry,
                Ok(None) => break,
                Err(err) => {
                    log::error!("failed to fetch app entry: {err:?}");
                    continue;
                }
            };
            let path = entry.path();
            if path.is_file() {
                let app: App = match load_app(&path).await {
                    Ok(Some(app)) => app,
                    Ok(None) => unreachable!(),
                    Err(err) => {
                        let path_str = path.to_str();
                        log::error!("failed to load file '{path_str:?}'. err={err:?}");
                        continue;
                    }
                };
                apps.push(app)
            }
        }
        Ok(apps)
    }

    pub fn autostart_dir(&self) -> PathBuf {
        self.autostart_dir.clone()
    }

    fn app_path(&self, app_id: &str) -> PathBuf {
        let mut autostart = self.autostart_dir();
        autostart.push(format!("{app_id}.desktop"));
        autostart
    }
}

const INI_SECTION: &str = "Desktop Entry";

async fn store_app(path: impl AsRef<Path>, app: &App) -> anyhow::Result<()> {
    log::debug!(
        "add entry path={:?} id={:?}",
        path.as_ref().to_str().unwrap_or(""),
        app.id
    );
    let mut autostart = Ini::new_cs();
    autostart.set(INI_SECTION, "Type", Some("Application".into()));
    autostart.set(INI_SECTION, "Name", Some(app.id.clone()));
    let command = app.cmd.join(" ");
    autostart.set(INI_SECTION, "Exec", Some(command));
    autostart
        .write_async(path.as_ref())
        .await
        .context("failed to write .desktop file")
}

async fn load_app(path: impl AsRef<Path>) -> anyhow::Result<Option<App>> {
    log::debug!("get entry path={:?}", path.as_ref().to_str().unwrap_or(""));
    let mut autostart = Ini::new_cs();
    if !tokio::fs::try_exists(&path).await.unwrap_or(false) {
        return Ok(None);
    }
    if let Err(err) = autostart.load_async(&path).await {
        return Err(anyhow::anyhow!("{err}")).context("failed to read .desktop file");
    }
    let id = match autostart.get(INI_SECTION, "Name") {
        Some(id) => id,
        None => anyhow::bail!("missing Name"),
    };
    // TODO: Split command properly
    let cmd = match autostart.get(INI_SECTION, "Exec") {
        Some(cmd) => vec![cmd],
        None => anyhow::bail!("missing Exec"),
    };
    Ok(Some(App {
        id,
        cmd,
        autostart: true,
        flags: 0,
    }))
}
