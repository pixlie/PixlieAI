use actix_web::{rt, web, App, HttpRequest, HttpServer};
use log::info;

async fn index(req: HttpRequest) -> &'static str {
    println!("REQ: {:?}", req);
    "Hello world!\r\n"
}

pub fn admin_manager() -> std::io::Result<()> {
    info!("Starting Pixlie AI admin, please visit http://localhost:58235");
    rt::System::new().block_on(
        HttpServer::new(|| App::new().service(web::resource("/").route(web::get().to(index))))
            .bind(("localhost", 58235))?
            .workers(1)
            .run(),
    )
}
