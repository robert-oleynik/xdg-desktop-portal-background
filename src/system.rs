use std::str::FromStr;

use anyhow::Context;
use async_std::path::{Path, PathBuf};
use async_std::stream::StreamExt;
use configparser::ini::{Ini, WriteOptions};

/// Used to manage all background applications.
#[derive(Default)]
pub struct System;

pub struct App {
    pub id: String,
    pub autostart: bool,
    pub cmd: Vec<String>,
    pub flags: u32,
}

impl System {
    pub async fn add_autostart(&mut self, app: &App) -> anyhow::Result<()> {
        let path = self.app_path(&app.id);
        if path.exists().await {
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
        let mut files = async_std::fs::read_dir(app_dir)
            .await
            .context("failed to read apps")?;
        loop {
            let entry = match files.next().await {
                Some(Ok(entry)) => entry,
                Some(Err(err)) => {
                    log::error!("failed to fetch app entry: {err:?}");
                    continue;
                }
                None => break,
            };
            let path = entry.path();
            if path.is_file().await {
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
        PathBuf::from_str("~/.config/autostart").unwrap()
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
    let mut autostart = Ini::new();
    autostart.set(INI_SECTION, "Type", Some("Application".into()));
    autostart.set(INI_SECTION, "Name", Some(app.id.clone()));
    let command = app.cmd.join(" ");
    autostart.set(INI_SECTION, "Exec", Some(command));
    let opts = WriteOptions::new_with_params(true, 0, 1);
    autostart
        .pretty_write_async(path.as_ref(), &opts)
        .await
        .context("failed to write .desktop file")
}

async fn load_app(path: impl AsRef<Path>) -> anyhow::Result<Option<App>> {
    log::debug!("get entry path={:?}", path.as_ref().to_str().unwrap_or(""));
    let mut autostart = Ini::new();
    if let Err(err) = autostart.load_async(path.as_ref()).await {
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
