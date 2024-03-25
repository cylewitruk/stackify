
use color_eyre::Result;

pub mod api;
pub mod errors;
pub mod db;
pub mod control;

#[tokio::main]
async fn main() -> Result<()> {

    let db_path = "~/.stackify/stackifyd.db";

    let monitor = 
        control::Monitor::new(db_path)?;
    let channel = monitor.start().await?;

    rocket::build()
        .mount("/api", api::routes())
        .launch()
        .await?;

    Ok(())
}
