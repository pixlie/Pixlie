use crate::logging::generate_request_id;
use actix_web::{
    Error, HttpMessage,
    dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
};
use futures::future::LocalBoxFuture;
use std::future::{Ready, ready};
use tracing::info;

/// Request tracking middleware for generating and propagating request IDs
pub struct RequestTracking;

impl<S, B> Transform<S, ServiceRequest> for RequestTracking
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RequestTrackingMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequestTrackingMiddleware { service }))
    }
}

pub struct RequestTrackingMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for RequestTrackingMiddleware<S>
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
        // Generate or extract request ID
        let request_id = req
            .headers()
            .get("X-Request-ID")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string())
            .unwrap_or_else(generate_request_id);

        // Store request ID in extensions for access in handlers
        req.extensions_mut().insert(request_id.clone());

        // Extract method and path for logging
        let method = req.method().to_string();
        let path = req.path().to_string();
        let remote_addr = req
            .peer_addr()
            .map(|addr| addr.to_string())
            .unwrap_or_else(|| "unknown".to_string());
        let user_agent = req
            .headers()
            .get("User-Agent")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("unknown")
            .to_string();

        // Create tracing span for this request
        let span = tracing::info_span!(
            "http_request",
            method = method.as_str(),
            path = path.as_str(),
            request_id = request_id.as_str(),
            remote_addr = remote_addr.as_str(),
            user_agent = user_agent.as_str(),
        );

        let start_time = std::time::Instant::now();
        let fut = self.service.call(req);

        Box::pin(async move {
            let _enter = span.enter();

            info!("Request started: {} {}", method, path,);

            let result = fut.await;

            let duration = start_time.elapsed();

            match &result {
                Ok(response) => {
                    info!(
                        "Request completed: {} - status: {} duration: {}ms",
                        request_id,
                        response.status().as_u16(),
                        duration.as_millis(),
                    );
                }
                Err(error) => {
                    tracing::error!(
                        "Request failed: {} - error: {:?} duration: {}ms",
                        request_id,
                        error,
                        duration.as_millis(),
                    );
                }
            }

            result
        })
    }
}

/// Helper function to extract request ID from actix-web request
#[allow(dead_code)]
pub fn get_request_id(req: &ServiceRequest) -> Option<String> {
    req.extensions().get::<String>().cloned()
}

/// Helper function to extract request ID from actix-web HttpRequest
#[allow(dead_code)]
pub fn get_request_id_from_http_request(req: &actix_web::HttpRequest) -> Option<String> {
    req.extensions().get::<String>().cloned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{App, HttpResponse, test, web};

    async fn test_handler(req: actix_web::HttpRequest) -> HttpResponse {
        let request_id = get_request_id_from_http_request(&req);
        HttpResponse::Ok().json(serde_json::json!({
            "request_id": request_id
        }))
    }

    #[actix_web::test]
    async fn test_request_tracking_middleware() {
        let app = test::init_service(
            App::new()
                .wrap(RequestTracking)
                .route("/test", web::get().to(test_handler)),
        )
        .await;

        let req = test::TestRequest::get().uri("/test").to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_request_id_header() {
        let app = test::init_service(
            App::new()
                .wrap(RequestTracking)
                .route("/test", web::get().to(test_handler)),
        )
        .await;

        let custom_id = "custom-request-id-123";
        let req = test::TestRequest::get()
            .uri("/test")
            .insert_header(("X-Request-ID", custom_id))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["request_id"], custom_id);
    }
}
