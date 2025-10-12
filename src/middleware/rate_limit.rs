use crate::models::User;
use actix_web::dev::{Service, Transform};
use actix_web::{
    body::{BoxBody, EitherBody, MessageBody},
    dev::{ServiceRequest, ServiceResponse},
    Error, HttpMessage, HttpResponse,
};
use deadpool_redis::Pool;
use futures::future::{ok, Ready};
use futures::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};

pub struct RateLimit {
    redis_pool: Pool,
}

impl RateLimit {
    pub fn new(redis_pool: Pool) -> Self {
        RateLimit { redis_pool }
    }
}

impl<S, B> Transform<S, ServiceRequest> for RateLimit
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<EitherBody<B, BoxBody>>;
    type Error = Error;
    type InitError = ();
    type Transform = RateLimitMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(RateLimitMiddleware {
            service: Rc::new(service),
            redis_pool: self.redis_pool.clone(),
        })
    }
}

pub struct RateLimitMiddleware<S> {
    service: Rc<S>,
    redis_pool: Pool,
}

impl<S, B> Service<ServiceRequest> for RateLimitMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<EitherBody<B, BoxBody>>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let redis_pool = self.redis_pool.clone();

        Box::pin(async move {
            // Skip rate limiting for /healthz
            if req.path() == "/healthz" {
                return service.call(req).await.map(|res| res.map_into_left_body());
            }

            // Determine rate limit key (extract user_id early to avoid borrowing issues)
            let (key, limit) = {
                let user_id = req.extensions().get::<User>().map(|u| u.id);
                if let Some(user_id) = user_id {
                    // Authenticated: 60 req/min per user
                    (format!("rate:user:{user_id}"), 60)
                } else {
                    // Unauthenticated: 10 req/min per IP
                    let ip = req
                        .connection_info()
                        .realip_remote_addr()
                        .unwrap_or("unknown")
                        .to_string();
                    (format!("rate:ip:{ip}"), 10)
                }
            };

            // Check rate limit
            match check_rate_limit(&redis_pool, &key, limit).await {
                Ok(allowed) => {
                    if !allowed {
                        let retry_after = 60; // seconds
                        let response = HttpResponse::TooManyRequests()
                            .insert_header(("Retry-After", retry_after.to_string()))
                            .json(serde_json::json!({
                                "error": "rate_limited",
                                "retry_after": retry_after
                            }));
                        return Ok(req.into_response(response).map_into_right_body());
                    }
                }
                Err(e) => {
                    log::error!("Rate limit check failed: {e}");
                    // Allow request on Redis error (fail open)
                }
            }

            service.call(req).await.map(|res| res.map_into_left_body())
        })
    }
}

async fn check_rate_limit(pool: &Pool, key: &str, limit: u64) -> Result<bool, anyhow::Error> {
    let mut conn = pool.get().await?;

    // INCR and check
    let count: u64 = redis::cmd("INCR").arg(key).query_async(&mut *conn).await?;

    if count == 1 {
        // Set expiry on first request
        redis::cmd("EXPIRE")
            .arg(key)
            .arg(60)
            .query_async::<_, ()>(&mut *conn)
            .await?;
    }

    Ok(count <= limit)
}
