// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use bytes::Bytes;
use core::future::Future;
use core_protobuf_data_access::invehicle_digital_twin;
use futures_core::task::{Context, Poll};
use http::uri::Uri;
use http_body::Body;
use log::{info};
use prost::Message;
use std::pin::Pin;
use tower::{Layer, Service};

const GRPC_HEADER_LENGTH: usize = 5;

pub trait GrpcInterceptingFilter : Sync {
    /// Is this intercepting filter applicable?
    /// 
    /// # Arguments
    /// * `service_name` - The request's associated service name.
    /// * `method_name` - The requests's associated method name.
    fn is_applicable(&self, service_name: &str, method_name: &str) -> bool;

    /// Indicates that the request must be handled.
    fn must_handle_request(&self) -> bool;

    /// Indicates that the response must be handled.
    fn must_handle_response(&self) -> bool;

    /// Handle request.
    fn handle_request(&self, protobuf_message: Bytes) -> Bytes;

    /// Handle response.
    fn handle_response(&self, protobuf_message: Bytes) -> Bytes;
}

type GrpcInterceptingFilterFactory = fn() -> Box<dyn GrpcInterceptingFilter + Send>;

#[derive(Clone)]
pub struct GrpcInterceptingFilterLayer {
    // intercepting_filter: &'static dyn GrpcInterceptingFilter,  
    intercepting_filter_factory: GrpcInterceptingFilterFactory,  
}

impl GrpcInterceptingFilterLayer {
    // pub fn new(intercepting_filter: &'static (dyn GrpcInterceptingFilter + 'static)) -> Self {    
    pub fn new(intercepting_filter_factory: GrpcInterceptingFilterFactory) -> Self {
        Self {
            intercepting_filter_factory: intercepting_filter_factory,
        }
    }
}

impl<S> Layer<S> for GrpcInterceptingFilterLayer  {
    type Service = GrpcInterceptingFilterService<S>;

    fn layer(&self, service: S) -> Self::Service {
        GrpcInterceptingFilterService {
            service,
            intercepting_filter_factory: self.intercepting_filter_factory,
        }
    }
}

#[derive(Clone)]
pub struct GrpcInterceptingFilterService<S> {
    service: S,
    // intercepting_filter: &'static dyn GrpcInterceptingFilter,
    intercepting_filter_factory: GrpcInterceptingFilterFactory,      
}

impl<S> GrpcInterceptingFilterService<S>
{
    fn retrieve_grpc_parts_from_uri(uri: &Uri) -> (String, String) {
        let uri_str = uri.to_string();
        let uri_parts: Vec<&str> = uri_str.split("/").collect();
        let mut service_name = String::new();
        let mut method_name = String::new();
        if uri_parts.len() == 5 {
            method_name = uri_parts[4].to_string();
            let qualified_service_name = uri_parts[3].to_string();
            let qualified_service_name_parts: Vec<&str> = qualified_service_name.split(".").collect();   
            if qualified_service_name_parts.len() == 2 {
                service_name = qualified_service_name_parts[1].to_string();
            }
        }
        (service_name, method_name)
    }
}

impl<S> Service<http::request::Request<tonic::transport::Body>> for GrpcInterceptingFilterService<S>
where
    S: Service<http::request::Request<tonic::transport::Body>,Response=http::response::Response<tonic::body::BoxBody>,Error=Box<dyn std::error::Error + Sync + Send>> + Send,     
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

   fn call(&mut self, mut request: http::request::Request<tonic::transport::Body>) -> Self::Future {
        let intercepting_filter = (self.intercepting_filter_factory)();

        let (service_name, method_name) = Self::retrieve_grpc_parts_from_uri(request.uri());   
        let is_applicable = intercepting_filter.is_applicable(&service_name, &method_name);

        if is_applicable && intercepting_filter.must_handle_request() {
            let (parts, body) = request.into_parts();
            let mut body_bytes: Bytes = futures::executor::block_on(hyper::body::to_bytes(body)).unwrap();
            // This article helped: https://stackoverflow.com/questions/76758914/parse-grpc-orginal-body-with-tonic-prost
            let protobuf_message_bytes: Bytes = body_bytes.split_off(GRPC_HEADER_LENGTH);
            let grpc_header_bytes = body_bytes;
            let new_protobuf_message_bytes: Bytes = intercepting_filter.handle_request(protobuf_message_bytes);
            let new_body_chunks: Vec<Result<_, std::io::Error>> = vec![
                Ok(grpc_header_bytes),
                Ok(new_protobuf_message_bytes),
            ];
            let stream = futures_util::stream::iter(new_body_chunks);
            let new_body = tonic::transport::Body::wrap_stream(stream);
            request = http::request::Request::from_parts(parts, new_body);
        }

        let fut = self.service.call(request);

        Box::pin(async move {
            match fut.await {
                Ok(response) => {
                    if is_applicable && intercepting_filter.must_handle_response() {
                        let (parts, body) = response.into_parts();
                        let mut body_bytes = hyper::body::to_bytes(body).await.unwrap();
                        // This article helped: https://stackoverflow.com/questions/76758914/parse-grpc-orginal-body-with-tonic-prost
                        let protobuf_message_bytes = body_bytes.split_off(GRPC_HEADER_LENGTH);
                        let grpc_header_bytes = body_bytes;
                        let new_protobuf_message_bytes = intercepting_filter.handle_response(protobuf_message_bytes);
                        let new_body_chunks: Vec<Result<_, std::io::Error>> = vec![                                
                            Ok(grpc_header_bytes),
                            Ok(new_protobuf_message_bytes),
                        ];
                        let stream = futures_util::stream::iter(new_body_chunks);
                        let new_body = tonic::transport::Body::wrap_stream(stream);
                        let new_box_body = new_body.map_err(|e| tonic::Status::from_error(Box::new(e))).boxed_unsync();
                        let new_response = http::response::Response::from_parts(parts, new_box_body);
                        return Ok(new_response);
                    } else {
                        return Ok(response);
                    }
                },
                Err(err) => {
                    return Err(err);
                }
            }
        })
   }
}

#[derive(Clone)]
pub struct SampleGrpcInterceptingFilter {
}

impl SampleGrpcInterceptingFilter
{
    const INVEHICLE_DIGITAL_TWIN_SERVICE_NAME: &str = "InvehicleDigitalTwin";
    const REGISTER_METHOD_NAME: &str = "Register";
}

impl GrpcInterceptingFilter for SampleGrpcInterceptingFilter {
    /// Is this intercepting filter applicable?
    /// 
    /// # Arguments
    /// * `service_name` - The request's associated service name.
    /// * `method_name` - The requests's associated method name.
    fn is_applicable(&self, service_name: &str, method_name: &str) -> bool {
        info!("service_name = '{service_name}'  method_name = '{method_name}'");
        service_name == Self::INVEHICLE_DIGITAL_TWIN_SERVICE_NAME && method_name == Self::REGISTER_METHOD_NAME
    }

    /// Indicates that the request must be handled.
    fn must_handle_request(&self) -> bool {
        true
    }

    /// Indicates that the response must be handled.
    fn must_handle_response(&self) -> bool {
        true
    }

    /// Handle request.
    fn handle_request(&self, protobuf_message_bytes: Bytes) -> Bytes {
        let register_request: invehicle_digital_twin::v1::RegisterRequest = Message::decode(&protobuf_message_bytes[..]).unwrap();

        info!("register_request = {:?}", register_request);

        let mut new_protobuf_message_buf: Vec<u8> = Vec::new();
        new_protobuf_message_buf.reserve(register_request.encoded_len());
        register_request.encode(&mut new_protobuf_message_buf).unwrap();
        Bytes::from(new_protobuf_message_buf)
    }

    /// Handle response.
    fn handle_response(&self, protobuf_message_bytes: Bytes) -> Bytes {
        let register_response: invehicle_digital_twin::v1::RegisterResponse = Message::decode(&protobuf_message_bytes[..]).unwrap();

        info!("register_response = {:?}", register_response);

        // This article helped: https://stackoverflow.com/questions/68203821/prost-the-encode-method-cannot-be-invoked-on-a-trait-object
        let mut new_protobuf_message_buf: Vec<u8> = Vec::new();
        new_protobuf_message_buf.reserve(register_response.encoded_len());
        register_response.encode(&mut new_protobuf_message_buf).unwrap();  
        Bytes::from(new_protobuf_message_buf      )
    }
}

pub fn sample_grpc_intercepting_filter_factory() -> Box<dyn GrpcInterceptingFilter + Send> {
    Box::new(SampleGrpcInterceptingFilter{})
}
