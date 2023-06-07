use actix_web::{web, HttpResponse};
use chrono::Utc;
use crate::domain::SubscriberEmail;
use crate::domain::SubscriberName;
use crate::domain::NewSubscriber;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

impl TryFrom<FormData> for NewSubscriber {
    type Error = String;

    fn try_from(value: FormData) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(value.name)?;
        let email = SubscriberEmail::parse(value.email)?;

        Ok(Self { email, name })
    }
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database.",
    skip(form, pool),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
pub async fn subscribe(form: web::Form<FormData>, pool: web::Data<PgPool>) -> HttpResponse { 
    let new_subscriber = match form.0.try_into() {
        Ok(sub) => sub,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };

    match insert_subscriber(&pool, &new_subscriber).await
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            tracing::error!("Failed to execute query. {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[tracing::instrument(
    skip(new_subscriber, pool)
)]
async fn insert_subscriber(
    pool: &PgPool,
    new_subscriber: &NewSubscriber,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        Utc::now()
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(())
}
