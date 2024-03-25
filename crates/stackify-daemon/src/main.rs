use color_eyre::Result;

pub mod api;
pub mod control;
pub mod db;
pub mod errors;

#[tokio::main]
async fn main() -> Result<()> {
    let db_path = "~/.stackify/stackifyd.db";

    let _channel = control::Monitor::new(db_path).start().await?;

    rocket::build()
        .mount("/api", api::routes())
        .launch()
        .await?;

    Ok(())
}
