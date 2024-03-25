pub mod api;
pub mod models;
pub mod errors;

fn main() {
    rocket::build()
        .mount("/api", api::routes())
        .launch();
}
