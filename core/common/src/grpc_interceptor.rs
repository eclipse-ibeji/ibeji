// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use bytes::Bytes;
use core::future::Future;
use futures_core::task::{Context, Poll};
use http::uri::Uri;
use http_body::Body;
use std::error::Error;
use std::pin::Pin;
use tower::{Layer, Service};

/// This module provides the gRPC Interceptor construct. It can be used to
/// intercept gRPC calls, and examine/modify their requests and responses.

/// This construct is based on the interceptor pattern. Details on the
/// interceptor pattern can be found in wikipedia:
/// https://en.wikipedia.org/wiki/Interceptor_pattern.

/// gRPC Interceptors rely on the tower crate's Layer construct to inject the
/// interceptor into the incoming and outgoing gRPC message paths.

/// These documents/code were very helpful in developing this solution:
/// * https://docs.rs/tower/latest/tower/trait.Layer.html
/// * https://docs.rs/tower/latest/tower/trait.Service.html
/// * https://stackoverflow.com/questions/68203821/prost-the-encode-method-cannot-be-invoked-on-a-trait-object
/// * https://github.com/hyperium/tonic/blob/master/examples/src/tower/client.rs
/// * https://stackoverflow.com/questions/76758914/parse-grpc-orginal-body-with-tonic-prost
/// * https://stackoverflow.com/questions/57632558/grpc-server-complaining-that-message-is-larger-than-max-size-when-its-not
/// * https://discord.com/channels/500028886025895936/628706823104626710/1086425720709992602
/// * https://github.com/tower-rs/tower/issues/727
/// * https://github.com/linkerd/linkerd2-proxy/blob/0814a154ba8c8cc7af394ac3fa6f940bd01755ae/linkerd/stack/src/fail_on_error.rs#LL30-L69C2

/// The gRPC header represents the gRPC call's Compress-Flag and Message-Length.
const GRPC_HEADER_LENGTH: usize = 5;

/// This is the trait that a gRPC Interceptor needs to imnplement.
pub trait GrpcInterceptor: Sync {
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

/// This is the type that represents the factory method for gRPC Interceptors.
type GrpcInterceptorFactory = fn() -> Box<dyn GrpcInterceptor + Send>;

/// The tower layer that hosts a service that hosts a gRPC Interceptor.
#[derive(Clone)]
pub struct GrpcInterceptorLayer {
    interceptor_factory: GrpcInterceptorFactory,
}

impl GrpcInterceptorLayer {
    /// Create the tower layer for a gRPC Interceptor.
    ///
    /// # Arguments
    /// * `interceptor_factory` - The factory method for creating the desired gRPC Interceptor.
    pub fn new(interceptor_factory: GrpcInterceptorFactory) -> Self {
        Self { interceptor_factory }
    }
}

impl<S> Layer<S> for GrpcInterceptorLayer {
    type Service = GrpcInterceptorService<S>;

    fn layer(&self, service: S) -> Self::Service {
        GrpcInterceptorService { service, interceptor_factory: self.interceptor_factory }
    }
}

/// The tower service that hosts a gRPC Interceptor.
#[derive(Clone)]
pub struct GrpcInterceptorService<S> {
    service: S,
    interceptor_factory: GrpcInterceptorFactory,
}

impl<S> GrpcInterceptorService<S> {
    /// Retrieve the gRPC service name and method name from a URI.
    ///
    /// * `uri` - The uri used for the gRPC call.
    fn retrieve_grpc_names_from_uri(uri: &Uri) -> (String, String) {
        let uri_string = uri.to_string();
        let uri_parts: Vec<&str> = uri_string.split('/').collect();
        let mut service_name = String::new();
        let mut method_name = String::new();
        if uri_parts.len() == 5 {
            method_name = uri_parts[4].to_string();
            let qualified_service_name = uri_parts[3].to_string();
            let qualified_service_name_parts: Vec<&str> =
                qualified_service_name.split('.').collect();
            if qualified_service_name_parts.len() == 2 {
                service_name = qualified_service_name_parts[1].to_string();
            }
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
    /// See https://docs.rs/tower/latest/tower/trait.Service.html       
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    /// Implementation of tower's Service trait's call method.
    /// See https://docs.rs/tower/latest/tower/trait.Service.html
    fn call(
        &mut self,
        mut request: http::request::Request<tonic::transport::Body>,
    ) -> Self::Future {
        let interceptor = (self.interceptor_factory)();

        let (service_name, method_name) = Self::retrieve_grpc_names_from_uri(request.uri());
        let is_applicable = interceptor.is_applicable(&service_name, &method_name);

        if is_applicable && interceptor.must_handle_request() {
            let (parts, body) = request.into_parts();
            let mut body_bytes: Bytes =
                match futures::executor::block_on(hyper::body::to_bytes(body)) {
                    Ok(bytes) => bytes,
                    Err(err) => {
                        return Box::pin(async move {
                            Err(Box::new(err) as Box<dyn std::error::Error + Sync + Send>)
                        })
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
            match fut.await {
                Ok(response) => {
                    if is_applicable && interceptor.must_handle_response() {
                        let (parts, body) = response.into_parts();
                        let mut body_bytes = match hyper::body::to_bytes(body).await {
                            Ok(bytes) => bytes,
                            Err(err) => {
                                return Err(
                                    Box::new(err) as Box<dyn std::error::Error + Sync + Send>
                                )
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
                        let new_box_body = new_body
                            .map_err(|e| tonic::Status::from_error(Box::new(e)))
                            .boxed_unsync();
                        let new_response =
                            http::response::Response::from_parts(parts, new_box_body);
                        Ok(new_response)
                    } else {
                        Ok(response)
                    }
                }
                Err(err) => Err(err),
            }
        })
    }
}
