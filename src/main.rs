use sea_orm::Database;
use secrecy::ExposeSecret;
use std::net::TcpListener;
use zero2prod::{config::get_configuration, startup, telemetry};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = telemetry::get_subscriber("zero2prod", "info", std::io::stdout);
    telemetry::init_subscriber(subscriber);

    let config = get_configuration().expect("Config should load");
    let connection = Database::connect(config.database.connection_string().expose_secret())
        .await
        .expect("Connection to DB should be established");
    let address = format!("{}:{}", config.application.host, config.application.port);
    let listener = TcpListener::bind(address)?;

    startup::run(listener, connection)?.await
}
