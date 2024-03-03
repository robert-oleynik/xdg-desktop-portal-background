use anyhow::Context;
use async_std::path::Path;
use clap::Parser;
use log::LevelFilter;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Root};
use log4rs::Config;
use xdg::BaseDirectories;

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

#[async_std::main]
async fn main() -> anyhow::Result<()> {
    println!("started");
    let base_dir = BaseDirectories::with_prefix("xdg-desktop-portal-background")
        .context("failed to receive base directory")?;
    let cache_path = base_dir.get_data_home();
    async_std::fs::create_dir_all(cache_path)
        .await
        .context("failed to create cache directories")?;
    let cache_path = base_dir.get_cache_file("background.log");
    init_logger(cache_path)?;

    let mut background_dir = base_dir.get_data_home();
    background_dir.push("apps");
    async_std::fs::create_dir_all(background_dir)
        .await
        .context("failed to create background directory")?;

    let _args = Args::parse();

    let conn = zbus::Connection::session()
        .await
        .context("failed to create zbus session")?;
    let mut bg = Background::from(base_dir);
    bg.startup().await.context("startup failed")?;
    conn.object_server()
        .at("/org/freedesktop/portal/desktop", bg)
        .await
        .context("faield to create background server")?;
    conn.request_name("org.freedesktop.impl.portal.desktop.background")
        .await
        .context("failed to request service name")?;

    log::info!("services started");

    async_std::future::pending::<()>().await;

    Ok(())
}
