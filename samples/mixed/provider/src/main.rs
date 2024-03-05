// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

mod provider_impl;
mod vehicle;

use digital_twin_model::{sdv_v0 as sdv, Metadata};
use env_logger::{Builder, Target};
use log::{debug, info, warn, LevelFilter};
use parking_lot::{Mutex, MutexGuard};
use samples_common::constants::{digital_twin_operation, digital_twin_protocol};
use samples_common::utils::{retrieve_invehicle_digital_twin_uri, retry_async_based_on_status};
use samples_common::provider_config;
use samples_protobuf_data_access::invehicle_digital_twin::v1::invehicle_digital_twin_client::InvehicleDigitalTwinClient;
use samples_protobuf_data_access::invehicle_digital_twin::v1::{EndpointInfo, EntityAccessInfo, RegisterRequest};
use samples_protobuf_data_access::sample_grpc::v1::digital_twin_consumer::digital_twin_consumer_client::DigitalTwinConsumerClient;
use samples_protobuf_data_access::sample_grpc::v1::digital_twin_consumer::PublishRequest;
use samples_protobuf_data_access::sample_grpc::v1::digital_twin_provider::digital_twin_provider_server::DigitalTwinProviderServer;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashSet;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tonic::{Status, transport::Server};

use crate::provider_impl::{ProviderImpl, SubscriptionMap};
use crate::vehicle::Vehicle;

#[derive(Debug, Serialize, Deserialize)]
struct AmbientAirTemperatureProperty {
    #[serde(rename = "AmbientAirTemperature")]
    ambient_air_temperature: sdv::hvac::ambient_air_temperature::TYPE,
    #[serde(rename = "$metadata")]
    metadata: Metadata,
}

#[derive(Debug, Serialize, Deserialize)]
struct HybridBatteryRemainingProperty {
    #[serde(rename = "HybridBatteryRemainaing")]
    hybrid_battery_remaining: sdv::obd::hybrid_battery_remaining::TYPE,
    #[serde(rename = "$metadata")]
    metadata: Metadata,
}

#[derive(Debug, Serialize, Deserialize)]
struct IsAirConditioingActiveProperty {
    #[serde(rename = "IsAirConditioingActive")]
    is_air_conditioning_active: sdv::hvac::is_air_conditioning_active::TYPE,
    #[serde(rename = "$metadata")]
    metadata: Metadata,
}

/// Register the entities endpoints.
///
/// # Arguments
/// * `invehicle_digital_twin_uri` - The In-Vehicle Digital Twin URI.
/// * `provider_uri` - The provider's URI.
async fn register_entities(
    invehicle_digital_twin_uri: &str,
    provider_uri: &str,
) -> Result<(), Status> {
    // AmbientAirTemperature
    let ambient_air_temperature_endpoint_info = EndpointInfo {
        protocol: digital_twin_protocol::GRPC.to_string(),
        operations: vec![digital_twin_operation::SUBSCRIBE.to_string()],
        uri: provider_uri.to_string(),
        context: sdv::hvac::ambient_air_temperature::ID.to_string(),
    };
    let ambient_air_temperature_access_info = EntityAccessInfo {
        name: sdv::hvac::ambient_air_temperature::NAME.to_string(),
        id: sdv::hvac::ambient_air_temperature::ID.to_string(),
        description: sdv::hvac::ambient_air_temperature::DESCRIPTION.to_string(),
        endpoint_info_list: vec![ambient_air_temperature_endpoint_info],
    };

    // IsAirConditioningActive
    let is_air_conditioning_active_endpoint_info = EndpointInfo {
        protocol: digital_twin_protocol::GRPC.to_string(),
        operations: vec![
            digital_twin_operation::SUBSCRIBE.to_string(),
            digital_twin_operation::SET.to_string(),
        ],
        uri: provider_uri.to_string(),
        context: sdv::hvac::is_air_conditioning_active::ID.to_string(),
    };
    let is_air_conditioning_active_access_info = EntityAccessInfo {
        name: sdv::hvac::is_air_conditioning_active::NAME.to_string(),
        id: sdv::hvac::is_air_conditioning_active::ID.to_string(),
        description: sdv::hvac::is_air_conditioning_active::DESCRIPTION.to_string(),
        endpoint_info_list: vec![is_air_conditioning_active_endpoint_info],
    };

    // HybridBatteryRemaining
    let hybrid_battery_remaining_endpoint_info = EndpointInfo {
        protocol: digital_twin_protocol::GRPC.to_string(),
        operations: vec![digital_twin_operation::SUBSCRIBE.to_string()],
        uri: provider_uri.to_string(),
        context: sdv::obd::hybrid_battery_remaining::ID.to_string(),
    };
    let hybrid_battery_remaining_access_info = EntityAccessInfo {
        name: sdv::obd::hybrid_battery_remaining::NAME.to_string(),
        id: sdv::obd::hybrid_battery_remaining::ID.to_string(),
        description: sdv::obd::hybrid_battery_remaining::DESCRIPTION.to_string(),
        endpoint_info_list: vec![hybrid_battery_remaining_endpoint_info],
    };

    // ShowNotification
    let show_notification_endpoint_info = EndpointInfo {
        protocol: digital_twin_protocol::GRPC.to_string(),
        operations: vec![digital_twin_operation::INVOKE.to_string()],
        uri: provider_uri.to_string(),
        context: sdv::hmi::show_notification::ID.to_string(),
    };
    let show_notification_access_info = EntityAccessInfo {
        name: sdv::hmi::show_notification::NAME.to_string(),
        id: sdv::hmi::show_notification::ID.to_string(),
        description: sdv::hmi::show_notification::DESCRIPTION.to_string(),
        endpoint_info_list: vec![show_notification_endpoint_info],
    };

    let entity_access_info_list = vec![
        ambient_air_temperature_access_info,
        is_air_conditioning_active_access_info,
        hybrid_battery_remaining_access_info,
        show_notification_access_info,
    ];

    let mut client = InvehicleDigitalTwinClient::connect(invehicle_digital_twin_uri.to_string())
        .await
        .map_err(|e| Status::internal(e.to_string()))?;
    let request = tonic::Request::new(RegisterRequest { entity_access_info_list });
    let _response = client.register(request).await?;

    Ok(())
}

/// Publish.
///
/// # Arguments
/// * `subscription_map` - Subscription map.
/// * `entity_id` - Entity id.
/// * `value` - The value to publish.
async fn publish(subscription_map: Arc<Mutex<SubscriptionMap>>, entity_id: &str, value: &str) {
    // This block controls the lifetime of the lock.
    let uris = {
        let lock: MutexGuard<SubscriptionMap> = subscription_map.lock();
        let get_result = lock.get(entity_id);
        match get_result {
            Some(val) => val.clone(),
            None => HashSet::new(),
        }
    };

    for uri in uris {
        debug!(
            "Sending a publish request for {entity_id} with value {value} to consumer URI {uri}"
        );

        let client_result = DigitalTwinConsumerClient::connect(uri).await;
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

/// Starts the vehicle simulator.
///
/// # Arguments
/// * `subscription_map` - Subscription map.
/// * `vehicle` - A vehicle struct that emulates the dynamic changes of in-vehicle signals.
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
            let ambient_air_temperature_property: AmbientAirTemperatureProperty =
                AmbientAirTemperatureProperty {
                    ambient_air_temperature,
                    metadata: Metadata {
                        model: sdv::hvac::ambient_air_temperature::ID.to_string(),
                    },
                };
            publish(
                subscription_map.clone(),
                sdv::hvac::ambient_air_temperature::ID,
                &serde_json::to_string(&ambient_air_temperature_property).unwrap(),
            )
            .await;
            let is_air_conditioning_active_property: IsAirConditioingActiveProperty =
                IsAirConditioingActiveProperty {
                    is_air_conditioning_active,
                    metadata: Metadata {
                        model: sdv::hvac::is_air_conditioning_active::ID.to_string(),
                    },
                };
            publish(
                subscription_map.clone(),
                sdv::hvac::is_air_conditioning_active::ID,
                &serde_json::to_string(&is_air_conditioning_active_property).unwrap(),
            )
            .await;
            let hybrid_battery_remaining_property: HybridBatteryRemainingProperty =
                HybridBatteryRemainingProperty {
                    hybrid_battery_remaining,
                    metadata: Metadata {
                        model: sdv::obd::hybrid_battery_remaining::ID.to_string(),
                    },
                };
            publish(
                subscription_map.clone(),
                sdv::obd::hybrid_battery_remaining::ID,
                &serde_json::to_string(&hybrid_battery_remaining_property).unwrap(),
            )
            .await;

            sleep(Duration::from_secs(5)).await;
        }
    });
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

    // Construct the provider URI from the provider authority.
    let provider_uri = format!("http://{provider_authority}"); // Devskim: ignore DS137138

    // Setup the HTTP server.
    let addr: SocketAddr = provider_authority.parse()?;
    let subscription_map = Arc::new(Mutex::new(SubscriptionMap::new()));
    let vehicle = Arc::new(Mutex::new(Vehicle::new()));
    let provider_impl =
        ProviderImpl { subscription_map: subscription_map.clone(), vehicle: vehicle.clone() };
    let server_future =
        Server::builder().add_service(DigitalTwinProviderServer::new(provider_impl)).serve(addr);
    info!("The HTTP server is listening on address '{provider_authority}'");

    info!("Sending a register request to the In-Vehicle Digital Twin Service URI {invehicle_digital_twin_uri}");
    retry_async_based_on_status(30, Duration::from_secs(1), || {
        register_entities(&invehicle_digital_twin_uri, &provider_uri)
    })
    .await?;

    start_vehicle_simulator(subscription_map.clone(), vehicle).await;

    server_future.await?;

    info!("The Provider has completed.");

    Ok(())
}
