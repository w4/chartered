use crate::endpoints::ErrorResponse;

use axum::{
    body::{boxed, Body, BoxBody},
    extract::{self, FromRequest, RequestParts},
    http::{Request, StatusCode},
    response::Response,
};
use futures::future::BoxFuture;
use governor::{clock::DefaultClock, state::keyed::DefaultKeyedStateStore, Quota, RateLimiter};
use tower::{Layer, Service};

use std::{
    net::IpAddr,
    num::NonZeroU32,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    task::{Context, Poll},
};

pub struct RateLimit {
    governor: Arc<RateLimiter<IpAddr, DefaultKeyedStateStore<IpAddr>, DefaultClock>>,
    counter: Arc<AtomicUsize>,
}

impl RateLimit {
    pub fn new(quota: Quota) -> Self {
        Self {
            governor: Arc::new(RateLimiter::keyed(quota)),
            counter: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn with_cost(&self, cost: u32) -> RateLimitLayer {
        RateLimitLayer {
            governor: self.governor.clone(),
            counter: self.counter.clone(),
            cost: NonZeroU32::new(cost).unwrap(),
        }
    }
}

pub struct RateLimitLayer {
    governor: Arc<RateLimiter<IpAddr, DefaultKeyedStateStore<IpAddr>, DefaultClock>>,
    counter: Arc<AtomicUsize>,
    cost: NonZeroU32,
}

impl<S> Layer<S> for RateLimitLayer {
    type Service = RateLimitMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RateLimitMiddleware {
            inner,
            governor: self.governor.clone(),
            counter: self.counter.clone(),
            cost: self.cost,
        }
    }
}

#[derive(Clone)]
pub struct RateLimitMiddleware<S> {
    inner: S,
    governor: Arc<RateLimiter<IpAddr, DefaultKeyedStateStore<IpAddr>, DefaultClock>>,
    counter: Arc<AtomicUsize>,
    cost: NonZeroU32,
}

impl<S, ReqBody> Service<Request<ReqBody>> for RateLimitMiddleware<S>
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
            let ip_addr = extract::Extension::<IpAddr>::from_request(&mut req)
                .await
                .map(|v| v.0);

            if let Ok(addr) = ip_addr {
                if let Err(_e) = this.governor.check_key_n(&addr, this.cost) {
                    return Ok(Response::builder()
                        .status(StatusCode::TOO_MANY_REQUESTS)
                        .body(boxed(Body::from(
                            serde_json::to_vec(&ErrorResponse {
                                error: Some(
                                    "You are being rate limited. Please wait a bit and try again."
                                        .into(),
                                ),
                            })
                            .unwrap(),
                        )))
                        .unwrap());
                }

                // every 500 requests, clear out keys that haven't been used in a while
                if this.counter.fetch_add(1, Ordering::AcqRel) % 500 == 0 {
                    this.governor.retain_recent();
                }
            }

            this.inner.call(req.try_into_request().unwrap()).await
        })
    }
}
