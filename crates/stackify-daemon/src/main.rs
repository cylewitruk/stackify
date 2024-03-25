use std::sync::Arc;

use color_eyre::Result;
use db::DaemonDb;
use diesel::{Connection, SqliteConnection};

pub mod api;
pub mod errors;
pub mod db;
pub mod control;

#[tokio::main]
async fn main() -> Result<()> {

    let monitor = 
        control::Monitor::new("~/.stackify/stackifyd.db")?;
    let (monitor, channel) = monitor.start()?;

    rocket::build()
        .mount("/api", api::routes())
        .launch()
        .await?;

    Ok(())
}
