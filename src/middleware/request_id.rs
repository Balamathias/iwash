use axum::{
    extract::Request,
    http::HeaderValue,
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

/// Middleware to add a unique request ID to each request
/// This helps with tracing and debugging in logs
pub async fn request_id_middleware(mut req: Request, next: Next) -> Response {
    let request_id = Uuid::new_v4().to_string();
    
    // Add request ID to request extensions for handlers to access
    req.extensions_mut().insert(RequestId(request_id.clone()));
    
    // Log the request with request ID
    tracing::info!(
        request_id = %request_id,
        method = %req.method(),
        uri = %req.uri(),
        "incoming request"
    );
    
    let mut response = next.run(req).await;
    
    // Add request ID to response headers
    if let Ok(header_value) = HeaderValue::from_str(&request_id) {
        response.headers_mut().insert("x-request-id", header_value);
    }
    
    response
}

/// Request ID extractor that can be used in handlers
#[derive(Clone, Debug)]
pub struct RequestId(pub String);
