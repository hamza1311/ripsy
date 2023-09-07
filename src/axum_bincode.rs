use axum::body::{Bytes, HttpBody};
use axum::extract::rejection::BytesRejection;
use axum::extract::FromRequest;
use axum::http::{header, HeaderValue, Request, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::{async_trait, BoxError};
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::Bincode;

pub enum BincodeRejection {
    NotBincodeContentType,
    BytesRejection(BytesRejection),
    BincodeSerdeError(bincode::Error),
}

impl IntoResponse for BincodeRejection {
    fn into_response(self) -> Response {
        match self {
            BincodeRejection::NotBincodeContentType => (
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                "Expected request with `Content-Type: application/bincode`",
            )
                .into_response(),
            BincodeRejection::BincodeSerdeError(err) => (
                StatusCode::BAD_REQUEST,
                format!("failed to parse bincode: {err}"),
            )
                .into_response(),
            BincodeRejection::BytesRejection(r) => r.into_response(),
        }
    }
}

impl<T: Serialize, E: Serialize> IntoResponse for Bincode<Result<T, E>> {
    fn into_response(self) -> Response {
        let status = match &self.0 {
            Ok(_) => StatusCode::OK,
            Err(_) => StatusCode::IM_A_TEAPOT,
        };

        let body = self.serialize().unwrap();
        let mut resp = (status, body).into_response();

        resp.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/bincode"),
        );
        resp
    }
}

#[async_trait]
impl<S, B, T> FromRequest<S, B> for Bincode<T>
where
    T: DeserializeOwned,
    B: HttpBody + Send + 'static,
    B::Data: Send,
    B::Error: Into<BoxError>,
    S: Send + Sync,
{
    type Rejection = BincodeRejection;

    async fn from_request(req: Request<B>, state: &S) -> Result<Self, Self::Rejection> {
        let content_type = req.headers().get(header::CONTENT_TYPE);
        if content_type != Some(&HeaderValue::from_static("application/bincode")) {
            return Err(BincodeRejection::NotBincodeContentType);
        }

        let bytes = Bytes::from_request(req, state)
            .await
            .map_err(BincodeRejection::BytesRejection)?;
        let value =
            bincode::deserialize::<T>(&bytes).map_err(BincodeRejection::BincodeSerdeError)?;

        Ok(Self(value))
    }
}
