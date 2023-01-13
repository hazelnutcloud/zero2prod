use sea_orm::Database;
use std::net::TcpListener;
use zero2prod::{config::get_configuration, startup};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let config = get_configuration().expect("Config should load");
    let connection = Database::connect(&config.database.connection_string())
        .await
        .expect("Connection to DB should be established");
    let address = format!("127.0.0.1:{}", config.application_port);
    let listener = TcpListener::bind(address)?;
    startup::run(listener, connection)?.await
}
