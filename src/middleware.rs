use std::future::{ready, Ready};
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures_util::future::LocalBoxFuture;
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use crate::models::jwt::{Claims, Access};

pub struct Guardian;

impl<S, B> Transform<S, ServiceRequest> for Guardian
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = Middleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(Middleware { service }))
    }
}

pub struct Middleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for Middleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let path = req.path().to_string();
        let is_admin_route = path.starts_with("/admin");
        let is_register_route = path == "/client/register" || path == "/admin/register";

        let server_key = std::env::var("SERVER_KEY").ok();
        let admin_key = std::env::var("ADMIN_KEY").ok();

        // 🔒 Validate X-Server-Key
        let req_server_key = req
            .headers()
            .get("X-Server-Key")
            .and_then(|h| h.to_str().ok());

        if server_key.is_none() || req_server_key != server_key.as_deref() {
            return Box::pin(async {
                Err(actix_web::error::ErrorUnauthorized("Invalid or missing X-Server-Key"))
            });
        }

        // 🔒 Validate X-Admin-Key for /admin routes
        if is_admin_route {
            let req_admin_key = req
                .headers()
                .get("X-Admin-Key")
                .and_then(|h| h.to_str().ok());

            if admin_key.is_none() || req_admin_key != admin_key.as_deref() {
                return Box::pin(async {
                    Err(actix_web::error::ErrorUnauthorized("Invalid or missing X-Admin-Key"))
                });
            }
        }

        // Allow register route to skip token check
        if is_register_route {
            let fut = self.service.call(req);
            return Box::pin(async move {
                let res = fut.await?;
                Ok(res)
            });
        }

        // Validate JWT
        let jwt_secret = match std::env::var("JWT_SECRET") {
            Ok(secret) => secret,
            Err(_) => {
                return Box::pin(async {
                    Err(actix_web::error::ErrorInternalServerError("JWT_SECRET not set"))
                });
            }
        };

        let auth_header = req
            .headers()
            .get("Authorization")
            .and_then(|h| h.to_str().ok());

        let token = match auth_header {
            Some(header) if header.starts_with("Bearer ") => {
                Some(header.trim_start_matches("Bearer ").trim())
            }
            _ => None,
        };

        if token.is_none() {
            return Box::pin(async {
                Err(actix_web::error::ErrorUnauthorized("Missing or invalid Authorization header"))
            });
        }

        let decoded = decode::<Claims>(
            token.unwrap(),
            &DecodingKey::from_secret(jwt_secret.as_bytes()),
            &Validation::new(Algorithm::HS256),
        );

        match decoded {
            Ok(data) => {
                let claims = data.claims;

                if is_admin_route && claims.access != Access::Admin {
                    return Box::pin(async {
                        Err(actix_web::error::ErrorForbidden("Admin access required"))
                    });
                }

                req.extensions_mut().insert(claims);

                let fut = self.service.call(req);
                Box::pin(async move {
                    let res = fut.await?;
                    Ok(res)
                })
            }
            Err(e) => Box::pin(async move {
                Err(actix_web::error::ErrorUnauthorized(format!("Invalid token: {}", e)))
            }),
        }
    }
}
