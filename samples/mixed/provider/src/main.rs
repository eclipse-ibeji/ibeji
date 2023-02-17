// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

mod provider_impl;
mod vehicle;

use dt_model_identifiers::sdv_v1 as sdv;
use env_logger::{Builder, Target};
use ibeji_common::{find_full_path, retrieve_dtdl};
use log::{debug, info, warn, LevelFilter};
use parking_lot::{Mutex, MutexGuard};
use proto::consumer::consumer_client::ConsumerClient;
use proto::consumer::PublishRequest;
use proto::digitaltwin::digital_twin_client::DigitalTwinClient;
use proto::digitaltwin::RegisterRequest;
use proto::provider::provider_server::ProviderServer;
use std::collections::HashSet;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tonic::transport::Server;

use crate::provider_impl::{ProviderImpl, SubscriptionMap};
use crate::vehicle::Vehicle;

async fn publish(subscription_map: Arc<Mutex<SubscriptionMap>>, entity_id: &str, value: &str) {
    let urls;

    // This block controls the lifetime of the lock.
    {
        let lock: MutexGuard<SubscriptionMap> = subscription_map.lock();
        let get_result = lock.get(entity_id);
        urls = match get_result {
            Some(val) => val.clone(),
            None => HashSet::new(),
        };
    }

    for url in urls {
        debug!("Publishing {entity_id} as {value} to {url}");

        let client_result = ConsumerClient::connect(url).await;
        if client_result.is_err() {
            warn!("Unable to connect. We will retry in a moment.");
            sleep(Duration::from_secs(1)).await;
            continue;
        }
        let mut client = client_result.unwrap();

        let request = tonic::Request::new(PublishRequest {
            entity_id: String::from(entity_id),
            value: String::from(value),
        });

        let response = client.publish(request).await;
        match response {
            Ok(_) => (),
            Err(status) => warn!("{status:?}"),
        }
    }
}

async fn start_vehicle_simulator(
    subscription_map: Arc<Mutex<SubscriptionMap>>,
    vehicle: Arc<Mutex<Vehicle>>,
) {
    debug!("Starting the Provider's vehicle simulator.");
    tokio::spawn(async move {
        loop {
            let ambient_air_temperature: i32;
            let is_air_conditioning_active: bool;
            let hybrid_battery_remaining: i32;

            // This block controls the lifetime of the lock.
            {
                let mut lock: MutexGuard<Vehicle> = vehicle.lock();

                lock.execute_epoch();

                // Make a copy of the property values that we will publish after the lock is released.
                ambient_air_temperature = lock.ambient_air_temperature;
                is_air_conditioning_active = lock.is_air_conditioning_active;
                hybrid_battery_remaining = lock.hybrid_battery_remaining;
            }

            info!("Ambient air temperature is {ambient_air_temperature}; Is air conditioning active is {is_air_conditioning_active}; Hybrid battery remaining is {hybrid_battery_remaining}");
            publish(
                subscription_map.clone(),
                sdv::vehicle::cabin::hvac::ambient_air_temperature::ID,
                &ambient_air_temperature.to_string(),
            )
            .await;
            publish(
                subscription_map.clone(),
                sdv::vehicle::cabin::hvac::is_air_conditioning_active::ID,
                &is_air_conditioning_active.to_string(),
            )
            .await;
            publish(
                subscription_map.clone(),
                sdv::vehicle::obd::hybrid_battery_remaining::ID,
                &hybrid_battery_remaining.to_string(),
            )
            .await;

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
    let provider_dtdl_path = find_full_path("content/mixed.json")?;
    let dtdl = retrieve_dtdl(&provider_dtdl_path)?;
    debug!("Prepared the Provider's DTDL.");

    // Setup the HTTP server.
    let addr: SocketAddr = "[::1]:40010".parse()?;
    let subscription_map = Arc::new(Mutex::new(SubscriptionMap::new()));
    let vehicle = Arc::new(Mutex::new(Vehicle::new()));
    let provider_impl =
        ProviderImpl { subscription_map: subscription_map.clone(), vehicle: vehicle.clone() };
    let server_future =
        Server::builder().add_service(ProviderServer::new(provider_impl)).serve(addr);

    debug!("Registering the Provider's DTDL with the In-Vehicle Digital Twin Service.");
    let mut client = DigitalTwinClient::connect("http://[::1]:50010").await?; // Devskim: ignore DS137138
    let request = tonic::Request::new(RegisterRequest { dtdl });
    let _response = client.register(request).await?;
    info!("The Provider's DTDL has been registered.");

    start_vehicle_simulator(subscription_map.clone(), vehicle).await;

    server_future.await?;

    info!("The Provider has completed.");

    Ok(())
}
