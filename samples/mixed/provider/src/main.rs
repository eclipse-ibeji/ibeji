// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

mod provider_impl;
mod vehicle;

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
use crate::vehicle::Vehicle;

/// The ids for the properties.
const AMBIENT_AIR_TEMPERATURE_PROPERTY_ID: &str =
    "dtmi:org:eclipse:sdv:vehicle:cabin:hvac:ambient_air_temperature;1";
const IS_AIR_CONDITIONING_ACTIVE: &str =
    "dtmi:org:eclipse:sdv:vehicle:cabin:hvac:is_air_conditioning_active;1";
const HYBRID_BATTERY_REMAINING: &str =
    "dtmi:org:eclipse:sdv:vehcile:obd:hybrid_battery_remaining;1";

async fn publish(subscription_map: Arc<Mutex<SubscriptionMap>>, entity_id: &str, value: &str) {
    let urls;
    {
        let lock: MutexGuard<SubscriptionMap> = subscription_map.lock().unwrap();
        let get_result = lock.get(entity_id);
        urls = match get_result {
            Some(val) => val.clone(),
            None => HashSet::new(),
        };
    }

    for url in urls {
        debug!("Publishing {} as {} to {}", entity_id, value, &url);

        let client_result = ConsumerClient::connect(url).await;
        if client_result.is_err() {
            continue;
        }
        let mut client = client_result.unwrap();

        let request = tonic::Request::new(PublishRequest {
            entity_id: String::from(entity_id),
            value: String::from(value),
        });

        let _response = client.publish(request).await;
    }
}

async fn start_vehicle_simulator(
    subscription_map: Arc<Mutex<SubscriptionMap>>,
    vehicle: Arc<Mutex<Vehicle>>,
) {
    info!("Starting the Provider's veicle simulator.");
    tokio::spawn(async move {
        loop {
            let ambient_air_temperature: u32;
            let is_air_conditioning_active: bool;
            let hybrid_battery_remaining: f32;
            let ui_message: String;

            {
                let mut lock: MutexGuard<Vehicle> = vehicle.lock().unwrap();

                lock.execute_epoch();

                // Make a copy of the peoprtes values that we will publish after the lock is released.
                ambient_air_temperature = lock.ambient_air_temperature;
                is_air_conditioning_active = lock.is_air_conditioning_active;
                hybrid_battery_remaining = lock.hybrid_battery_remaining;
                ui_message = lock.ui_message.clone();
            }

            info!("Ambient air temperature is {ambient_air_temperature}; Is air conditioning active is {is_air_conditioning_active}; Hybrid battery remaining is {hybrid_battery_remaining}; UI message is '{ui_message}'");
            publish(
                subscription_map.clone(),
                AMBIENT_AIR_TEMPERATURE_PROPERTY_ID,
                &ambient_air_temperature.to_string(),
            )
            .await;
            publish(
                subscription_map.clone(),
                IS_AIR_CONDITIONING_ACTIVE,
                &is_air_conditioning_active.to_string(),
            )
            .await;
            publish(
                subscription_map.clone(),
                HYBRID_BATTERY_REMAINING,
                &hybrid_battery_remaining.to_string(),
            )
            .await;

            sleep(Duration::from_millis(1000)).await;
        }
    });
}

#[tokio::main]
#[allow(clippy::collapsible_else_if)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup logging.
    Builder::new().filter(None, LevelFilter::Info).target(Target::Stdout).init();

    info!("The Provider has started.");

    info!("Preparing the Provider's DTDL.");
    let provider_dtdl_path = find_full_path("content/mixed.json")?;
    let dtdl = retrieve_dtdl(&provider_dtdl_path)?;
    info!("Prepared the Provider's DTDL.");

    // Setup the HTTP server.
    let addr: SocketAddr = "[::1]:40010".parse()?;
    let subscription_map = Arc::new(Mutex::new(SubscriptionMap::new()));
    let vehicle = Arc::new(Mutex::new(Vehicle::new()));
    let provider_impl =
        ProviderImpl { subscription_map: subscription_map.clone(), vehicle: vehicle.clone() };
    let server_future =
        Server::builder().add_service(ProviderServer::new(provider_impl)).serve(addr);

    info!("Registering the Provider's DTDL with the Digital Twin Service.");
    let mut client = DigitalTwinClient::connect("http://[::1]:50010").await?; // Devskim: ignore DS137138
    let request = tonic::Request::new(RegisterRequest { dtdl });
    let _response = client.register(request).await?;
    info!("The Provider's DTDL has been registered.");

    start_vehicle_simulator(subscription_map.clone(), vehicle).await;

    server_future.await?;

    info!("The Provider has completed.");

    Ok(())
}
