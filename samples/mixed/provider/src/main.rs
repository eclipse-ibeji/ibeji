// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

mod provider_impl;
mod vehicle;

use digital_twin_model::sdv_v1 as sdv;
use env_logger::{Builder, Target};
use log::{debug, info, warn, LevelFilter};
use parking_lot::{Mutex, MutexGuard};
use samples_common::constants::{digital_twin_operation, digital_twin_protocol};
use samples_protobuf_data_access::digital_twin::v1::digital_twin_client::DigitalTwinClient;
use samples_protobuf_data_access::digital_twin::v1::{EndpointInfo, EntityAccessInfo, RegisterRequest};
use samples_protobuf_data_access::sample_grpc::v1::digital_twin_consumer::digital_twin_consumer_client::DigitalTwinConsumerClient;
use samples_protobuf_data_access::sample_grpc::v1::digital_twin_consumer::PublishRequest;
use samples_protobuf_data_access::sample_grpc::v1::digital_twin_provider::digital_twin_provider_server::DigitalTwinProviderServer;
use std::collections::HashSet;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tonic::transport::Server;

use crate::provider_impl::{ProviderImpl, SubscriptionMap};
use crate::vehicle::Vehicle;

const IN_VEHICLE_DIGITAL_TWIN_SERVICE_URI: &str = "http://[::1]:50010"; // Devskim: ignore DS137138
const PROVIDER_AUTHORITY: &str = "[::1]:40010";

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
        debug!(
            "Sending a publish request for {entity_id} with value {value} to consumer URI {url}"
        );

        let client_result = DigitalTwinConsumerClient::connect(url).await;
        if client_result.is_err() {
            warn!("Unable to connect. We will retry in a moment.");
            sleep(Duration::from_secs(1)).await;
            continue;
        }
        let mut client = client_result.unwrap();

        let request = tonic::Request::new(PublishRequest {
            entity_id: entity_id.to_string(),
            value: value.to_string(),
        });

        let response = client.publish(request).await;
        match response {
            Ok(_) => (),
            Err(status) => warn!("{status:?}"),
        }

        debug!("Completed the publish request");
    }
}

async fn start_vehicle_simulator(
    subscription_map: Arc<Mutex<SubscriptionMap>>,
    vehicle: Arc<Mutex<Vehicle>>,
) {
    info!("Starting the Provider's vehicle simulator.");
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

            info!("Publishing the values: Ambient air temperature is {ambient_air_temperature}; Is air conditioning active is {is_air_conditioning_active}; Hybrid battery remaining is {hybrid_battery_remaining}");
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
// #[allow(clippy::collapsible_else_if)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup logging.
    Builder::new().filter(None, LevelFilter::Info).target(Target::Stdout).init();

    info!("The Provider has started.");

    // AmbientAirTemperature
    let ambient_air_temperature_endpoint_info = EndpointInfo {
        protocol: digital_twin_protocol::GRPC.to_string(),
        operations: vec![
            digital_twin_operation::SUBSCRIBE.to_string(),
            digital_twin_operation::UNSUBSCRIBE.to_string(),
        ],
        uri: "http://[::1]:40010".to_string(), // Devskim: ignore DS137138
        context: sdv::vehicle::cabin::hvac::ambient_air_temperature::ID.to_string(),
    };
    let ambient_air_temperature_access_info = EntityAccessInfo {
        name: "AmbientAirTemperature".to_string(),
        id: sdv::vehicle::cabin::hvac::ambient_air_temperature::ID.to_string(),
        description: "The immediate surroundings air temperature (in Fahrenheit).".to_string(),
        endpoint_info_list: vec![ambient_air_temperature_endpoint_info],
    };

    // IsAirConditioningActive
    let is_air_conditioning_active_endpoint_info = EndpointInfo {
        protocol: digital_twin_protocol::GRPC.to_string(),
        operations: vec![
            digital_twin_operation::SUBSCRIBE.to_string(),
            digital_twin_operation::UNSUBSCRIBE.to_string(),
            digital_twin_operation::SET.to_string(),
        ],
        uri: "http://[::1]:40010".to_string(), // Devskim: ignore DS137138
        context: sdv::vehicle::cabin::hvac::ambient_air_temperature::ID.to_string(),
    };
    let is_air_conditioning_active_access_info = EntityAccessInfo {
        name: "IsAirConditioningActive".to_string(),
        id: sdv::vehicle::cabin::hvac::is_air_conditioning_active::ID.to_string(),
        description: "Is air conditioning active?".to_string(),
        endpoint_info_list: vec![is_air_conditioning_active_endpoint_info],
    };

    // HybridBatteryRemaining
    let hybrid_battery_remaining_endpoint_info = EndpointInfo {
        protocol: digital_twin_protocol::GRPC.to_string(),
        operations: vec![
            digital_twin_operation::SUBSCRIBE.to_string(),
            digital_twin_operation::UNSUBSCRIBE.to_string(),
        ],
        uri: "http://[::1]:40010".to_string(), // Devskim: ignore DS137138
        context: sdv::vehicle::obd::hybrid_battery_remaining::ID.to_string(),
    };
    let hybrid_battery_remaining_access_info = EntityAccessInfo {
        name: "HybridBatteryRemaining".to_string(),
        id: sdv::vehicle::obd::hybrid_battery_remaining::ID.to_string(),
        description: "The remaining hybrid battery life.".to_string(),
        endpoint_info_list: vec![hybrid_battery_remaining_endpoint_info],
    };

    // ShowNotification
    let show_notification_endpoint_info = EndpointInfo {
        protocol: digital_twin_protocol::GRPC.to_string(),
        operations: vec![digital_twin_operation::INVOKE.to_string()],
        uri: "http://[::1]:40010".to_string(), // Devskim: ignore DS137138
        context: sdv::vehicle::cabin::infotainment::hmi::show_notification::ID.to_string(),
    };
    let show_notification_access_info = EntityAccessInfo {
        name: "ShowNotification".to_string(),
        id: sdv::vehicle::cabin::infotainment::hmi::show_notification::ID.to_string(),
        description: "Show a notification on the HMI.".to_string(),
        endpoint_info_list: vec![show_notification_endpoint_info],
    };

    let entity_access_info_list = vec![
        ambient_air_temperature_access_info,
        is_air_conditioning_active_access_info,
        hybrid_battery_remaining_access_info,
        show_notification_access_info,
    ];

    // Setup the HTTP server.
    let addr: SocketAddr = PROVIDER_AUTHORITY.parse()?;
    let subscription_map = Arc::new(Mutex::new(SubscriptionMap::new()));
    let vehicle = Arc::new(Mutex::new(Vehicle::new()));
    let provider_impl =
        ProviderImpl { subscription_map: subscription_map.clone(), vehicle: vehicle.clone() };
    let server_future =
        Server::builder().add_service(DigitalTwinProviderServer::new(provider_impl)).serve(addr);
    info!("The HTTP server is listening on address '{PROVIDER_AUTHORITY}'");

    info!("Sending a register request with the Provider's entity access info to the In-Vehicle Digital Twin Service URI {IN_VEHICLE_DIGITAL_TWIN_SERVICE_URI}");
    let mut client = DigitalTwinClient::connect(IN_VEHICLE_DIGITAL_TWIN_SERVICE_URI).await?;
    let request = tonic::Request::new(RegisterRequest { entity_access_info_list });
    let _response = client.register(request).await?;
    debug!("The Provider's entity access info has been registered.");

    start_vehicle_simulator(subscription_map.clone(), vehicle).await;

    server_future.await?;

    info!("The Provider has completed.");

    Ok(())
}
