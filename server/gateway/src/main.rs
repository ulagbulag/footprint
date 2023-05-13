use std::net::SocketAddr;

use actix_cors::Cors;
use actix_web::{
    get,
    web::{Data, Query},
    App, HttpResponse, HttpServer, Responder,
};
use anyhow::Result;
use ark_core::{env::infer, logger};
use footprint_api::DataRef;
use footprint_client::Client;

#[get("/")]
async fn get_metric(client: Data<Client>, Query(query): Query<DataRef>) -> impl Responder {
    match client.get_raw(&query).await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(e) => HttpResponse::Forbidden().json(e.to_string()),
    }
}

#[get("/health")]
async fn health() -> impl Responder {
    HttpResponse::Ok().json("healthy")
}

#[actix_web::main]
async fn main() {
    async fn try_main() -> Result<()> {
        // Initialize kubernetes client
        let addr =
            infer::<_, SocketAddr>("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:80".parse().unwrap());
        let client = Data::new(Client::try_default()?);

        // Start web server
        HttpServer::new(move || {
            let cors = Cors::default()
                .allow_any_header()
                .allow_any_method()
                .allow_any_origin();

            let app = App::new()
                .app_data(Data::clone(&client))
                .service(get_metric)
                .service(health)
                .wrap(cors);

            #[cfg(feature = "put")]
            {
                app.service(put)
            }
            #[cfg(not(feature = "put"))]
            {
                app
            }
        })
        .bind(addr)
        .unwrap_or_else(|e| panic!("failed to bind to {addr}: {e}"))
        .run()
        .await
        .map_err(Into::into)
    }

    logger::init_once();
    try_main().await.expect("running a server")
}
