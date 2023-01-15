use entity::prelude::Subscriptions;
use migration::{Migrator, MigratorTrait};
use once_cell::sync::Lazy;
use sea_orm::{Database, DatabaseConnection, EntityTrait};
use std::{net::TcpListener, vec};
use zero2prod::{startup, telemetry};

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info";
    let subscriber_name = "zero2prod";
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber =
            telemetry::get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        telemetry::init_subscriber(subscriber);
    } else {
        let subscriber =
            telemetry::get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        telemetry::init_subscriber(subscriber);
    }
});

pub struct TestApp {
    pub address: String,
    pub db_pool: DatabaseConnection,
}

async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let listener = TcpListener::bind("127.0.0.1:0").expect("Listener should bind");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let connection = configure_database().await;

    let server = startup::run(listener, connection.clone()).expect("Run should return server");
    let _ = tokio::spawn(server);
    TestApp {
        address,
        db_pool: connection,
    }
}

async fn configure_database() -> DatabaseConnection {
    let connection = Database::connect("sqlite::memory:")
        .await
        .expect("Connection to DB should be established");
    Migrator::up(&connection, None)
        .await
        .expect("Migration should run");
    connection
}

#[tokio::test]
async fn health_check_works() {
    let address = spawn_app().await.address;

    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/health_check", address))
        .send()
        .await
        .expect("Request should return a response");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let app = spawn_app().await;

    let client = reqwest::Client::new();

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(format!("{}/subscriptions", app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Request should return a response");

    assert_eq!(200, response.status().as_u16());

    let saved = Subscriptions::find()
        .one(&app.db_pool)
        .await
        .expect("Saved subscription should be found")
        .expect("Saved subscription should be present");

    assert_eq!(saved.name, "le guin");
    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let app_address = spawn_app().await.address;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(format!("{}/subscriptions", app_address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Request should return a response");

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}
