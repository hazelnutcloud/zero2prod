use std::net::TcpListener;

use actix_web::{dev::Server, web, App, HttpServer};
use sea_orm::DatabaseConnection;
use tracing_actix_web::TracingLogger;

use crate::routes::{health_check, subscriptions};

pub fn run(
    listener: TcpListener,
    connection: DatabaseConnection,
) -> Result<Server, std::io::Error> {
    let connect = web::Data::new(connection);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .service(health_check::health_check)
            .service(subscriptions::subscribe)
            .app_data(connect.clone())
    })
    .listen(listener)?
    .run();
    Ok(server)
}
