use axum::{
    extract::{FromRequest, Request},
    response::{IntoResponse, Response},
    Json,
};
use async_trait::async_trait;
use serde::de::DeserializeOwned;
use validator::Validate;
use crate::errors::AppError;

pub struct ValidatedJson<T>(pub T);

impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = Response;

    fn from_request(req: Request, state: &S) -> impl core::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        async move {
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(IntoResponse::into_response)?;

        value.validate().map_err(|errors| {
            let error_message = errors
                .field_errors()
                .iter()
                .flat_map(|(field, errors)| {
                    errors.iter().map(move |error| {
                        format!(
                            "{}: {}",
                            field,
                            error.message.as_ref().unwrap_or(&"Invalid value".into())
                        )
                    })
                })
                .collect::<Vec<String>>()
                .join(", ");

            AppError::Validation(error_message).into_response()
        })?;

        Ok(ValidatedJson(value))
        }
    }
}
