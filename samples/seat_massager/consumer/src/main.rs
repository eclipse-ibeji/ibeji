// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

mod respond_impl;

use digital_twin_model::sdv_v2 as sdv;
use env_logger::{Builder, Target};
use log::{debug, info, warn, LevelFilter};
use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng; // trait needed to initialize StdRng
use samples_common::constants::{digital_twin_operation, digital_twin_protocol};
use samples_common::consumer_config;
use samples_common::utils::{
    discover_digital_twin_provider_using_ibeji, retrieve_invehicle_digital_twin_uri,
};
use samples_protobuf_data_access::async_rpc::v1::request::request_client::RequestClient;
use samples_protobuf_data_access::async_rpc::v1::request::AskRequest;
use samples_protobuf_data_access::async_rpc::v1::respond::respond_server::RespondServer;
use samples_protobuf_data_access::async_rpc::v1::respond::AnswerRequest;
use std::net::SocketAddr;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};
use tonic::transport::Server;
use uuid::Uuid;

use seat_massager_common::TargetedPayload;

/// Start the seat massage steps.
///
/// # Arguments
/// `consumer_uri` - The consumer uri.
/// `instance_id` - The instance id.
/// `provider_uri` - The provider uri.
/// `rx` - The receiver for the asynchrnous channel for AnswerRequest's.
fn start_seat_massage_steps(
    consumer_uri: String,
    instance_id: String,
    provider_uri: String,
    mut rx: mpsc::Receiver<AnswerRequest>,
) {
    debug!("Starting the perform steps sequence.");

    tokio::spawn(async move {
        loop {
            let client_result = RequestClient::connect(provider_uri.clone()).await;
            if client_result.is_err() {
                warn!("Unable to connect. We will retry in a moment.");
                sleep(Duration::from_secs(1)).await;
                continue;
            }
            let mut client = client_result.unwrap();

            let request_id = Uuid::new_v4().to_string();

            let mut rng = StdRng::from_entropy();
            let airbag_identifier = rng.gen_range(1..=15);
            let inflation_level = rng.gen_range(1..=10);
            let duration_in_seconds = rng.gen_range(1..=5);

            let request_payload: sdv::airbag_seat_massager::perform_step::request::TYPE =
                sdv::airbag_seat_massager::perform_step::request::TYPE {
                    step: vec![vec![sdv::airbag_seat_massager::airbag_adjustment::TYPE {
                        airbag_identifier,
                        inflation_level,
                        duration_in_seconds,
                    }]],
                    ..Default::default()
                };

            let request_payload_json: String =
                serde_json::to_string_pretty(&request_payload).unwrap();

            let targeted_payload = TargetedPayload {
                instance_id: instance_id.clone(),
                member_path: sdv::airbag_seat_massager::perform_step::NAME.to_string(),
                operation: digital_twin_operation::INVOKE.to_string(),
                payload: request_payload_json,
            };

            let targeted_payload_json = serde_json::to_string_pretty(&targeted_payload).unwrap();

            let request = tonic::Request::new(AskRequest {
                respond_uri: consumer_uri.clone(),
                request_id: request_id.clone(),
                payload: targeted_payload_json.clone(),
            });

            let response = client.ask(request).await;
            if let Err(status) = response {
                warn!("{status:?}");
                continue;
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

    let consumer_authority = settings
        .consumer_authority
        .expect("consumer_authority must be specified in the config file");

    // Setup the asynchrnous channel for AnswerRequest's.
    let (tx, rx) = mpsc::channel(100);

    let respond_impl = respond_impl::RespondImpl::new(tx);

    // Setup the HTTP server.
    let addr: SocketAddr = consumer_authority.parse().unwrap();
    let server_future = Server::builder().add_service(RespondServer::new(respond_impl)).serve(addr);
    info!("The HTTP server is listening on address '{consumer_authority}'");

    // Retrieve the provider URI.
    let provider_endpoint_info = discover_digital_twin_provider_using_ibeji(
        &invehicle_digital_twin_uri,
        sdv::premium_airbag_seat_massager::ID,
        digital_twin_protocol::GRPC,
        &[digital_twin_operation::INVOKE.to_string()],
    )
    .await
    .unwrap();
    let provider_uri = provider_endpoint_info.uri;
    let instance_id = provider_endpoint_info.context;
    info!("The URI for the premium seat massager's provider is {provider_uri}");

    let consumer_uri = format!("http://{consumer_authority}"); // Devskim: ignore DS137138

    start_seat_massage_steps(consumer_uri.clone(), instance_id, provider_uri.clone(), rx);

    server_future.await?;

    debug!("The Consumer has completed.");

    Ok(())
}
