use std::net::SocketAddr;

use actix_web::{get, App, HttpResponse, HttpServer, Responder};
use actix_web_prom::PrometheusMetricsBuilder;
use anyhow::{anyhow, Result};
use ark_core::{env::infer, logger};

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().json("footprint")
}

#[get("/health")]
async fn health() -> impl Responder {
    HttpResponse::Ok().json("healthy")
}

#[cfg(feature = "put")]
#[::actix_web::put("/")]
async fn put(
    ::actix_web::web::Json(location): ::actix_web::web::Json<::footprint_api::Location>,
) -> impl Responder {
    ::footprint_provider_api::update(location);
    HttpResponse::Ok().finish()
}

#[actix_web::main]
async fn main() {
    async fn try_main() -> Result<()> {
        // Initialize kubernetes client
        let addr =
            infer::<_, SocketAddr>("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:80".parse().unwrap());

        let prometheus = PrometheusMetricsBuilder::new("ulagbulag_footprint")
            .endpoint("/metrics")
            .build()
            .map_err(|e| anyhow!("{e}"))?;

        ::footprint_provider_api::register(&prometheus.registry)?;

        #[cfg(feature = "dummy")]
        if infer::<_, bool>("FOOTPRINT_PROVIDER_DUMMY").unwrap_or_default() {
            ::footprint_provider_dummy::spawn()?
        }

        // Start web server
        HttpServer::new(move || {
            let app = App::new()
                .wrap(prometheus.clone())
                .service(index)
                .service(health);

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
