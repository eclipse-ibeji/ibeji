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
    // Define a retry strategy.
    let retry_strategy = ExponentialBackoff::from_millis(100)
        .map(jitter) // add jitter to delays
        .take(10); // limit to 10 retries

    // Connect to the digital twin graph service.
    let client = Retry::spawn(retry_strategy.clone(), || async {
        DigitalTwinGraphClient::connect(invehicle_digital_twin_uri.clone()).await.map_err(
            |err_msg| {
                format!("Unable to connect to the digital twin graph service due to: {err_msg}")
            },
        )
    })
    .await?;

    // Find vehicle instances.
    let find_vehicle_response = Retry::spawn(retry_strategy.clone(), || async {
        // Make a local mutable copy for use only within this closure body.
        let mut client = client.clone();

        let request: FindRequest = FindRequest { model_id: sdv::vehicle::ID.to_string() };

        client
            .find(request)
            .await
            .map_err(|err_msg| format!("Unable to call find due to: {err_msg}"))
    })
    .await?;
    let find_vehicle_response_inner = find_vehicle_response.into_inner();

    if find_vehicle_response_inner.values.is_empty() {
        return Err("Unable to find vehicle instances".to_string());
    }

    // For now, we will just use the first vehicle instance.
    let vehicle: sdv::vehicle::TYPE =
        serde_json::from_str(&find_vehicle_response_inner.values[0]).unwrap();
    // info!("The vehicle is: {:?}", vehicle);
    info!("The vehicle's instance id is: {:?}", vehicle.instance_id);
    if vehicle.cabin.is_empty() {
        return Err("The vehicle does not have a cabin".to_string());
    }
    let cabin_instance_id = vehicle.cabin[0].instance_id.clone();
    info!("The cabin's instance id is: {:?}", cabin_instance_id);

    // Get the cabin instance.
    let get_cabin_response = Retry::spawn(retry_strategy.clone(), || async {
        // Make a local mutable copy for use only within this closure body.
        let mut client = client.clone();

        let request: GetRequest =
            GetRequest { instance_id: cabin_instance_id.clone(), member_path: "".to_string() };

        client
            .get(request.clone())
            .await
            .map_err(|err_msg| format!("Unable to call get due to: {err_msg}"))
    })
    .await?;
    let get_cabin_response_inner = get_cabin_response.into_inner();

    let mut front_left_seat_instance_id_option: Option<String> = None;
    let cabin: sdv::cabin::TYPE = serde_json::from_str(&get_cabin_response_inner.value).unwrap();
    // info!("The cabin is: {:?}", cabin);
    for seat_relationship in cabin.seat.iter() {
        if (seat_relationship.seat_row == 1)
            && (seat_relationship.seat_position == sdv::cabin::seat::SEAT_POSITION_TYPE::left)
        {
            info!("The front left seat's instance id is: {:?}", seat_relationship.instance_id);
            front_left_seat_instance_id_option = Some(seat_relationship.instance_id.clone());
        }
    }

    if front_left_seat_instance_id_option.is_none() {
        return Err("The front left seat is not found".to_string());
    }

    let front_left_seat_instance_id = front_left_seat_instance_id_option.unwrap();

    // Get the seat instance.
    let get_seat_response = Retry::spawn(retry_strategy.clone(), || async {
        // Make a local mutable copy for use only within this closure body.
        let mut client = client.clone();

        let request: GetRequest = GetRequest {
            instance_id: front_left_seat_instance_id.clone(),
            member_path: "".to_string(),
        };

        client
            .get(request.clone())
            .await
            .map_err(|err_msg| format!("Unable to call get due to: {err_msg}"))
    })
    .await?;
    let get_seat_response_inner = get_seat_response.into_inner();

    let seat: sdv::seat::TYPE = serde_json::from_str(&get_seat_response_inner.value).unwrap();
    // info!("The seat is: {:?}", seat);
    if seat.seat_massager.is_empty() {
        return Err("The seat does not have a seat massage".to_string());
    }
    let seat_massager_instance_id = seat.seat_massager[0].instance_id.clone();
    info!("The seat massager's instance id is: {:?}", seat_massager_instance_id);

    // Get the seat massager instance.
    let get_seat_massager_response = Retry::spawn(retry_strategy.clone(), || async {
        // Make a local mutable copy for use only within this closure body.
        let mut client = client.clone();

        let request: GetRequest = GetRequest {
            instance_id: seat_massager_instance_id.clone(),
            member_path: "".to_string(),
        };

        client
            .get(request.clone())
            .await
            .map_err(|err_msg| format!("Unable to call get due to: {err_msg}"))
    })
    .await?;
    let get_seat_massager_response_inner = get_seat_massager_response.into_inner();

    let seat_massager_json: serde_json::Value =
        serde_json::from_str(&get_seat_massager_response_inner.value).unwrap();

    if seat_massager_json["@type"] != sdv::premium_airbag_seat_massager::ID {
        return Err(format!(
            "The seat massager instance is not of the expected model, instead it is a {0}",
            seat_massager_json["@type"]
        ));
    }
    let seat_massager: sdv::premium_airbag_seat_massager::TYPE =
        serde_json::from_value(seat_massager_json.clone()).unwrap();

    info!("The seat massager is: {:?}", seat_massager);

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
