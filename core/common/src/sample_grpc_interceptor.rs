// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use bytes::Bytes;
use core_protobuf_data_access::invehicle_digital_twin;
use log::info;
use prost::Message;
use std::error::Error;

use crate::grpc_interceptor::GrpcInterceptor;

/// Sample gRPC interceptor.
#[derive(Clone)]
pub struct SampleGrpcInterceptor {}

impl SampleGrpcInterceptor {
    const INVEHICLE_DIGITAL_TWIN_SERVICE_NAME: &str = "InvehicleDigitalTwin";
    const REGISTER_METHOD_NAME: &str = "Register";

    /// The factory method for creating a SampleGrpcInterceptor.
    pub fn sample_grpc_interceptor_factory() -> Box<dyn GrpcInterceptor + Send> {
        Box::new(SampleGrpcInterceptor {})
    }
}

impl GrpcInterceptor for SampleGrpcInterceptor {
    /// Is this interceptor applicable?
    ///
    /// # Arguments
    /// * `service_name` - The gRPC call's service name.
    /// * `method_name` - The gRPC call's method name.
    fn is_applicable(&self, service_name: &str, method_name: &str) -> bool {
        service_name == Self::INVEHICLE_DIGITAL_TWIN_SERVICE_NAME
            && method_name == Self::REGISTER_METHOD_NAME
    }

    /// Indicates that the request must be handled.
    fn must_handle_request(&self) -> bool {
        true
    }

    /// Indicates that the response must be handled.
    fn must_handle_response(&self) -> bool {
        true
    }

    /// Handle request. Return the new request.
    ///
    /// # Arguments
    /// * `service_name` - The gRPC call's service name.
    /// * `method_name` - The gRPC call's method name.
    /// * `protobuf_message_bytes` - The request's protobuf messages as bytes.
    fn handle_request(
        &self,
        _service_name: &str,
        _method_name: &str,
        protobuf_message_bytes: Bytes,
    ) -> Result<Bytes, Box<dyn Error + Send + Sync>> {
        let register_request: invehicle_digital_twin::v1::RegisterRequest =
            Message::decode(&protobuf_message_bytes[..]).unwrap();

        info!("register_request = {:?}", register_request);

        let mut new_protobuf_message_buf: Vec<u8> = Vec::new();
        new_protobuf_message_buf.reserve(register_request.encoded_len());
        register_request.encode(&mut new_protobuf_message_buf).unwrap();
        Ok(Bytes::from(new_protobuf_message_buf))
    }

    /// Handle response. Return the new response.
    ///
    /// # Arguments
    /// * `service_name` - The gRPC call's service name.
    /// * `method_name` - The gRPC call's method name.
    /// * `protobuf_message_bytes` - The response's protobuf messages as bytes.
    fn handle_response(
        &self,
        _service_name: &str,
        _method_name: &str,
        protobuf_message_bytes: Bytes,
    ) -> Result<Bytes, Box<dyn Error + Send + Sync>> {
        let register_response: invehicle_digital_twin::v1::RegisterResponse =
            Message::decode(&protobuf_message_bytes[..]).unwrap();

        info!("register_response = {:?}", register_response);

        let mut new_protobuf_message_buf: Vec<u8> = Vec::new();
        new_protobuf_message_buf.reserve(register_response.encoded_len());
        register_response.encode(&mut new_protobuf_message_buf).unwrap();
        Ok(Bytes::from(new_protobuf_message_buf))
    }
}
