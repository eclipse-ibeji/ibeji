// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use bytes::Bytes;
use core_protobuf_data_access::invehicle_digital_twin;
use log::info;
use prost::Message;

use crate::intercepting_filter::GrpcInterceptingFilter;

#[derive(Clone)]
pub struct SampleGrpcInterceptingFilter {}

impl SampleGrpcInterceptingFilter {
    const INVEHICLE_DIGITAL_TWIN_SERVICE_NAME: &str = "InvehicleDigitalTwin";
    const REGISTER_METHOD_NAME: &str = "Register";
}

#[allow(unused_variables)]
impl GrpcInterceptingFilter for SampleGrpcInterceptingFilter {
    /// Is this intercepting filter applicable?
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
        service_name: &str,
        method_name: &str,
        protobuf_message_bytes: Bytes,
    ) -> Bytes {
        let register_request: invehicle_digital_twin::v1::RegisterRequest =
            Message::decode(&protobuf_message_bytes[..]).unwrap();

        info!("register_request = {:?}", register_request);

        let mut new_protobuf_message_buf: Vec<u8> = Vec::new();
        new_protobuf_message_buf.reserve(register_request.encoded_len());
        register_request.encode(&mut new_protobuf_message_buf).unwrap();
        Bytes::from(new_protobuf_message_buf)
    }

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
        protobuf_message_bytes: Bytes,
    ) -> Bytes {
        let register_response: invehicle_digital_twin::v1::RegisterResponse =
            Message::decode(&protobuf_message_bytes[..]).unwrap();

        info!("register_response = {:?}", register_response);

        let mut new_protobuf_message_buf: Vec<u8> = Vec::new();
        new_protobuf_message_buf.reserve(register_response.encoded_len());
        register_response.encode(&mut new_protobuf_message_buf).unwrap();
        Bytes::from(new_protobuf_message_buf)
    }
}

/// The factory method for creating a SampleGrpcInterceptingFilter.
pub fn sample_grpc_intercepting_filter_factory() -> Box<dyn GrpcInterceptingFilter + Send> {
    Box::new(SampleGrpcInterceptingFilter {})
}
