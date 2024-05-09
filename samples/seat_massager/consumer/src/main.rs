// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

mod respond_impl;

use digital_twin_model::sdv_v1 as sdv;
use env_logger::{Builder, Target};
use log::{debug, error, info, warn, LevelFilter};
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
use tokio::time::{sleep, timeout, Duration};
use tonic::transport::Server;
use uuid::Uuid;

use seat_massager_common::TargetedPayload;

/// Start the seat massage steps.
///
/// # Arguments
/// `consumer_uri` - The consumer uri.
/// `instance_id` - The instance id.
/// `provider_uri` - The provider uri.
/// `rx` - The receiver for the asynchronous channel for AnswerRequest's.
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

            // Note: The ask id must be a universally unique value.
            let ask_id = Uuid::new_v4().to_string();

            // Randomly generate the airbag adjustment field values.
            let mut rng = StdRng::from_entropy();
            let airbag_identifier = rng.gen_range(1..=15);
            let inflation_level = rng.gen_range(1..=10);
            let inflation_duration_in_seconds = rng.gen_range(1..=5);

            let request_payload: sdv::airbag_seat_massager::perform_step::request::PAYLOAD_TYPE =
                sdv::airbag_seat_massager::perform_step::request::PAYLOAD_TYPE {
                    step: vec![sdv::airbag_seat_massager::airbag_adjustment::TYPE {
                        airbag_identifier,
                        inflation_level,
                        inflation_duration_in_seconds,
                    }],
                    ..Default::default()
                };

            // Serialize the request payload.
            let request_payload_json: String =
                serde_json::to_string_pretty(&request_payload).unwrap();

            let targeted_payload = TargetedPayload {
                instance_id: instance_id.clone(),
                member_path: sdv::airbag_seat_massager::perform_step::NAME.to_string(),
                operation: digital_twin_operation::INVOKE.to_string(),
                payload: request_payload_json,
            };

            // Serialize the targeted payload.
            let targeted_payload_json = serde_json::to_string_pretty(&targeted_payload).unwrap();

            let request = tonic::Request::new(AskRequest {
                respond_uri: consumer_uri.clone(),
                ask_id: ask_id.clone(),
                payload: targeted_payload_json.clone(),
            });

            // Send the ask.
            let response = client.ask(request).await;
            if let Err(status) = response {
                warn!("Unable to call ask, due to {status:?}\nWe will retry in a moment.");
                sleep(Duration::from_secs(1)).await;
                continue;
            }

            // Wait for the answer request.
            let mut answer_request: AnswerRequest = Default::default();
            let mut attempts_after_failure = 0;
            const MAX_ATTEMPTS_AFTER_FAILURE: u8 = 10;
            while attempts_after_failure < MAX_ATTEMPTS_AFTER_FAILURE {
                match timeout(Duration::from_secs(5), rx.recv()).await {
                    Ok(Some(request)) => {
                        if ask_id == request.ask_id {
                            // We have received the answer request that we are expecting.
                            answer_request = request;
                            break;
                        } else {
                            // Ignore this answer request, as it is not the one that we are expecting.
                            warn!("Received an unexpected answer request with ask_id '{}'.  We will retry in a moment.", request.ask_id);
                            // Immediately try again.  This was not a failure, so we do not increment attempts_after_failure or sleep.
                            continue;
                        }
                    }
                    Ok(None) => {
                        error!("Unable to receive an answer request, as the channel is closed.  We will not perform any more steps.");
                        return;
                    }
                    Err(error_message) => {
                        warn!("Failed to receive the answer request.  The error message is '{}'.  We will retry in a moment.", error_message);
                        sleep(Duration::from_secs(1)).await;
                        attempts_after_failure += 1;
                        continue;
                    }
                }
            }

            info!(
                "Received an answer request.  The ask_id is '{}'. The payload is '{}",
                answer_request.ask_id, answer_request.payload
            );

            debug!("Completed the massage step request.");

            // Wait for a second before performing the next step.
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

    // Setup the asynchronous channel for AnswerRequest's.
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
