/*  Copyright 2022-23, Juspay India Pvt Ltd
    This program is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License
    as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version. This program
    is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY
    or FITNESS FOR A PARTICULAR PURPOSE. See the GNU Affero General Public License for more details. You should have received a copy of
    the GNU Affero General Public License along with this program. If not, see <https://www.gnu.org/licenses/>.
*/

use crate::incoming_api;
use crate::tools::prometheus::INCOMING_API;
use actix::fut::{ready, Ready};
use actix_http::StatusCode;
use actix_web::{
    body::{BoxBody, MessageBody},
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpRequest,
};
use futures::future::LocalBoxFuture;
use regex::Regex;
use tokio::time::Instant;
use tracing::Span;
use tracing::{error, info};
use tracing_actix_web::{DefaultRootSpanBuilder, RootSpanBuilder};
use uuid::Uuid;

/// Responsible for building and managing root spans in the domain.
///
/// `DomainRootSpanBuilder` creates root spans that encapsulate the lifecycle of a request within
/// the domain. It extracts essential information such as request_id, merchant_id, and token from
/// the headers of incoming requests to enrich the spans.
pub struct DomainRootSpanBuilder;

impl RootSpanBuilder for DomainRootSpanBuilder {
    fn on_request_start(request: &ServiceRequest) -> Span {
        let request_id = request
            .headers()
            .get("x-request-id")
            .and_then(|request_id| request_id.to_str().ok())
            .map(|str| str.to_string())
            .unwrap_or(Uuid::new_v4().to_string());

        let merchant_id = request
            .headers()
            .get("mid")
            .and_then(|merchant_id| merchant_id.to_str().ok())
            .map(|str| str.to_string());

        let token = request
            .headers()
            .get("token")
            .and_then(|token| token.to_str().ok())
            .map(|str| str.to_string());

        tracing_actix_web::root_span!(request, request_id, merchant_id, token)
    }

    fn on_request_end<B: MessageBody>(span: Span, outcome: &Result<ServiceResponse<B>, Error>) {
        DefaultRootSpanBuilder::on_request_end(span, outcome);
    }
}

/// Responsible for collecting metrics from incoming requests and their responses.
///
/// `IncomingRequestMetrics` acts as a middleware, capturing essential information and
/// metrics from incoming service requests and the corresponding responses or errors.
/// The collected metrics can include headers, paths, methods, and the duration of request handling.
pub struct IncomingRequestMetrics;

impl<S> Transform<S, ServiceRequest> for IncomingRequestMetrics
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type InitError = ();
    type Transform = IncomingRequestMetricsMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(IncomingRequestMetricsMiddleware { service }))
    }
}

pub struct IncomingRequestMetricsMiddleware<S> {
    service: S,
}

impl<S> Service<ServiceRequest> for IncomingRequestMetricsMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let start_time = Instant::now();

        let req_headers = get_headers(req.request());
        let req_path = get_path(req.request());
        let req_method = get_method(req.request());

        let fut = self.service.call(req);
        Box::pin(async move {
            match fut.await {
                Ok(response) => {
                    calculate_metrics(
                        response.response().error(),
                        response.status(),
                        get_headers(response.request()),
                        get_method(response.request()),
                        get_path(response.request()),
                        start_time,
                    );
                    Ok(response)
                }
                Err(err) => {
                    let err_resp_status = err.error_response().status();
                    calculate_metrics(
                        Some(&err),
                        err_resp_status,
                        req_headers,
                        req_method,
                        req_path,
                        start_time,
                    );
                    Err(err)
                }
            }
        })
    }
}

/// Get the path from the HTTP request.
///
/// Retrieves the path from the incoming request and replaces any matched info with placeholders.
///
/// # Arguments
/// * `request` - The incoming HTTP request.
///
/// # Returns
/// * `String` - The path string with placeholders for matched info.
fn get_path(request: &HttpRequest) -> String {
    let mut path = urlencoding::decode(request.path())
        .ok()
        .map(|s| s.to_string())
        .unwrap_or(request.path().to_string());

    request
        .match_info()
        .iter()
        .for_each(|(path_name, path_val)| {
            path = path.replace(path_val, format!(":{path_name}").as_str());
        });

    if let Ok(re) =
        Regex::new(r"[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}")
    {
        path = re.replace_all(&path, ":id").to_string()
    }

    path
}

/// Get the method from the HTTP request.
///
/// Retrieves the HTTP method (e.g., GET, POST) from the incoming request.
///
/// # Arguments
/// * `request` - The incoming HTTP request.
///
/// # Returns
/// * `String` - The HTTP method as a string.
fn get_method(request: &HttpRequest) -> String {
    request.method().to_string()
}

/// Get the headers from the HTTP request.
///
/// Retrieves and formats the headers from the incoming HTTP request.
///
/// # Arguments
/// * `request` - The incoming HTTP request.
///
/// # Returns
/// * `String` - A formatted string representation of the headers.
fn get_headers(request: &HttpRequest) -> String {
    format!("{:?}", request.headers())
}

/// Calculate and log metrics from HTTP requests and responses.
///
/// This function calculates metrics such as latency and logs information including
/// error responses, HTTP methods, paths, and headers.
///
/// # Arguments
/// * `err_resp` - Optional reference to an error response.
/// * `resp_status` - The status code of the response.
/// * `req_headers` - A string representation of the request headers.
/// * `req_method` - The HTTP method of the request as a string.
/// * `req_path` - The path of the request as a string.
/// * `time` - The instant at which the request was received.
fn calculate_metrics(
    err_resp: Option<&Error>,
    resp_status: StatusCode,
    req_headers: String,
    req_method: String,
    req_path: String,
    time: Instant,
) {
    if let Some(err_resp) = err_resp {
        let err_resp_code = err_resp.to_string();
        error!(tag = "[INCOMING API - ERROR]", request_method = %req_method, request_path = %req_path, request_headers = req_headers, response_code = err_resp_code, response_status = resp_status.as_str(), latency = format!("{:?}ms", time.elapsed().as_millis()));
        incoming_api!(
            req_method.as_str(),
            req_path.as_str(),
            resp_status.as_str(),
            err_resp_code.as_str(),
            time
        );
    } else {
        info!(tag = "[INCOMING API]", request_method = %req_method, request_path = %req_path, request_headers = req_headers, response_status = resp_status.as_str(), latency = format!("{:?}ms", time.elapsed().as_millis()));
        incoming_api!(
            req_method.as_str(),
            req_path.as_str(),
            resp_status.as_str(),
            "SUCCESS",
            time
        );
    }
}
