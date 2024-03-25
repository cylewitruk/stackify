use color_eyre::Result;

pub mod api;
pub mod errors;
pub mod db;

#[rocket::main]
async fn main() -> Result<()> {
    rocket::build()
        .mount("/api", api::routes())
        .launch().await?;

    Ok(())
}
