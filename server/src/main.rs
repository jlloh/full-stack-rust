use actix_files as fs;
use actix_web::{
    get,
    web::{self, Data},
    App, HttpServer,
};
use anyhow::Result;
use log::info;

#[actix_web::main]
async fn main() -> Result<()> {
    env_logger::init();
    start_webserver().await?;
    Ok(())
}

pub struct AppState {
    pub dummy: String,
}

pub fn start_webserver() -> actix_web::dev::Server {
    info!("Starting local callback webserver");

    let server = HttpServer::new(move || {
        let app_state = Data::new(AppState {
            dummy: "abcd".to_string(),
        });
        App::new()
            .app_data(app_state)
            .service(hello)
            .service(fs::Files::new("/", "./dist").index_file("index.html"))
        // .default_service(web::get().to(index))
    });

    server.bind(("localhost", 8080)).unwrap().run()
}

#[get("/hello")]
async fn hello(_app_state: web::Data<AppState>) -> String {
    "hello there".to_string()
}

// async fn index(req: HttpRequest) -> ActixResult<fs::NamedFile> {
//     Ok(fs::NamedFile::open("./dist/index.html")?)
// }
