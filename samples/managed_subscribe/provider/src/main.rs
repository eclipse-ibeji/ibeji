// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

mod provider_impl;

use std::net::SocketAddr;

use digital_twin_model::sdv_v1 as sdv;
use env_logger::{Builder, Target};
use log::{debug, info, warn, LevelFilter};
use samples_common::constants::{digital_twin_operation, digital_twin_protocol};
use samples_common::provider_config;
use samples_common::utils::{retrieve_invehicle_digital_twin_uri, retry_async_based_on_status};
use samples_protobuf_data_access::extensions::managed_subscribe::v1::managed_subscribe_callback_server::ManagedSubscribeCallbackServer;
use samples_protobuf_data_access::invehicle_digital_twin::v1::invehicle_digital_twin_client::InvehicleDigitalTwinClient;
use samples_protobuf_data_access::invehicle_digital_twin::v1::{
    EndpointInfo, EntityAccessInfo, RegisterRequest,
};
use tokio::sync::watch;
use tokio::signal;
use tokio::time::{sleep, Duration};
use tonic::Status;
use tonic::transport::Server;

use crate::provider_impl::ProviderImpl;

const EXTENSION_URI: &str = "http://0.0.0.0:5010";

/// Register the ambient air temperature property's endpoint.
///
/// # Arguments
/// * `invehicle_digital_twin_uri` - The In-Vehicle Digital Twin URI.
/// * `broker_uri` - The broker's URI.
/// * `topic` - The topic.
async fn register_ambient_air_temperature(
    invehicle_digital_twin_uri: &str,
) -> Result<(), Status> {
    let endpoint_info = EndpointInfo {
        protocol: digital_twin_protocol::GRPC.to_string(),
        operations: vec![digital_twin_operation::MANAGEDSUBSCRIBE.to_string()],
        uri: EXTENSION_URI.to_string(),
        context: "GetSubscriptionInfo".to_string(),
    };

    let entity_access_info = EntityAccessInfo {
        name: sdv::hvac::ambient_air_temperature::NAME.to_string(),
        id: sdv::hvac::ambient_air_temperature::ID.to_string(),
        description: sdv::hvac::ambient_air_temperature::DESCRIPTION.to_string(),
        endpoint_info_list: vec![endpoint_info],
    };

    let mut client = InvehicleDigitalTwinClient::connect(invehicle_digital_twin_uri.to_string())
        .await
        .map_err(|e| Status::internal(e.to_string()))?;
    let request =
        tonic::Request::new(RegisterRequest { entity_access_info_list: vec![entity_access_info] });
    let _response = client.register(request).await?;

    Ok(())
}

/// Start the ambient air temperature data stream.
///
/// # Arguments
/// `min_interval_ms` - minimum frequency for data stream.
fn start_ambient_air_temperature_data_stream(min_interval_ms: u64) -> watch::Receiver<i32> {
    debug!("Starting the Provider's ambient air temperature data stream.");
    let mut temperature: i32 = 75;
    let (sender, reciever) = watch::channel(temperature);
    tokio::spawn(async move {
        let mut is_temperature_increasing: bool = true;
        loop {
            debug!(
                "Recording new value for {} of {temperature}",
                sdv::hvac::ambient_air_temperature::ID
            );

            if let Err(err) = sender.send(temperature) {
                warn!("Failed to get new value due to '{err:?}'");
                break;
            }

            debug!("Completed the publish request");

            // Calculate the new temperature.
            // It bounces back and forth between 65 and 85 degrees.
            if is_temperature_increasing {
                if temperature == 85 {
                    is_temperature_increasing = false;
                    temperature -= 1;
                } else {
                    temperature += 1;
                }
            } else if temperature == 65 {
                is_temperature_increasing = true;
                temperature += 1;
            } else {
                temperature -= 1;
            }

            sleep(Duration::from_millis(min_interval_ms)).await;
        }
    });

    reciever
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup logging.
    Builder::new().filter(None, LevelFilter::Info).target(Target::Stdout).init();

    info!("The Provider has started.");

    let settings = provider_config::load_settings();

    let provider_authority = settings.provider_authority;

    let invehicle_digital_twin_uri = retrieve_invehicle_digital_twin_uri(
        settings.invehicle_digital_twin_uri,
        settings.chariott_uri,
    )
    .await?;

    // Start mock data stream.
    let min_interval_ms = 1000; // 1 second
    let data_stream = start_ambient_air_temperature_data_stream(min_interval_ms);

    // Setup provider management cb endpoint.
    let provider = ProviderImpl::new(data_stream, min_interval_ms);

    // Start service.
    let addr: SocketAddr = provider_authority.parse()?;
    let server_future =
        Server::builder().add_service(ManagedSubscribeCallbackServer::new(provider)).serve(addr);

    debug!("Sending a register request to the In-Vehicle Digital Twin Service URI {invehicle_digital_twin_uri}");
    retry_async_based_on_status(30, Duration::from_secs(1), || {
        register_ambient_air_temperature(&invehicle_digital_twin_uri)
    })
    .await?;

    server_future.await?;

    signal::ctrl_c().await.expect("Failed to listen for control-c event");

    info!("The Provider has completed.");

    Ok(())
}
