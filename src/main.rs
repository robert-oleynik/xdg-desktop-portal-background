use std::path::Path;

use anyhow::Context;
use clap::Parser;
use log::LevelFilter;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Root};
use log4rs::Config;

use crate::background::Background;

mod background;
mod system;

#[derive(Parser)]
pub struct Args {}

pub fn init_logger(logfile: impl AsRef<Path>) -> anyhow::Result<()> {
    let logfile = FileAppender::builder()
        .build(logfile.as_ref())
        .context("failed to create file appender logger")?;
    let appender = Appender::builder().build("logfile", Box::new(logfile));
    let root = Root::builder()
        .appender("logfile")
        .build(LevelFilter::Debug);
    let config = Config::builder()
        .appender(appender)
        .build(root)
        .context("failed to generate logger config")?;
    log4rs::init_config(config).context("failed to initialize logger")?;
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let home = std::env::var("HOME").unwrap();
    println!("started");
    tokio::fs::create_dir_all(format!("{home}/.config/autostart"))
        .await
        .context("faield to create autostart dir")?;
    let cache_path = format!("{home}/.cache/xdg-desktop-portal-background");
    tokio::fs::create_dir_all(&cache_path)
        .await
        .context("failed to create cache directories")?;
    let cache_path = format!("{cache_path}/background.log");
    init_logger(cache_path)?;

    let _args = Args::parse();

    let conn = zbus::Connection::session()
        .await
        .context("failed to create zbus session")?;
    let bg = Background::default();
    conn.object_server()
        .at("/org/freedesktop/portal/desktop", bg)
        .await
        .context("faield to create background server")?;
    conn.request_name("org.freedesktop.impl.portal.desktop.background")
        .await
        .context("failed to request service name")?;

    log::info!("services started");

    std::future::pending().await
}
