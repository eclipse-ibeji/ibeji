// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

mod responder_impl;

use digital_twin_model::sdv_v1 as sdv;
use env_logger::{Builder, Target};
use log::{debug, info, LevelFilter, warn};
use samples_common::constants::{digital_twin_operation, digital_twin_protocol};
use samples_common::utils::{discover_digital_twin_provider_using_ibeji, retrieve_invehicle_digital_twin_uri};
use samples_common::consumer_config;
use samples_protobuf_data_access::async_rpc::v1::responder::responder_server::ResponderServer;
use samples_protobuf_data_access::async_rpc::v1::requestor::requestor_client::RequestorClient;
use samples_protobuf_data_access::async_rpc::v1::requestor::{AskRequest};
use samples_protobuf_data_access::async_rpc::v1::responder::{AnswerRequest};
// use serde_derive::{Deserialize, Serialize};
use std::net::SocketAddr;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};
use tonic::transport::Server;
use uuid::Uuid;

use seat_massager_common::TargetedPayload;

/// Start the seat massage steps.
///
/// # Arguments
/// `provider_uri` - The provider uri.
fn start_seat_massage_steps(consumer_uri: String, instance_id: String, provider_uri: String, mut rx: mpsc::Receiver<AnswerRequest>) {
    debug!("Starting the consumer's seat massage sequence.");

    tokio::spawn(async move {
        loop {
            let client_result = RequestorClient::connect(provider_uri.clone()).await;
            if client_result.is_err() {
                warn!("Unable to connect. We will retry in a moment.");
                sleep(Duration::from_secs(1)).await;
                continue;
            }
            let mut client = client_result.unwrap();

            let request_id = Uuid::new_v4().to_string();

            let request_payload: sdv::airbag_seat_massager::perform_step::request::TYPE = sdv::airbag_seat_massager::perform_step::request::TYPE {
                step: vec!(vec!(sdv::airbag_seat_massager::airbag_adjustment::TYPE { airbag_identifier: 1, inflation_level: 10 }))
            };

            let request_payload_json: String = serde_json::to_string_pretty(&request_payload).unwrap();            

            let targeted_payload = TargetedPayload {
                model_id: sdv::airbag_seat_massager::ID.to_string(),
                instance_id: instance_id.clone(),
                member_name: sdv::airbag_seat_massager::perform_step::NAME.to_string(),
                payload: request_payload_json,
            };

            let targeted_payload_json = serde_json::to_string_pretty(&targeted_payload).unwrap();

            let request = tonic::Request::new(AskRequest {
                responder_uri: consumer_uri.clone(),
                request_id: request_id.clone(),
                payload: targeted_payload_json.clone()
            });

            let response = client.ask(request).await;
            if let Err(status) = response {
                warn!("{status:?}");
                continue
            }

            if let Some(answer_request) = rx.recv().await {
                info!("Received an answer request: {:?}", answer_request);
            } else {
                warn!("Failed to receive the answer request.");
                continue;
            }

            debug!("Completed the massage step request.");

            sleep(Duration::from_secs(1)).await;
        }
    });
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup logging.
    Builder::new().filter(None, LevelFilter::Info).target(Target::Stdout).init();

    info!("The Consumer has started.");

    let settings = consumer_config::load_settings();

    let invehicle_digital_twin_uri = retrieve_invehicle_digital_twin_uri(
        settings.invehicle_digital_twin_uri,
        settings.chariott_uri,
    )
    .await?;

    let (tx, rx) = mpsc::channel(100);

    let responder_impl = responder_impl::ResponderImpl::new(tx);

    let consumer_authority = settings
        .consumer_authority
        .expect("consumer_authority must be specified in the config file");

    // Setup the HTTP server.
    let addr: SocketAddr = consumer_authority.parse().unwrap();
    let server_future =
        Server::builder().add_service(ResponderServer::new(responder_impl)).serve(addr);
    info!("The HTTP server is listening on address '{consumer_authority}'");

    // Retrieve the provider URI.
    let provider_endpoint_info = discover_digital_twin_provider_using_ibeji(
        &invehicle_digital_twin_uri,
        sdv::airbag_seat_massager::ID,
        digital_twin_protocol::GRPC,
        &[digital_twin_operation::INVOKE.to_string()],
    )
    .await
    .unwrap();
    let provider_uri = provider_endpoint_info.uri;
    let instance_id = provider_endpoint_info.context;
    info!("The URI for the massage airbags property's provider is {provider_uri}");

    let consumer_uri = format!("http://{consumer_authority}"); // Devskim: ignore DS137138

    start_seat_massage_steps(consumer_uri.clone(), instance_id, provider_uri.clone(), rx);

    server_future.await?;

    debug!("The Consumer has completed.");

    Ok(())
}
