use std::io::ErrorKind;
use std::process::{Child, Command};

use anyhow::Context;
use async_std::path::{Path, PathBuf};
use async_std::stream::StreamExt;
use xdg::BaseDirectories;

use crate::app::App;

/// Used to manage all background applications.
pub struct System {
    base_dir: BaseDirectories,
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

    pub async fn start(&self, app: &App) -> anyhow::Result<Child> {
        let mut process = Command::new(&app.cmd[0]);
        if app.cmd.len() > 1 {
            process.args(&app.cmd[1..]);
        }
        process.spawn().context("failed to spawn process")
    }

    pub async fn list_apps(&self) -> anyhow::Result<Vec<App>> {
        let mut data_dir = self.base_dir.get_data_home();
        data_dir.push("apps");
        let mut apps = Vec::new();
        let mut files = async_std::fs::read_dir(data_dir)
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
                        let path_str = path.to_str().unwrap();
                        log::error!("failed to load file '{path_str}'. err={err:?}");
                        continue;
                    }
                };
                apps.push(app)
            }
        }
        Ok(apps)
    }

    fn app_path(&self, app_id: &str) -> PathBuf {
        self.base_dir
            .get_data_file(format!("apps/{app_id}.toml"))
            .into()
    }
}

impl From<BaseDirectories> for System {
    fn from(dirs: BaseDirectories) -> Self {
        System { base_dir: dirs }
    }
}

async fn store_app(path: impl AsRef<Path>, app: &App) -> anyhow::Result<()> {
    let content = toml::to_string(app).context("failed to generate app file")?;
    async_std::fs::write(path, &content)
        .await
        .context("failed to write app file")
}

async fn load_app(path: impl AsRef<Path>) -> anyhow::Result<Option<App>> {
    let content = match async_std::fs::read_to_string(path).await {
        Ok(content) => content,
        Err(err) if err.kind() == ErrorKind::NotFound => return Ok(None),
        Err(err) => return Err(err).context("failed to read app file")?,
    };
    let app = toml::from_str(&content).context("failed to parse app file")?;
    Ok(Some(app))
}
