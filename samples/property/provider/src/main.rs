// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

mod provider_impl;

use digital_twin_model::sdv_v1 as sdv;
use env_logger::{Builder, Target};
use log::{debug, info, warn, LevelFilter};
use parking_lot::{Mutex, MutexGuard};
use samples_common::constants::{digital_twin_operation, digital_twin_protocol};
use samples_common::misc::{retrieve_invehicle_digital_twin_url, retry_async_based_on_status};
use samples_common::provider_config::load_settings;
use samples_protobuf_data_access::digital_twin::v1::digital_twin_client::DigitalTwinClient;
use samples_protobuf_data_access::digital_twin::v1::{EndpointInfo, EntityAccessInfo, RegisterRequest};
use samples_protobuf_data_access::sample_grpc::v1::digital_twin_consumer::digital_twin_consumer_client::DigitalTwinConsumerClient;
use samples_protobuf_data_access::sample_grpc::v1::digital_twin_consumer::PublishRequest;
use samples_protobuf_data_access::sample_grpc::v1::digital_twin_provider::digital_twin_provider_server::DigitalTwinProviderServer;
use std::collections::HashSet;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tonic::{Status, transport::Server};

use crate::provider_impl::{ProviderImpl, SubscriptionMap};

/// Register the ambient air temperature property's endpoint.
///
/// # Arguments
/// * `invehicle_digital_twin_url` - The In-Vehicle Digital Twin URL.
/// * `provider_uri` - The provider's URI.
async fn register_ambient_air_temperature(
    invehicle_digital_twin_url: &str,
    provider_uri: &str,
) -> Result<(), Status> {
    let endpoint_info = EndpointInfo {
        protocol: digital_twin_protocol::GRPC.to_string(),
        operations: vec![
            digital_twin_operation::SUBSCRIBE.to_string(),
            digital_twin_operation::UNSUBSCRIBE.to_string(),
        ],
        uri: provider_uri.to_string(),
        context: sdv::vehicle::cabin::hvac::ambient_air_temperature::ID.to_string(),
    };

    let entity_access_info = EntityAccessInfo {
        name: "AmbientAirTemperature".to_string(),
        id: sdv::vehicle::cabin::hvac::ambient_air_temperature::ID.to_string(),
        description: "The immediate surroundings air temperature (in Fahrenheit).".to_string(),
        endpoint_info_list: vec![endpoint_info],
    };

    let mut client = DigitalTwinClient::connect(invehicle_digital_twin_url.to_string())
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
/// `id_to_subscribers_map` - The id to subscribers map.
#[allow(clippy::collapsible_else_if)]
fn start_ambient_air_temperature_data_stream(subscription_map: Arc<Mutex<SubscriptionMap>>) {
    debug!("Starting the Provider's ambient air temperature data stream.");
    tokio::spawn(async move {
        let mut temperature: u32 = 75;
        let mut is_temperature_increasing: bool = true;
        loop {
            let urls;

            // This block controls the lifetime of the lock.
            {
                let lock: MutexGuard<SubscriptionMap> = subscription_map.lock();
                let get_result = lock.get(sdv::vehicle::cabin::hvac::ambient_air_temperature::ID);
                urls = match get_result {
                    Some(val) => val.clone(),
                    None => HashSet::new(),
                };
            }

            for url in urls {
                info!("Sending a publish request for {} with value {temperature} to consumer URI {url}",
                    sdv::vehicle::cabin::hvac::ambient_air_temperature::ID);

                let client_result = DigitalTwinConsumerClient::connect(url).await;
                if client_result.is_err() {
                    warn!("Unable to connect. We will retry in a moment.");
                    sleep(Duration::from_secs(1)).await;
                    continue;
                }
                let mut client = client_result.unwrap();

                let request = tonic::Request::new(PublishRequest {
                    entity_id: sdv::vehicle::cabin::hvac::ambient_air_temperature::ID.to_string(),
                    value: temperature.to_string(),
                });

                let _response = client.publish(request).await;

                debug!("Completed the publish request");
            }

            // Calculate the new temperature.
            // It bounces back and forth between 65 and 85 degrees.
            if is_temperature_increasing {
                if temperature == 85 {
                    is_temperature_increasing = false;
                    temperature -= 1;
                } else {
                    temperature += 1;
                }
            } else {
                if temperature == 65 {
                    is_temperature_increasing = true;
                    temperature += 1;
                } else {
                    temperature -= 1;
                }
            }

            sleep(Duration::from_secs(5)).await;
        }
    });
}

#[tokio::main]
#[allow(clippy::collapsible_else_if)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup logging.
    Builder::new().filter(None, LevelFilter::Info).target(Target::Stdout).init();

    info!("The Provider has started.");

    let settings = load_settings();

    let provider_authority = settings.provider_authority;

    let invehicle_digital_twin_url = retrieve_invehicle_digital_twin_url(
        settings.invehicle_digital_twin_url,
        settings.chariott_url,
    )
    .await?;

    // Construct the provider URI from the provider authority.
    let provider_uri = format!("http://{provider_authority}"); // Devskim: ignore DS137138

    // Setup the HTTP server.
    let addr: SocketAddr = provider_authority.parse()?;
    let subscription_map = Arc::new(Mutex::new(SubscriptionMap::new()));
    let provider_impl = ProviderImpl { subscription_map: subscription_map.clone() };
    let server_future =
        Server::builder().add_service(DigitalTwinProviderServer::new(provider_impl)).serve(addr);
    info!("The HTTP server is listening on address '{provider_authority}'");

    info!("Sending a register request to the In-Vehicle Digital Twin Service URI {invehicle_digital_twin_url}");
    retry_async_based_on_status(30, Duration::from_secs(1), || {
        register_ambient_air_temperature(&invehicle_digital_twin_url, &provider_uri)
    })
    .await?;

    start_ambient_air_temperature_data_stream(subscription_map.clone());

    server_future.await?;

    debug!("The Provider has completed.");

    Ok(())
}
