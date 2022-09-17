//! Adds the user's IP address to the request, taking into account the `trusted_ip_header` config
//! value.

use axum::{
    body::BoxBody,
    extract::{self, FromRequest, RequestParts},
    http::{Request, Response},
};
use futures::future::BoxFuture;
use tower::{Layer, Service};

use std::{
    net::IpAddr,
    str::FromStr,
    sync::Arc,
    task::{Context, Poll},
};

#[derive(Clone)]
pub struct AddIp {
    trusted_ip_header: Option<Arc<str>>,
}

impl AddIp {
    pub fn new(trusted_ip_header: Option<String>) -> Self {
        Self {
            trusted_ip_header: trusted_ip_header.map(Arc::from),
        }
    }
}

impl<S> Layer<S> for AddIp {
    type Service = AddIpService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        AddIpService {
            inner,
            trusted_ip_header: self.trusted_ip_header.clone(),
        }
    }
}

#[derive(Clone)]
pub struct AddIpService<S> {
    inner: S,
    trusted_ip_header: Option<Arc<str>>,
}

impl<S, ReqBody> Service<Request<ReqBody>> for AddIpService<S>
where
    S: Service<Request<ReqBody>, Response = Response<BoxBody>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    ReqBody: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        // ensure we take the instance that has already been poll_ready'd
        let clone = self.clone();
        let mut this = std::mem::replace(self, clone);

        Box::pin(async move {
            let mut req = RequestParts::new(req);

            let mut ip = None;

            if let Some(trusted_ip_header) = this.trusted_ip_header.as_deref() {
                ip = req
                    .headers()
                    .get(trusted_ip_header)
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| IpAddr::from_str(v).ok());
            }

            // no trusted ip header, fallback to the socket address
            if ip.is_none() {
                ip = extract::ConnectInfo::<std::net::SocketAddr>::from_request(&mut req)
                    .await
                    .map(|v| v.0.ip())
                    .ok();
            }

            req.extensions_mut().insert(ip.unwrap());

            this.inner.call(req.try_into_request().unwrap()).await
        })
    }
}
