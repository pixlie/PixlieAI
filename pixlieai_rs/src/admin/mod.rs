use crate::{config::get_path_to_static_dir, error::PiResult};
use actix_files as fs;
use actix_web::{rt, App, HttpServer};
use log::info;

pub fn admin_manager() -> PiResult<()> {
    info!("Starting Pixlie AI admin, please visit http://localhost:58235");
    let path_to_static_dir = get_path_to_static_dir()?;
    rt::System::new().block_on(
        HttpServer::new(move || {
            App::new()
                .service(fs::Files::new("/", path_to_static_dir.clone()).index_file("index.html"))
        })
        .bind(("localhost", 58235))?
        .workers(1)
        .run(),
    )?;
    Ok(())
}
