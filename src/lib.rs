use std::{future::{ready, Ready, Future}, pin::Pin};

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    web, Error,
    App
};
use actix_web::HttpResponse;


pub struct BlockOpenai;

// `S` - type of the next service
// `B` - type of response's body
impl<S, B> Transform<S, ServiceRequest> for BlockOpenai
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = BlockOpenaiMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(BlockOpenaiMiddleware { service }))
    }
}

pub struct BlockOpenaiMiddleware<S> {
    /// The next service to call
    service: S,
}

// This future doesn't have the requirement of being `Send`.
// See: futures_util::future::LocalBoxFuture
type LocalBoxFuture<T> = Pin<Box<dyn Future<Output = T> + 'static>>;

// `S`: type of the wrapped service
// `B`: type of the body - try to be generic over the body where possible
impl<S, B> Service<ServiceRequest> for BlockOpenaiMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<Result<Self::Response, Self::Error>>;

    // This service is ready when its next service is ready
    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        println!("Hi from start. You requested: {}", req.path());

        // A more complex middleware, could return an error or an early response here.
        let user_agent: Option<&str> = match req
        .request()
        .headers()
        .get(actix_web::http::header::USER_AGENT) {
            Some(ua) => Some(ua.to_str().unwrap()),
            None => None,
        }.clone();

        //get ip address
        let ip_address = req.peer_addr();

        if user_agent == Some("GPTBot") {
            println!("GPTBot detected. Blocking request.");
            return Box::pin(async move {
                Ok(req.into_response(HttpResponse::Forbidden().finish().into_body()))
            });
        }
 
        let fut = self.service.call(req);

        Box::pin(async move {

            let res = fut.await?;

            println!("Hi from response");
            Ok(res)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn valid_middleware() {
        
let app = App::new()
.wrap(BlockOpenai)
.route("/", web::get().to(|| async { "Hello, middleware!" }));
    }
}