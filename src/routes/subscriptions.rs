use actix_web::{post, web, HttpResponse, Responder};
use entity::subscriptions;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, DatabaseConnection};

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

#[post("/subscriptions")]
pub async fn subscribe(
    form: web::Form<FormData>,
    connection: web::Data<DatabaseConnection>,
) -> impl Responder {
    let new_subscription = subscriptions::ActiveModel {
        id: Set(uuid::Uuid::new_v4()),
        email: Set(form.email.to_owned()),
        name: Set(form.name.to_owned()),
        subscribed_at: Set(chrono::Utc::now().into()),
        ..Default::default()
    };
    match new_subscription.insert(connection.get_ref()).await {
        Ok(_) => HttpResponse::Ok(),
        Err(e) => {
            println!("Failed to execute query: {}", e);
            HttpResponse::InternalServerError()
        }
    }
}
