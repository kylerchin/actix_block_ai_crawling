//! This crate blocks generative AI from accessing your services
//! 
//! It is a middleware which blocks user agents, and maybe in the future, IP ranges as well
//! 
//! It's extremely simple to use. Just add `.wrap(actix_block_ai_crawling::BlockAi);` to your app.
//! 
//! ```
//! let app = App::new()
//! .wrap(actix_block_ai_crawling::BlockAi);
//! ```

//this code was written by Kyler Chin. Not by a machine learning model.
use std::future::{ready, Ready};

use actix_web::{
    body::EitherBody,
    dev::{self, Service, ServiceRequest, ServiceResponse, Transform},
    http, Error, HttpResponse,
};

use futures_util::future::LocalBoxFuture;

pub struct BlockAi;

impl<S, B> Transform<S, ServiceRequest> for BlockAi
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = BlockAiMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(BlockAiMiddleware { service }))
    }
}
pub struct BlockAiMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for BlockAiMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    dev::forward_ready!(service);

    fn call(&self, request: ServiceRequest) -> Self::Future {
        let blocked_user_agents = ["ChatGPT-User", "GPTBot", "CCBot", "Google-Extended"];

        let user_agent: Option<&str> = match request
            .request()
            .headers()
            .get(actix_web::http::header::USER_AGENT)
        {
            Some(ua) => Some(ua.to_str().unwrap()),
            None => None,
        }
        .clone();

        if user_agent.is_some() {
            let user_agent = user_agent.unwrap();

            if blocked_user_agents.contains(&user_agent) {
                let (request, _pl) = request.into_parts();

                let response = HttpResponse::Forbidden()
                    .finish()
                    // constructed responses map to "right" body
                    .map_into_right_body();

                return Box::pin(async { Ok(ServiceResponse::new(request, response)) });
            }
        }

        let res = self.service.call(request);

        Box::pin(async move {
            // forwarded responses map to "left" body
            res.await.map(ServiceResponse::map_into_left_body)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::web;
    use actix_web::App;

    #[test]
    fn valid_middleware() {
        let app = App::new()
            .wrap(BlockAi)
            .route("/", web::get().to(|| async { "Hello, middleware!" }));
    }
}
