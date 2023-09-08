// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

// use core::error::Error;
use core::future::Future;
use core_protobuf_data_access::chariott::service_discovery::core::v1::service_registry_client::ServiceRegistryClient;
use core_protobuf_data_access::chariott::service_discovery::core::v1::{
    RegisterRequest, ServiceMetadata,
};
use core_protobuf_data_access::invehicle_digital_twin;
use core_protobuf_data_access::invehicle_digital_twin::v1::invehicle_digital_twin_server::InvehicleDigitalTwinServer;
use env_logger::{Builder, Target};
use futures::StreamExt;
use futures_core::task::Context;
use futures_core::task::Poll;
use log::{debug, error, info, LevelFilter};
use parking_lot::RwLock;
use prost::Message;
use std::boxed::Box;
use std::collections::HashMap;
// use std::fmt::Debug;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use tonic::transport::Server;
use tonic::{Request, Status};
use tower::{Layer, Service, ServiceBuilder};

use tonic::body::BoxBody;

// use hyper::body::Body;

use http_body::Body;

use http_body::Full;

mod invehicle_digital_twin_config;
mod invehicle_digital_twin_impl;

const INVEHICLE_DIGITAL_TWIN_SERVICE_NAMESPACE: &str = "sdv.ibeji";
const INVEHICLE_DIGITAL_TWIN_SERVICE_NAME: &str = "invehicle_digital_twin";
const INVEHICLE_DIGITAL_TWIN_SERVICE_VERSION: &str = "1.0";
const INVEHICLE_DIGITAL_TWIN_SERVICE_COMMUNICATION_KIND: &str = "grpc+proto";
const INVEHICLE_DIGITAL_TWIN_SERVICE_COMMUNICATION_REFERENCE: &str = "https://github.com/eclipse-ibeji/ibeji/blob/main/interfaces/digital_twin/v1/digital_twin.proto";


// https://docs.rs/tower/latest/tower/trait.Layer.html

#[derive(Clone)]
pub struct MyLayer {
}

impl MyLayer {
    pub fn new() -> Self {
        Self {}
    }
}

impl<S> Layer<S> for MyLayer {
    type Service = MyService<S>;

    fn layer(&self, service: S) -> Self::Service {
        MyService {
            service,
        }
    }
}

#[derive(Clone)]
pub struct MyService<S> {
    service: S,
}

impl<S> MyService<S>
{
    fn type_to_string<T>(_: &T) -> String {
        format!("{}", std::any::type_name::<T>())
    }
    
    async fn body_to_bytes(mut body:tonic::transport::Body) -> Vec<u8> {        
        let mut data = Vec::new();
        while let Some(chunk) = body.next().await {
            data.extend_from_slice(&chunk.unwrap());
        }
        data
    }

    /*
    fn create_box_body_from_bytes(bytes: Vec<u8>) -> tonic::body::BoxBody {
        let body = bytes::Bytes::from(bytes);
        let full_body = Full::new(body);
        full_body.boxed_unsync()
        // tonic::body::BoxBody::new(body)
    }
    */

    /*
    impl From<BoxBody<bytes::Bytes, Infallible>> for UnsyncBoxBody<bytes::Bytes, Status> {
        fn from(body: BoxBody<bytes::Bytes, Infallible>) -> Self {
            // Implement the conversion here
        }
    }
    */

    /* 
    /// Convert a [`http_body::Body`] into a [`BoxBody`].
    fn boxed<B>(body: B) -> BoxBody
    where
        B: http_body::Body<Data = bytes::Bytes> + Send + 'static,
        B::Error: Into<Status>,
    {
        Self::try_downcast(body).unwrap_or_else(|body| body.map_err(Error::new).boxed_unsync())
    }

    fn try_downcast<T, K>(k: K) -> Result<T, K>
where
    T: 'static,
    K: Send + 'static,
{
    let mut k = Some(k);
    if let Some(k) = <dyn std::any::Any>::downcast_mut::<Option<T>>(&mut k) {
        Ok(k.take().unwrap())
    } else {
        Err(k.unwrap())
    }
}
*/
}


// https://github.com/hyperium/tonic/issues/733
// https://github.com/hyperium/tonic/blob/master/examples/src/tower/client.rs
// https://github.com/hyperium/tonic/issues/481

// use hyper::{Body, http, Request, Response, Server};

impl<S> Service<http::request::Request<tonic::transport::Body>> for MyService<S>
where
    S: Service<http::request::Request<tonic::transport::Body>,Response=http::response::Response<tonic::body::BoxBody>,Error=Box<dyn std::error::Error + Sync + Send>> + Send,     
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        info!("poll_ready");
        self.service.poll_ready(cx)
    }

   fn call(&mut self, request: http::request::Request<tonic::transport::Body>) -> Self::Future {           
        info!("uri = {}", request.uri());
        let uri = request.uri().to_string();
        let uri_parts: Vec<&str> = uri.split("/").collect();
        let (parts, body) = request.into_parts();
        let new_body;
        let mut must_intercept: bool = false;
        if uri_parts.len() == 5 {
            let service_name = uri_parts[3];
            let method_name = uri_parts[4];
            info!("service name = {}", service_name);
            info!("method name = {}", method_name);
            if method_name == "Register" {

                must_intercept = true;

                let mut body_buf = futures::executor::block_on(Self::body_to_bytes(body));
                // let body_buf = futures::executor::block_on(hyper::body::to_bytes(body)).unwrap();

                // This article helped: https://stackoverflow.com/questions/76758914/parse-grpc-orginal-body-with-tonic-prost
                let grpc_header_length: usize = 5;
                let protobuf_message_buf = body_buf.split_off(grpc_header_length);
                let grpc_header_buf = body_buf;

                let register_request: invehicle_digital_twin::v1::RegisterRequest = Message::decode(&protobuf_message_buf[..]).unwrap();
                info!("register_request = {:?}", register_request);

                // This article helped: https://stackoverflow.com/questions/68203821/prost-the-encode-method-cannot-be-invoked-on-a-trait-object
                let mut new_protobuf_message_buf: Vec<u8> = Vec::new();
                new_protobuf_message_buf.reserve(register_request.encoded_len());
                register_request.encode(&mut new_protobuf_message_buf).unwrap();


                let new_body_chunks: Vec<Result<_, std::io::Error>> = vec![
                    Ok(grpc_header_buf),
                    Ok(new_protobuf_message_buf),
                ];

                let stream = futures_util::stream::iter(new_body_chunks);

                new_body = tonic::transport::Body::wrap_stream(stream);

            }
            else {
                new_body = body;
            }
        } else {
            new_body = body;
        }
  
        let new_request = http::request::Request::from_parts(parts, new_body);

        let fut = self.service.call(new_request);

        // create a response in a future.
        Box::pin(async move {
            info!("fut type: {}", Self::type_to_string(&fut));
                   
            match fut.await {
                Ok(response) => {
                    if ! must_intercept {
                        return Ok(response);
                    } 

                    info!("response: {:?}", &response);
                    let (parts, body) = response.into_parts();
                    info!("parts: {:?}", parts);

                    let body_buf_result = hyper::body::to_bytes(body).await;

                    // std::error::Error 
                    let mut new_body_chunks: Vec<Result<Vec<u8>, Status>> = vec![];
                    // let mut new_body_chunks: Vec<Result<Vec<u8>, Box<dyn std::error::Error>>> = vec![];                    
                    // let mut new_body_chunks: Vec<Result<Vec<u8>, std::io::Error>> = vec![];                 

                    match body_buf_result {
                        Ok(mut body_buf) => {
                            info!("We have an okay body_buf");

                            // This article helped: https://stackoverflow.com/questions/76758914/parse-grpc-orginal-body-with-tonic-prost
                            let grpc_header_length: usize = 5;
                            let protobuf_message_buf = body_buf.split_off(grpc_header_length);
                            let grpc_header_buf = body_buf;
            
                            let register_response: invehicle_digital_twin::v1::RegisterResponse = Message::decode(&protobuf_message_buf[..]).unwrap();
                            info!("register_response = {:?}", register_response);

                            // This article helped: https://stackoverflow.com/questions/68203821/prost-the-encode-method-cannot-be-invoked-on-a-trait-object
                            let mut new_protobuf_message_buf: Vec<u8> = Vec::new();
                            new_protobuf_message_buf.reserve(register_response.encoded_len());
                            register_response.encode(&mut new_protobuf_message_buf).unwrap();

                            // new_body_chunks: Vec<Result<_, std::io::Error>> = vec![
                            new_body_chunks = vec![                                
                                Ok(grpc_header_buf.to_vec()),
                                Ok(new_protobuf_message_buf),
                            ];

                            info!("Successfully prepared new_body_chunks");                      
                        },
                        Err(_) => {
                            // TODO: Do something here.
                        }
                    }

                    let stream = futures_util::stream::iter(new_body_chunks);
                    let new_body = tonic::transport::Body::wrap_stream(stream);

                    info!("We now have a new_body");

/*                    
                    info!("TYPE = {}", Self::type_to_string(&new_body));
                    let new_box_body: BoxBody = BoxBody::new(new_body);
                    // let new_box_body: http_body::combinators::UnsyncBoxBody<HttpBody, Status> = http_body::combinators::UnsyncBoxBody::<HttpBody, Status>::new(new_body);
*/

/*
                    let mut data = Vec::new();
                    let mut  new_body_chunks_iter = new_body_chunks.into_iter();
                    while let Some(chunk) = new_body_chunks_iter.next() {
                        data.extend_from_slice(&chunk.unwrap());
                    }

                    let new_body = hyper::Body::from(data);

*/                    
/*                    
                    // map_err(|err| Status::new(tonic::Code::Unknown, err.to_string()));
                    let bytes = bytes::Bytes::from(data);
                    // let new_body = tonic::body::Body::from(bytes);
                    // let new_body = http_body::Full::new(bytes);
                
                    // pub type BoxBody = UnsyncBoxBody<Bytes, Status>;
                    // let new_box_body = http_body::combinators::UnsyncBoxBody::new(new_body);
                    // let new_box_body =  http_body::combinators::UnsyncBoxBody::<bytes::Bytes, Status>::new(new_body);

                    let new_body = http_body::Full::new(bytes);
                    // let new_box_body: BoxBody = Self::create_box_body_from_bytes(data);
                    let new_box_body = new_body.boxed_unsync();
*/
                    // This Discord post helped: https://discord.com/channels/500028886025895936/628706823104626710/1086425720709992602
                    let new_box_body = new_body.map_err(|e| tonic::Status::from_error(Box::new(e))).boxed_unsync();
                    info!("We now have a new_box_body");
                    // let new_body = hyper::Body::from(bytes);
                    // let new_box_body = BoxBody::new(new_body);
                    let new_response = http::response::Response::from_parts(parts, new_box_body);
                    info!("we now have a new_response");
                    Ok(new_response)
                },
                Err(err) => {
                    Err(err)
                }
            }
        })

        // This article helped: https://docs.rs/tower/latest/tower/trait.Service.html

        // https://github.com/tower-rs/tower/issues/727
        // https://github.com/linkerd/linkerd2-proxy/blob/0814a154ba8c8cc7af394ac3fa6f940bd01755ae/linkerd/stack/src/fail_on_error.rs#LL30-L69C2     
    }
}

/// Register the invehicle digital twin service with Chariott.
///
/// # Arguments
/// * `chariott_uri` - Chariott's URI.
/// * `invehicle_digital_twin_uri` - In-vehicle D
/// igital Twin Service's URI.
async fn register_invehicle_digital_twin_service_with_chariott(
    chariott_uri: &str,
    invehicle_digital_twin_uri: &str,
) -> Result<(), Status> {
    let mut client = ServiceRegistryClient::connect(chariott_uri.to_string())
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

    let service = Some(ServiceMetadata {
        namespace: INVEHICLE_DIGITAL_TWIN_SERVICE_NAMESPACE.to_string(),
        name: INVEHICLE_DIGITAL_TWIN_SERVICE_NAME.to_string(),
        version: INVEHICLE_DIGITAL_TWIN_SERVICE_VERSION.to_string(),
        uri: invehicle_digital_twin_uri.to_string(),
        communication_kind: INVEHICLE_DIGITAL_TWIN_SERVICE_COMMUNICATION_KIND.to_string(),
        communication_reference: INVEHICLE_DIGITAL_TWIN_SERVICE_COMMUNICATION_REFERENCE.to_string(),
    });

    let request = Request::new(RegisterRequest { service });

    client
        .register(request)
        .await
        .map_err(|_| Status::internal("Chariott register request failed"))?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup logging.
    Builder::new().filter(None, LevelFilter::Info).target(Target::Stdout).init();

    info!("The In-Vehicle Digital Twin Service has started.");

    // Load the config.
    let settings = invehicle_digital_twin_config::load_settings();
    let invehicle_digital_twin_authority = settings.invehicle_digital_twin_authority;
    let chariott_uri_option = settings.chariott_uri;

/*
    let my_layer = MyLayer{};

    let layer = ServiceBuilder::new()
        .layer(TraceLayer::new_for_grpc())
        .layer(my_layer)
        .into_inner();

    // Setup the HTTP server.
    let addr: SocketAddr = invehicle_digital_twin_authority.parse()?;
    let invehicle_digital_twin_impl = invehicle_digital_twin_impl::InvehicleDigitalTwinImpl {
        entity_access_info_map: Arc::new(RwLock::new(HashMap::new())),
    };
    let invehicle_digital_twin_address = format!("http://{invehicle_digital_twin_authority}"); // Devskim: ignore DS137138
    let server_future = Server::builder()
        .layer(layer)
        .add_service(InvehicleDigitalTwinServer::new(invehicle_digital_twin_impl))
        .serve(addr);
    info!("The HTTP server is listening on address '{invehicle_digital_twin_address}'");
*/

    let addr: SocketAddr = invehicle_digital_twin_authority.parse()?;

    let invehicle_digital_twin_address = format!("http://{invehicle_digital_twin_authority}"); // Devskim: ignore DS137138
    info!("The HTTP server is listening on address '{invehicle_digital_twin_address}'");



    // Register the invehicle digital twin service with Chariott if Chariott's URI was provided in the config.
    if chariott_uri_option.is_some() {
        let response = register_invehicle_digital_twin_service_with_chariott(
            &chariott_uri_option.unwrap(),
            &invehicle_digital_twin_address,
        )
        .await;
        if let Err(error) = response {
            error!("Failed to register this service with Chariott: '{error}'");
            return Err(error)?;
        }
        info!("This service is now registered with Chariott.");
    } else {
        info!("This service is not using Chariott.");
    }

    let invehicle_digital_twin_impl = invehicle_digital_twin_impl::InvehicleDigitalTwinImpl {
        entity_access_info_map: Arc::new(RwLock::new(HashMap::new())),
    };

    let layer = ServiceBuilder::new()
        .layer(MyLayer::new());

    Server::builder()
        .layer(layer)
        .add_service(InvehicleDigitalTwinServer::new(invehicle_digital_twin_impl))
        .serve(addr)
        .await?;

    debug!("The Digital Twin Service has completed.");

    Ok(())
}
