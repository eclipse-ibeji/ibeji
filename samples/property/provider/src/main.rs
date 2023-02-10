// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

mod provider_impl;

use dt_model_identifiers::sdv_v1 as sdv;
use env_logger::{Builder, Target};
use ibeji_common::{find_full_path, retrieve_dtdl};
use log::{debug, info, LevelFilter};
use proto::consumer::consumer_client::ConsumerClient;
use proto::consumer::PublishRequest;
use proto::digitaltwin::digital_twin_client::DigitalTwinClient;
use proto::digitaltwin::RegisterRequest;
use proto::provider::provider_server::ProviderServer;
use std::collections::HashSet;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex, MutexGuard};
use tokio::time::{sleep, Duration};
use tonic::transport::Server;

use crate::provider_impl::{ProviderImpl, SubscriptionMap};

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
                let lock: MutexGuard<SubscriptionMap> = subscription_map.lock().unwrap();
                let get_result = lock.get(sdv::vehicle::cabin::hvac::ambient_air_temperature::ID);
                urls = match get_result {
                    Some(val) => val.clone(),
                    None => HashSet::new(),
                };
            }

            for url in urls {
                info!("Publishing the ambient air temperature as {} to {}", temperature, &url);

                let client_result = ConsumerClient::connect(url).await;
                if client_result.is_err() {
                    continue;
                }
                let mut client = client_result.unwrap();

                let request = tonic::Request::new(PublishRequest {
                    entity_id: String::from(sdv::vehicle::cabin::hvac::ambient_air_temperature::ID),
                    value: temperature.to_string(),
                });

                let _response = client.publish(request).await;
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

    debug!("Preparing the Provider's DTDL.");
    let provider_dtdl_path = find_full_path("content/ambient_air_temperature.json")?;
    let dtdl = retrieve_dtdl(&provider_dtdl_path)?;
    debug!("Prepared the Provider's DTDL.");

    // Setup the HTTP server.
    let addr: SocketAddr = "[::1]:40010".parse()?;
    let subscription_map = Arc::new(Mutex::new(SubscriptionMap::new()));
    let provider_impl = ProviderImpl { subscription_map: subscription_map.clone() };
    let server_future =
        Server::builder().add_service(ProviderServer::new(provider_impl)).serve(addr);

    debug!("Registering the Provider's DTDL with the Digital Twin Service.");
    let mut client = DigitalTwinClient::connect("http://[::1]:50010").await?; // Devskim: ignore DS137138
    let request = tonic::Request::new(RegisterRequest { dtdl });
    let _response = client.register(request).await?;
    info!("The Provider's DTDL has been registered.");

    start_ambient_air_temperature_data_stream(subscription_map.clone());

    server_future.await?;

    debug!("The Provider has completed.");

    Ok(())
}
