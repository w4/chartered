//! Check the API key in the authorization header is valid otherwise returns a 401 for authenticated
//! endpoints.

use axum::{
    body::{boxed, Body, BoxBody},
    extract::{self, FromRequest, RequestParts},
    http::{Request, Response, StatusCode},
    response::IntoResponse,
    TypedHeader,
};
use chartered_db::{users::User, ConnectionPool};
use futures::future::BoxFuture;
use headers::{authorization::Bearer, Authorization};
use std::{
    sync::Arc,
    task::{Context, Poll},
};
use tower::Service;

use crate::endpoints::ErrorResponse;

#[derive(Clone)]
pub struct WebAuthMiddleware<S>(pub S);

impl<S, ReqBody> Service<Request<ReqBody>> for WebAuthMiddleware<S>
where
    S: Service<Request<ReqBody>, Response = Response<BoxBody>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    ReqBody: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.0.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        // best practice is to clone the inner service like this
        // see https://github.com/tower-rs/tower/issues/547 for details
        let clone = self.0.clone();
        let mut inner = std::mem::replace(&mut self.0, clone);

        Box::pin(async move {
            let mut req = RequestParts::new(req);

            // extract the authorization header
            let authorization: Authorization<Bearer> =
                match extract::TypedHeader::from_request(&mut req).await {
                    Ok(TypedHeader(v)) => v,
                    Err(e) => return Ok(e.into_response()),
                };

            // grab the ConnectionPool from the extensions created when we initialised the
            // server
            let db = req.extensions().get::<ConnectionPool>().unwrap().clone();

            // grab the UserSession that's currently being used for this request and the User that
            // owns the key, otherwise return a 401 if the key doesn't exist
            let (session, user) =
                match User::find_by_session_key(db, String::from(authorization.0.token()))
                    .await
                    .unwrap()
                {
                    Some((session, user)) => (Arc::new(session), Arc::new(user)),
                    None => {
                        return Ok(Response::builder()
                            .status(StatusCode::UNAUTHORIZED)
                            .body(boxed(Body::from(
                                serde_json::to_vec(&ErrorResponse {
                                    error: Some("Expired auth token".into()),
                                })
                                .unwrap(),
                            )))
                            .unwrap())
                    }
                };

            if session.user_ssh_key_id.is_some() {
                // SSH sessions can't be used for the web API
                return Ok(Response::builder()
                    .status(StatusCode::UNAUTHORIZED)
                    .body(boxed(Body::from(
                        serde_json::to_vec(&ErrorResponse {
                            error: Some("Invalid auth token".into()),
                        })
                        .unwrap(),
                    )))
                    .unwrap());
            }

            // insert both the user and the session into extensions so handlers can
            // get their hands on them
            req.extensions_mut().insert(user);
            req.extensions_mut().insert(session);

            // calls handlers/other middleware and drives the request to response
            let response: Response<BoxBody> = inner.call(req.try_into_request().unwrap()).await?;

            Ok(response)
        })
    }
}
