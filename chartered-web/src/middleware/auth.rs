//! Check the API key embedded in the path is valid otherwise returns a 401 for authenticated
//! endpoints.

use axum::{
    body::{boxed, Body, BoxBody},
    extract::{self, FromRequest, RequestParts},
    http::{Request, Response, StatusCode},
};
use chartered_db::users::User;
use chartered_db::ConnectionPool;
use futures::future::BoxFuture;
use std::sync::Arc;
use std::{
    collections::HashMap,
    task::{Context, Poll},
};
use tower::Service;

use crate::endpoints::ErrorResponse;

#[derive(Clone)]
pub struct AuthMiddleware<S>(pub S);

impl<S, ReqBody> Service<Request<ReqBody>> for AuthMiddleware<S>
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

            // extracts all parameters from the path so we can get the API key which should
            // always be named key
            let params = extract::Path::<HashMap<String, String>>::from_request(&mut req)
                .await
                .unwrap();
            let key = params.get("key").map(String::as_str).unwrap_or_default();

            // grab the ConnectionPool from the extensions created when we initialised the
            // server
            let db = req.extensions().get::<ConnectionPool>().unwrap().clone();

            // grab the UserSession that's currently being used for this request and the User that
            // owns the key, otherwise return a 401 if the key doesn't exist
            let (session, user) = match User::find_by_session_key(db, String::from(key))
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
