// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

mod consumer_impl;

use digital_twin_model::{sdv_v1 as sdv, Metadata};
use env_logger::{Builder, Target};
use log::{debug, info, LevelFilter, warn};
use samples_common::constants::{digital_twin_operation, digital_twin_protocol};
use samples_common::utils::{discover_digital_twin_provider_using_ibeji, retrieve_invehicle_digital_twin_url};
use samples_common::consumer_config;
use samples_protobuf_data_access::sample_grpc::v1::digital_twin_consumer::digital_twin_consumer_server::DigitalTwinConsumerServer;
use samples_protobuf_data_access::sample_grpc::v1::digital_twin_provider::digital_twin_provider_client::DigitalTwinProviderClient;
use samples_protobuf_data_access::sample_grpc::v1::digital_twin_provider::{GetRequest, SetRequest};
use serde_derive::{Deserialize, Serialize};
use std::cmp::max;
use std::net::SocketAddr;
use tokio::time::{sleep, Duration};
use tonic::transport::Server;

#[derive(Debug, Serialize, Deserialize)]
struct Property {
    #[serde(rename = "MassageAirbags")]
    massage_airbags: sdv::airbag_seat_massager::massage_airbags::TYPE,
    #[serde(rename = "$metadata")]
    metadata: Metadata,
}

/// Start the seat massage sequence.
///
/// # Arguments
/// `provider_uri` - The provider uri.
fn start_seat_massage_sequence(provider_uri: String) {
    debug!("Starting the consumer's seat massage sequence.");

    let mut crest_row: i8 = 0;
    let mut is_wave_moving_forwards = true;
    const MAX_ROW: i8 = 5;

    let metadata: Metadata =
        Metadata { model: sdv::airbag_seat_massager::massage_airbags::ID.to_string() };
    let mut property: Property = Property { massage_airbags: Vec::new(), metadata };

    tokio::spawn(async move {
        loop {
            let client_result = DigitalTwinProviderClient::connect(provider_uri.clone()).await;
            if client_result.is_err() {
                warn!("Unable to connect. We will retry in a moment.");
                sleep(Duration::from_secs(1)).await;
                continue;
            }
            let mut client = client_result.unwrap();

            // We assume that the seat has 6 rows with 3 airbags in each row.
            // The sequence will mimic a wave motion.  With the crest of the wave
            // having the maximum inflation level and either side of the crest
            // having gradually decreasing inflation levels.
            // crest_row represents the row where the crest of the wave is located.
            property.massage_airbags = Vec::new();
            for airbag in 0..18 {
                let row = airbag / 3;
                let rows_from_crest = i8::abs(crest_row - row);
                let inflation_level = max(100 - (rows_from_crest * 20), 0);
                property.massage_airbags.push(inflation_level as i32);
            }

            let value = serde_json::to_string_pretty(&property).unwrap();

            info!(
                "Sending a set request for entity id {} to provider URI {provider_uri}",
                sdv::airbag_seat_massager::massage_airbags::ID
            );

            let request = tonic::Request::new(SetRequest {
                entity_id: sdv::airbag_seat_massager::massage_airbags::ID.to_string(),
                value,
            });

            let response = client.set(request).await;
            if let Err(status) = response {
                warn!("{status:?}");
            }

            // Set crest_row for the next loop iteration.
            if crest_row == 0 {
                is_wave_moving_forwards = true;
            } else if crest_row == MAX_ROW {
                is_wave_moving_forwards = false;
            }
            if is_wave_moving_forwards {
                crest_row += 1;
            } else {
                crest_row -= 1;
            }

            debug!("Completed the set request.");

            sleep(Duration::from_secs(1)).await;
        }
    });
}

/// Start the seat massage get repeater.
///
/// # Arguments
/// `provider_uri` - The provider uri.
/// `consumer_uri` - The consumer uri.
fn start_seat_massage_get_repeater(provider_uri: String, consumer_uri: String) {
    debug!("Starting the consumer's seat massage get repeater.");

    tokio::spawn(async move {
        loop {
            info!(
                "Sending a get request for entity id {} to provider URI {provider_uri}",
                sdv::airbag_seat_massager::massage_airbags::ID
            );

            let client_result = DigitalTwinProviderClient::connect(provider_uri.clone()).await;
            if client_result.is_err() {
                warn!("Unable to connect. We will retry in a moment.");
                sleep(Duration::from_secs(1)).await;
                continue;
            }
            let mut client = client_result.unwrap();

            let request = tonic::Request::new(GetRequest {
                entity_id: sdv::airbag_seat_massager::massage_airbags::ID.to_string(),
                consumer_uri: consumer_uri.clone(),
            });

            let response = client.get(request).await;
            if let Err(status) = response {
                warn!("{status:?}");
            }

            debug!("Completed the get request.");

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

    let invehicle_digital_twin_url = retrieve_invehicle_digital_twin_url(
        settings.invehicle_digital_twin_url,
        settings.chariott_url,
    )
    .await?;

    let consumer_authority = settings.consumer_authority;

    // Setup the HTTP server.
    let addr: SocketAddr = consumer_authority.parse().unwrap();
    let consumer_impl = consumer_impl::ConsumerImpl::default();
    let server_future =
        Server::builder().add_service(DigitalTwinConsumerServer::new(consumer_impl)).serve(addr);
    info!("The HTTP server is listening on address '{consumer_authority}'");

    // Retrieve the provider URI.
    let provider_endpoint_info = discover_digital_twin_provider_using_ibeji(
        &invehicle_digital_twin_url,
        sdv::airbag_seat_massager::massage_airbags::ID,
        digital_twin_protocol::GRPC,
        &[digital_twin_operation::SET.to_string()],
    )
    .await
    .unwrap();
    let provider_uri = provider_endpoint_info.uri;
    info!("The URI for the massage airbags property's provider is {provider_uri}");

    let consumer_uri = format!("http://{consumer_authority}"); // Devskim: ignore DS137138

    start_seat_massage_sequence(provider_uri.clone());

    start_seat_massage_get_repeater(provider_uri, consumer_uri);

    server_future.await?;

    debug!("The Consumer has completed.");

    Ok(())
}
