use actix_web::{post, web, HttpResponse, Responder};
use entity::subscriptions;
use migration::DbErr;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, DatabaseConnection};

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

#[tracing::instrument(
    name = "Adding a new subscription",
    skip(form, connection),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
#[post("/subscriptions")]
pub async fn subscribe(
    form: web::Form<FormData>,
    connection: web::Data<DatabaseConnection>,
) -> impl Responder {
    match insert_subscriber(&form, connection.get_ref()).await {
        Ok(_) => HttpResponse::Ok(),
        Err(_) => HttpResponse::InternalServerError(),
    }
}

#[tracing::instrument(
    name = "Saving new subscription in the database",
    skip(connection, form)
)]
pub async fn insert_subscriber(
    form: &FormData,
    connection: &DatabaseConnection,
) -> Result<(), DbErr> {
    let new_subscription = subscriptions::ActiveModel {
        id: Set(uuid::Uuid::new_v4()),
        email: Set(form.email.to_owned()),
        name: Set(form.name.to_owned()),
        subscribed_at: Set(chrono::Utc::now().into()),
    };
    match new_subscription.insert(connection).await {
        Ok(_) => Ok(()),
        Err(e) => {
            tracing::error!("Failed to save new subscription in the database: {:?}", e);
            Err(e)
        }
    }
}
