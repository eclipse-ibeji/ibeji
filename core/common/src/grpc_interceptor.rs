// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use bytes::Bytes;
use core::future::Future;
use dyn_clone::DynClone;
use futures_core::task::{Context, Poll};
use http::uri::Uri;
use http_body::Body;
use hyper::{body::HttpBody, Method};
use log::warn;
use regex::Regex;
use std::error::Error;
use std::pin::Pin;
use tower::{Layer, Service};

// This module provides the gRPC Interceptor construct. It can be used to
// intercept gRPC calls, and examine/modify their requests and responses.

/// The gRPC header represents the gRPC call's Compress-Flag and Message-Length.
const GRPC_HEADER_LENGTH: usize = 5;

/// This is the trait that a gRPC Interceptor needs to imnplement.
pub trait GrpcInterceptor: Sync + DynClone {
    /// Is this interceptor applicable?
    ///
    /// # Arguments
    /// * `service_name` - The gRPC call's service name.
    /// * `method_name` - The gRPC call's method name.
    fn is_applicable(&self, service_name: &str, method_name: &str) -> bool;

    /// Indicates that the request must be handled.
    fn must_handle_request(&self) -> bool;

    /// Indicates that the response must be handled.
    fn must_handle_response(&self) -> bool;

    /// Handle request. Return the new new request.
    ///
    /// # Arguments
    /// * `service_name` - The gRPC call's service name.
    /// * `method_name` - The gRPC call's method name.
    /// * `protobuf_message_bytes` - The request's protobuf messages as bytes.
    fn handle_request(
        &self,
        service_name: &str,
        method_name: &str,
        protobuf_message: Bytes,
    ) -> Result<Bytes, Box<dyn Error + Send + Sync>>;

    /// Handle response. Return the new response.
    ///
    /// # Arguments
    /// * `service_name` - The gRPC call's service name.
    /// * `method_name` - The gRPC call's method name.
    /// * `protobuf_message_bytes` - The response's protobuf messages as bytes.
    fn handle_response(
        &self,
        service_name: &str,
        method_name: &str,
        protobuf_message: Bytes,
    ) -> Result<Bytes, Box<dyn Error + Send + Sync>>;
}

// Macro that allows for clonable dynamic traits.
dyn_clone::clone_trait_object!(GrpcInterceptor);

/// The tower layer that hosts a service that hosts a gRPC Interceptor.
#[derive(Clone)]
pub struct GrpcInterceptorLayer {
    interceptor: Box<dyn GrpcInterceptor + Send>,
}

impl GrpcInterceptorLayer {
    /// Create the tower layer for a gRPC Interceptor.
    ///
    /// # Arguments
    /// * `interceptor` - The boxed gRPC Interceptor.
    pub fn new(interceptor: Box<dyn GrpcInterceptor + Send>) -> Self {
        Self { interceptor }
    }
}

impl<S> Layer<S> for GrpcInterceptorLayer {
    type Service = GrpcInterceptorService<S>;

    fn layer(&self, service: S) -> Self::Service {
        GrpcInterceptorService { service, interceptor: self.interceptor.clone() }
    }
}

#[derive(Clone)]
/// The tower service that hosts a gRPC Interceptor.
pub struct GrpcInterceptorService<S> {
    service: S,
    interceptor: Box<dyn GrpcInterceptor + Send>,
}

impl<S> GrpcInterceptorService<S> {
    /// Retrieve the gRPC service name and method name from a URI.
    /// If it cannot succesfully be parsed, then an empty service name and/or method name will be returned.
    ///
    /// * `uri` - The uri used for the gRPC call.
    fn retrieve_grpc_names_from_uri(uri: &Uri) -> (String, String) {
        let mut service_name = String::new();
        let mut method_name = String::new();
        // A gRPC URI path looks like this "/invehicle_digital_twin.InvehicleDigitalTwin/FindById".
        match Regex::new(r"^/[^/\.]+\.([^/]+)/(.+)$") {
            Ok(regex_pattern) => {
                if let Some(caps) = regex_pattern.captures(uri.path()) {
                    // Note: caps.get(0) represents the entire string that matched.
                    //       In the earlier gRPC URI path example it would be
                    //       "/invehicle_digital_twin.InvehicleDigitalTwin/FindById".
                    //       caps.get(1) and caps.get(2) represent the sub-parts that matched.
                    if caps.len() == 3 {
                        service_name = caps.get(1).unwrap().as_str().to_string();
                        method_name = caps.get(2).unwrap().as_str().to_string();
                    }
                }
            }
            Err(err) => warn!("Regex pattern for gRPC names is not valid: {err}"),
        }

        (service_name, method_name)
    }
}

impl<S> Service<http::request::Request<tonic::transport::Body>> for GrpcInterceptorService<S>
where
    S: Service<
            http::request::Request<tonic::transport::Body>,
            Response = http::response::Response<tonic::body::BoxBody>,
            Error = Box<dyn std::error::Error + Sync + Send>,
        > + Send,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    /// Implementation of tower's Service trait's poll_ready method.
    /// See <https://docs.rs/tower/latest/tower/trait.Service.html>
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    /// Implementation of tower's Service trait's call method.
    /// See <https://docs.rs/tower/latest/tower/trait.Service.html>
    fn call(
        &mut self,
        mut request: http::request::Request<tonic::transport::Body>,
    ) -> Self::Future {
        let interceptor = self.interceptor.clone();

        let (service_name, method_name) = Self::retrieve_grpc_names_from_uri(request.uri());
        let is_applicable = interceptor.is_applicable(&service_name, &method_name) && (request.method() == Method::POST);

        if is_applicable && interceptor.must_handle_request() {
            let (parts, body) = request.into_parts();

            // Rust requires that we initilaize body_bytes_timeout_result.
            // Note: We will never use the initilaized value, so using an Ok value is fine.
            let mut body_bytes_timeout_result = Ok(Ok(Bytes::new()));
            // There is a known issue where hyper::body::to_bytes sometimes hangs in the code below.
            // We will use a timeout to break out when this happens.
            futures::executor::block_on(async {
                body_bytes_timeout_result = async_std::future::timeout(core::time::Duration::from_secs(5), hyper::body::to_bytes(body)).await;
            });
            let mut body_bytes: Bytes = match body_bytes_timeout_result {
                Ok(Ok(bytes)) => bytes,
                Ok(Err(err)) => {
                    return Box::pin(async move {
                        Err(Box::new(err) as Box<dyn std::error::Error + Sync + Send>)
                    });
                },
                Err(err) => {
                    return Box::pin(async move {
                        Err(Box::new(err) as Box<dyn std::error::Error + Sync + Send>)
                    });
                }
            };       

            let protobuf_message_bytes: Bytes = body_bytes.split_off(GRPC_HEADER_LENGTH);
            let grpc_header_bytes = body_bytes;
            let new_protobuf_message_bytes: Bytes = match interceptor.handle_request(
                &service_name,
                &method_name,
                protobuf_message_bytes,
            ) {
                Ok(bytes) => bytes,
                Err(err) => return Box::pin(async move { Err(err) }),
            };
            let new_body_chunks: Vec<Result<_, std::io::Error>> =
                vec![Ok(grpc_header_bytes), Ok(new_protobuf_message_bytes)];
            let stream = futures_util::stream::iter(new_body_chunks);
            let new_body = tonic::transport::Body::wrap_stream(stream);
            request = http::request::Request::from_parts(parts, new_body);
        }

        let fut = self.service.call(request);

        Box::pin(async move {
            let mut response = fut.await?;

            if is_applicable && interceptor.must_handle_response() {
                let (parts, body) = response.into_parts();
                let mut body_bytes = match hyper::body::to_bytes(body).await {
                    Ok(bytes) => bytes,
                    Err(err) => {
                        return Err(Box::new(err) as Box<dyn std::error::Error + Sync + Send>)
                    }
                };
                let protobuf_message_bytes = body_bytes.split_off(GRPC_HEADER_LENGTH);
                let grpc_header_bytes = body_bytes;
                let new_protobuf_message_bytes = match interceptor.handle_response(
                    &service_name,
                    &method_name,
                    protobuf_message_bytes,
                ) {
                    Ok(bytes) => bytes,
                    Err(err) => return Err(err),
                };
                let new_body_chunks: Vec<Result<_, std::io::Error>> =
                    vec![Ok(grpc_header_bytes), Ok(new_protobuf_message_bytes)];
                let stream = futures_util::stream::iter(new_body_chunks);
                let new_body = tonic::transport::Body::wrap_stream(stream);
                let new_box_body = HttpBody::map_err(new_body, |e| tonic::Status::from_error(Box::new(e))).boxed_unsync();
                response = http::response::Response::from_parts(parts, new_box_body);
            }

            Ok(response)
        })
    }
}
