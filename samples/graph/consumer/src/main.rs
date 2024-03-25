// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use digital_twin_model::sdv_v1 as sdv;
use env_logger::{Builder, Target};
use log::{debug, info, LevelFilter};
use samples_common::consumer_config;
use samples_common::utils::retrieve_invehicle_digital_twin_uri;
use samples_protobuf_data_access::digital_twin_graph::v1::digital_twin_graph::digital_twin_graph_client::DigitalTwinGraphClient;
use samples_protobuf_data_access::digital_twin_graph::v1::digital_twin_graph::{FindRequest, GetRequest};
use tokio_retry::Retry;
use tokio_retry::strategy::{ExponentialBackoff, jitter};

/// Start the seat massage steps.
///
/// # Arguments
/// `invehicle_digital_twin_uri` - The in-vehicle digital twin uri.
async fn interact_with_digital_twin(invehicle_digital_twin_uri: String) -> Result<(), String> {
    let retry_strategy = ExponentialBackoff::from_millis(100)
        .map(jitter) // add jitter to delays
        .take(10); // limit to 10 retries

    let client_result = Retry::spawn(retry_strategy.clone(), || async {
        let client_result =
            DigitalTwinGraphClient::connect(invehicle_digital_twin_uri.clone()).await;
        if client_result.is_err() {
            return Err("Unable to connect.".to_string());
        }
        Ok(client_result.unwrap())
    })
    .await;

    if client_result.is_err() {
        return Err("Unable to connect".to_string());
    }

    let client = client_result.unwrap();

    let request: FindRequest = FindRequest { model_id: sdv::vehicle::ID.to_string() };

    let response_result = Retry::spawn(retry_strategy.clone(), || async {
        // Make a local mutable copy for use only within this cloure body.
        let mut client = client.clone();

        // Invoke find.
        let response_result = client.find(request.clone()).await;
        if response_result.is_err() {
            return Err("Unable to call find".to_string());
        }
        Ok(response_result.unwrap())
    })
    .await;

    if response_result.is_err() {
        return Err("Unable to call find".to_string());
    }

    let response = response_result.unwrap();

    let mut cabin_instance_id_opt = None;

    let response_inner = response.into_inner();
    for value in response_inner.values.iter() {
        let vehicle: sdv::vehicle::TYPE = serde_json::from_str(value).unwrap();
        info!("The value is: {:?}", vehicle);
        info!("The cabin's instance id is: {:?}", vehicle.cabin[0].instance_id);
        cabin_instance_id_opt = Some(vehicle.cabin[0].instance_id.clone());
    }

    if cabin_instance_id_opt.is_none() {
        return Err("Unable to find cabin instance id".to_string());
    }

    let cabin_instance_id = cabin_instance_id_opt.unwrap();

    let get_cabin_request: GetRequest =
        GetRequest { instance_id: cabin_instance_id, member_path: "".to_string() };

    let get_cabin_response_result = Retry::spawn(retry_strategy, || async {
        // Make a local mutable copy for use only within this cloure body.
        let mut client = client.clone();

        // Invoke find.
        let get_cabin_response_result = client.get(get_cabin_request.clone()).await;
        if get_cabin_response_result.is_err() {
            return Err("Unable to call get".to_string());
        }
        Ok(get_cabin_response_result.unwrap())
    })
    .await;

    if get_cabin_response_result.is_err() {
        return Err("Unable to call get".to_string());
    }

    let get_cabin_response = get_cabin_response_result.unwrap();

    let get_cabin_response_inner = get_cabin_response.into_inner();
    let cabin: sdv::cabin::TYPE = serde_json::from_str(&get_cabin_response_inner.value).unwrap();
    info!("The value is: {:?}", cabin);
    for seat in cabin.seat.iter() {
        info!("The seat's instance id is: {:?}", seat.instance_id);
    }

    Ok(())
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

    interact_with_digital_twin(invehicle_digital_twin_uri).await?;

    debug!("The Consumer has completed.");

    Ok(())
}
