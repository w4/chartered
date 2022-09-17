//! Logs each and every request out in a format similar to that of Apache's logs.

use axum::{
    extract::{self, FromRequest, RequestParts},
    http::{Request, Response},
};
use futures::future::BoxFuture;
use once_cell::sync::Lazy;
use regex::Regex;
use tower::Service;
use tracing::{error, info, Instrument};

use std::{
    fmt::Debug,
    net::IpAddr,
    task::{Context, Poll},
};

pub trait GenericError: std::error::Error + Debug + Send + Sync {}

#[derive(Clone)]
pub struct LoggingMiddleware<S>(pub S);

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for LoggingMiddleware<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>, Error = std::convert::Infallible>
        + Clone
        + Send
        + 'static,
    S::Future: Send + 'static,
    S::Response: Default + Debug,
    ReqBody: Send + Debug + 'static,
    ResBody: Default + Send + 'static,
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

        let request_id = chartered_db::uuid::Uuid::new_v4();
        let span = tracing::info_span!("web", "request_id" = request_id.to_string().as_str());

        Box::pin(async move {
            let start = std::time::Instant::now();
            let user_agent = req.headers().get(axum::http::header::USER_AGENT).cloned();
            let method = req.method().clone();
            let uri = replace_sensitive_path(req.uri().path());

            let mut req = RequestParts::new(req);
            let ip_addr = extract::Extension::<IpAddr>::from_request(&mut req)
                .await
                .map_or_else(|_| "0.0.0.0".parse().unwrap(), |v| v.0);

            // this is infallible because of the type of S::Error
            let response = inner.call(req.try_into_request().unwrap()).await?;

            if response.status().is_server_error() {
                error!(
                    "{ip} - \"{method} {uri}\" {status} {duration:?} \"{user_agent}\" \"{error:?}\"",
                    ip = ip_addr,
                    method = method,
                    uri = uri,
                    status = response.status().as_u16(),
                    duration = start.elapsed(),
                    user_agent = user_agent
                        .as_ref()
                        .and_then(|v| v.to_str().ok())
                        .unwrap_or("unknown"),
                    error = match response.extensions().get::<Box<dyn GenericError>>() {
                        Some(e) => Err(e),
                        None => Ok(()),
                    }
                );
            } else {
                info!(
                    "{ip} - \"{method} {uri}\" {status} {duration:?} \"{user_agent}\" \"{error:?}\"",
                    ip = ip_addr,
                    method = method,
                    uri = uri,
                    status = response.status().as_u16(),
                    duration = start.elapsed(),
                    user_agent = user_agent
                        .as_ref()
                        .and_then(|v| v.to_str().ok())
                        .unwrap_or("unknown"),
                    error = match response.extensions().get::<Box<dyn GenericError>>() {
                        Some(e) => Err(e),
                        None => Ok(()),
                    }
                );
            }

            Ok(response)
        }.instrument(span))
    }
}

fn replace_sensitive_path(uri: &str) -> String {
    static SENSITIVE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^/a/(.*?)/").unwrap());
    SENSITIVE_REGEX.replace(uri, "/a/[snip]/").into_owned()
}
