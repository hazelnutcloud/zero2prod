use std::{net::TcpListener, vec};

use entity::prelude::Subscriptions;
use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbBackend, EntityTrait, Statement};
use zero2prod::{
    config::{get_configuration, DatabaseSettings},
    startup,
};

pub struct TestApp {
    pub address: String,
    pub db_pool: DatabaseConnection,
}

async fn spawn_app() -> TestApp {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Listener should bind");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let mut config = get_configuration().expect("Config should load");
    config.database.database_name = uuid::Uuid::new_v4().to_string();
    let connection = configure_database(&config.database).await;

    let server = startup::run(listener, connection.clone()).expect("Run should return server");
    let _ = tokio::spawn(server);
    TestApp {
        address,
        db_pool: connection,
    }
}

async fn configure_database(config: &DatabaseSettings) -> DatabaseConnection {
    let connection = Database::connect(&config.connection_string_without_db())
        .await
        .expect("Connection to DB should be established");
    connection
        .execute(Statement::from_string(
            DbBackend::Postgres,
            format!("CREATE DATABASE \"{}\"", config.database_name),
        ))
        .await
        .expect("Database should be created");
    println!("database name: {}", config.database_name);
    println!("connection string: {}", config.connection_string());
    let new_connection = Database::connect(&config.connection_string())
        .await
        .expect("Connection to DB should be established");
    Migrator::up(&new_connection, None)
        .await
        .expect("Migration should run");
    new_connection
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
