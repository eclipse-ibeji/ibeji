// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

mod provider_impl;

use digital_twin_model::sdv_v1 as sdv;
use env_logger::{Builder, Target};
use log::{debug, info, warn, LevelFilter};
use parking_lot::{Mutex, MutexGuard};
use proto::digital_twin::digital_twin_client::DigitalTwinClient;
use proto::digital_twin::{EndpointInfo, EntityAccessInfo, RegisterRequest};
use samples_proto::sample_grpc::v1::digital_twin_consumer::digital_twin_consumer_client::DigitalTwinConsumerClient;
use samples_proto::sample_grpc::v1::digital_twin_consumer::PublishRequest;
use samples_proto::sample_grpc::v1::digital_twin_provider::digital_twin_provider_server::DigitalTwinProviderServer;
use std::collections::HashSet;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tonic::transport::Server;

use crate::provider_impl::{ProviderImpl, SubscriptionMap};

const IN_VEHICLE_DIGITAL_TWIN_SERVICE_URI: &str = "http://[::1]:50010"; // Devskim: ignore DS137138
const PROVIDER_AUTHORITY: &str = "[::1]:40010";

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
                    entity_id: String::from(sdv::vehicle::cabin::hvac::ambient_air_temperature::ID),
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

    let endpoint_info = EndpointInfo {
        protocol: String::from("grpc"),
        operations: vec![String::from("Subscribe"), String::from("Unsubscribe")],
        uri: String::from("http://[::1]:40010"), // Devskim: ignore DS137138
        context: String::from(sdv::vehicle::cabin::hvac::ambient_air_temperature::ID),
    };

    let entity_access_info = EntityAccessInfo {
        name: String::from("AmbientAirTemperature"),
        id: String::from(sdv::vehicle::cabin::hvac::ambient_air_temperature::ID),
        description: String::from("The immediate surroundings air temperature (in Fahrenheit)."),
        endpoint_info_list: vec![endpoint_info],
    };

    // Setup the HTTP server.
    let addr: SocketAddr = PROVIDER_AUTHORITY.parse()?;
    let subscription_map = Arc::new(Mutex::new(SubscriptionMap::new()));
    let provider_impl = ProviderImpl { subscription_map: subscription_map.clone() };
    let server_future =
        Server::builder().add_service(DigitalTwinProviderServer::new(provider_impl)).serve(addr);
    info!("The HTTP server is listening on address '{PROVIDER_AUTHORITY}'");

    info!("Sending a register request with the Provider's DTDL to the In-Vehicle Digital Twin Service URI {IN_VEHICLE_DIGITAL_TWIN_SERVICE_URI}");
    let mut client = DigitalTwinClient::connect(IN_VEHICLE_DIGITAL_TWIN_SERVICE_URI).await?;
    let request =
        tonic::Request::new(RegisterRequest { entity_access_info_list: vec![entity_access_info] });
    let _response = client.register(request).await?;
    debug!("The Provider's DTDL has been registered.");

    start_ambient_air_temperature_data_stream(subscription_map.clone());

    server_future.await?;

    debug!("The Provider has completed.");

    Ok(())
}
