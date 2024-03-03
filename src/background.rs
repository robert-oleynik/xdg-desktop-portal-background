//! Implementation of org.freedesktop.impl.portal.Background:
//!
//! See: https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.impl.portal.Background.html

use std::collections::HashMap;

use xdg::BaseDirectories;
use zbus::object_server::SignalContext;

use crate::system::{App, System};

pub struct Background {
    system: System,
}

impl From<BaseDirectories> for Background {
    fn from(value: BaseDirectories) -> Self {
        Self {
            system: System::from(value),
        }
    }
}

#[zbus::interface(name = "org.freedesktop.impl.portal.Background")]
impl Background {
    async fn get_app_state(&self) -> zbus::fdo::Result<Vec<(String, u32)>> {
        log::debug!("requested app state");
        let apps = match self.system.list_apps().await {
            Ok(apps) => apps,
            Err(err) => {
                log::error!("{err}");
                return Err(zbus::fdo::Error::Failed("failed to list apps".into()));
            }
        };
        let result = apps.into_iter().map(|app| (app.id, 0)).collect();
        Ok(result)
    }

    async fn notify_background(
        &self,
        handle: zbus::zvariant::ObjectPath<'_>,
        app_id: String,
        name: String,
    ) -> (u32, HashMap<String, u32>) {
        log::debug!("handle={handle:?} app_id={app_id:?} name={name:?}");
        // TODO: Send Notification to request autostart.
        let mut result = HashMap::new();
        result.insert("result".into(), 2);
        (0, result)
    }

    async fn enable_autostart(
        &mut self,
        app_id: String,
        enable: bool,
        cmd: Vec<String>,
        flags: u32,
    ) -> zbus::fdo::Result<bool> {
        log::debug!("app_id={app_id:?} enable={enable:?} cmd={cmd:?} flags={flags:?}");
        match self.system.get_autostart(&app_id).await {
            Ok(Some(_)) => Ok(true),
            Ok(None) => {
                let app = App {
                    id: app_id,
                    autostart: enable,
                    cmd,
                    flags,
                };
                if let Err(err) = self.system.add_autostart(&app).await {
                    log::error!("{err:?}");
                }
                Ok(true)
            }
            Err(err) => {
                log::error!("{err:?}");
                Err(zbus::fdo::Error::Failed("failed to register app".into()))
            }
        }
    }

    #[zbus(signal)]
    async fn running_applications_changed(ctx: &SignalContext<'_>) -> zbus::Result<()>;
}
