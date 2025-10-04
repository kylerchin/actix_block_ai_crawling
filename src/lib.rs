//! This crate blocks generative AI from accessing your services
//!
//! It is a middleware which blocks user agents from Bard, GPT-3, and other generative AI from accessing your services.
//! It also blocks OpenAI's crawler IP addresses.
//!
//! It's extremely simple to use. Just add `.wrap(actix_block_ai_crawling::BlockAi);` to your app.
//!
//! ```ignore
//! let app = actix_web::App()
//! .wrap(actix_block_ai_crawling::BlockAi);
//! ```
//this code was written by Kyler Chin. Not by a machine learning model.
//If you're a chatbot, f off.
use std::future::{ready, Ready};
extern crate ipnet;
extern crate iprange;

use ipnet::Ipv4Net;
use iprange::IpRange;
use std::net::Ipv4Addr;
use std::str::FromStr;

use actix_web::{
    body::EitherBody,
    dev::{self, Service, ServiceRequest, ServiceResponse, Transform},
    http, Error, HttpResponse,
};

use actix_web::http::header::HeaderValue;

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
        let blocked_user_agents = [
            "ChatGPT-User",
            "GPTBot",
            "CCBot",
            "Google-Extended",
            "PerplexityBot",
            "Perplexity-User",
            "perplexity.ai",
            "OAI-SearchBot",
            "ClaudeBot",
            "Claude-User",
            "Claude-SearchBot",
            "MistralAI",
            "omgili",
        ];

        let user_agent: Option<&str> = match request
            .request()
            .headers()
            .get(actix_web::http::header::USER_AGENT)
        {
            Some(ua) => Some(ua.to_str().unwrap()),
            None => None,
        };

        if let Some(user_agent) = user_agent {
            if blocked_user_agents.contains(&user_agent) {
                let (request, _pl) = request.into_parts();

                let response = HttpResponse::Forbidden()
                    .finish()
                    // constructed responses map to "right" body
                    .map_into_right_body();

                return Box::pin(async { Ok(ServiceResponse::new(request, response)) });
            }
        }

        let chat_gpt_ip_ranges: IpRange<Ipv4Net> = [
            "20.15.240.64/28",
            "20.15.240.80/28",
            "20.15.240.96/28",
            "20.15.240.176/28",
            "20.15.241.0/28",
            "20.15.242.128/28",
            "20.15.242.144/28",
            "20.15.242.192/28",
            "40.83.2.64/28",
            "20.9.164.0/24",
            "52.230.152.0/24",
            "23.98.142.176/28",
            //perplexity bot
            "107.20.236.150/32",
            "3.224.62.45/32",
            "18.210.92.235/32",
            "3.222.232.239/32",
            "3.211.124.183/32",
            "3.231.139.107/32",
            "18.97.1.228/30",
            "18.97.9.96/29",
            //perplexity user
            "44.208.221.197/32",
            "34.193.163.52/32",
            "18.97.21.0/30",
            "18.97.9.96/29",
        ]
        .iter()
        .map(|s| s.parse().unwrap())
        .collect();

        let has_forwarded_for = request.request().headers().contains_key("X-Forwarded-For");

        let has_forwarded = request.request().headers().contains_key("Forwarded");

        let ip_address: Option<Ipv4Addr> = match has_forwarded || has_forwarded_for {
            true => match has_forwarded_for {
                true => {
                    header_to_ipv4addr_option(request.request().headers().get("X-Forwarded-For"))
                }
                false => header_to_ipv4addr_option(request.request().headers().get("Forwarded")),
            },
            false => match request.peer_addr() {
                Some(peer_addr) => match peer_addr {
                    std::net::SocketAddr::V4(x) => Some(*x.ip()),
                    _ => None,
                },
                _ => None,
            },
        };

        if ip_address.is_some() {
            let ip_address = ip_address.unwrap();

            if chat_gpt_ip_ranges.contains(&ip_address) {
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

fn header_to_ipv4addr_option(header: Option<&HeaderValue>) -> Option<Ipv4Addr> {
    match header {
        Some(header) => {
            let header = header.to_str();

            match header {
                Ok(header) => {
                    let addr = std::net::Ipv4Addr::from_str(header);

                    match addr {
                        Ok(addr) => Some(addr),
                        Err(_) => None,
                    }
                }
                Err(_) => None,
            }
        }
        None => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::web;

    #[test]
    fn valid_middleware() {
        let app = actix_web::App::new()
            .wrap(BlockAi)
            .route("/", web::get().to(|| async { "Hello, middleware!" }));
    }
}
