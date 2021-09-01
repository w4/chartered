use axum::http::{Request, Response, StatusCode};
use futures::future::BoxFuture;
use std::task::{Context, Poll};
use tower::Service;

#[derive(Clone)]
pub struct AuthMiddleware<S>(pub S);

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for AuthMiddleware<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    ReqBody: Send + 'static,
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

        Box::pin(async move {
            // if true {
            //     return Ok(Response::builder()
            //         .status(StatusCode::UNAUTHORIZED)
            //         .body(ResBody::default())
            //         .unwrap());
            // }

            let res: Response<ResBody> = inner.call(req).await?;

            Ok(res)
        })
    }
}
