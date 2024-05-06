// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use digital_twin_model::sdv_v1 as sdv;
use env_logger::{Builder, Target};
use log::{info, LevelFilter};
use rand::rngs::StdRng;
use rand::Rng; // trait needed for gen_range
use rand::SeedableRng; // trait needed to initialize StdRng
use samples_common::consumer_config;
use samples_common::utils::retrieve_invehicle_digital_twin_uri;
use samples_protobuf_data_access::digital_twin_graph::v1::digital_twin_graph::digital_twin_graph_client::DigitalTwinGraphClient;
use samples_protobuf_data_access::digital_twin_graph::v1::digital_twin_graph::{FindRequest, FindResponse, GetRequest, GetResponse, InvokeRequest, InvokeResponse};
use tokio_retry::Retry;
use tokio_retry::strategy::{ExponentialBackoff, jitter};

// The base duration in milliseconds for the exponential backoff strategy.
const BACKOFF_BASE_DURATION_IN_MILLIS: u64 = 100;

// The maximum number of retries for the exponential backoff strategy.
const MAX_RETRIES: usize = 100;

/// Connect to the digital twin graph service.
///
/// # Arguments
/// * `invehicle_digital_twin_uri` - The in-vehicle digital twin uri.
/// # Returns
/// The digital twin graph client.
async fn connect_to_digital_twin_graph_service(
    invehicle_digital_twin_uri: String,
) -> Result<DigitalTwinGraphClient<tonic::transport::Channel>, String> {
    let retry_strategy = ExponentialBackoff::from_millis(BACKOFF_BASE_DURATION_IN_MILLIS)
        .map(jitter) // add jitter to delays
        .take(MAX_RETRIES);

    let client: DigitalTwinGraphClient<tonic::transport::Channel> =
        Retry::spawn(retry_strategy.clone(), || async {
            DigitalTwinGraphClient::connect(invehicle_digital_twin_uri.clone()).await.map_err(
                |err_msg| {
                    format!("Unable to connect to the digital twin graph service due to: {err_msg}")
                },
            )
        })
        .await?;

    Ok(client)
}

/// Find all instances of a model.
///
/// # Arguments
/// * `client` - The digital twin graph client.
/// * `model_id` - The model id.
/// # Returns
/// The find response.
async fn find(
    client: DigitalTwinGraphClient<tonic::transport::Channel>,
    model_id: String,
) -> Result<FindResponse, String> {
    let retry_strategy = ExponentialBackoff::from_millis(BACKOFF_BASE_DURATION_IN_MILLIS)
        .map(jitter) // add jitter to delays
        .take(MAX_RETRIES);

    let request = FindRequest { model_id: model_id.clone() };

    let find_vehicle_response = Retry::spawn(retry_strategy.clone(), || async {
        let mut client = client.clone();
        client
            .find(request.clone())
            .await
            .map_err(|err_msg| format!("Unable to call find the instances due to: {err_msg}"))
    })
    .await?
    .into_inner();

    Ok(find_vehicle_response)
}

/// Get an instance.
///
/// # Arguments
/// * `client` - The digital twin graph client.
/// * `instance_id` - The instance id.
/// * `member_path` - The member path.
/// # Returns
/// The get response.
async fn get(
    client: DigitalTwinGraphClient<tonic::transport::Channel>,
    instance_id: String,
    member_path: String,
) -> Result<GetResponse, String> {
    let retry_strategy = ExponentialBackoff::from_millis(BACKOFF_BASE_DURATION_IN_MILLIS)
        .map(jitter) // add jitter to delays
        .take(MAX_RETRIES);

    let request = GetRequest { instance_id: instance_id.clone(), member_path: member_path.clone() };

    let get_response = Retry::spawn(retry_strategy.clone(), || async {
        let mut client = client.clone();

        client
            .get(request.clone())
            .await
            .map_err(|err_msg| format!("Unable to get the instance due to: {err_msg}"))
    })
    .await?
    .into_inner();

    Ok(get_response)
}

/// Invoke an instance's operation.
///
/// # Arguments
/// * `client` - The digital twin graph client.
/// * `instance_id` - The instance id.
/// * `member_path` - The member path.
/// * `request_payload` - The request payload.
/// # Returns
/// The invoke response.
async fn invoke(
    client: DigitalTwinGraphClient<tonic::transport::Channel>,
    instance_id: String,
    member_path: String,
    request_payload: String,
) -> Result<InvokeResponse, String> {
    let mut client = client.clone();

    let request = InvokeRequest {
        instance_id: instance_id.clone(),
        member_path: member_path.clone(),
        request_payload: request_payload.clone(),
    };

    let invoke_response = client
        .invoke(request.clone())
        .await
        .map_err(|err_msg| format!("Unable to invoke the instance's operation due to: {err_msg}"))?
        .into_inner();

    Ok(invoke_response)
}

/// Find a vehicle instance.
///
/// # Arguments
/// * `client` - The digital twin graph client.
/// # Returns
/// The vehicle instance.
async fn find_vehicle(
    client: DigitalTwinGraphClient<tonic::transport::Channel>,
) -> Result<sdv::vehicle::TYPE, String> {
    // Find all vehicle instances.
    let find_vehicle_response: FindResponse = find(client, sdv::vehicle::ID.to_string()).await?;
    if find_vehicle_response.values.is_empty() {
        return Err("Unable to find vehicle instances".to_string());
    }

    // For now, we will just use the first vehicle instance.
    let vehicle: sdv::vehicle::TYPE =
        serde_json::from_str(&find_vehicle_response.values[0]).unwrap();

    info!("The vehicle's instance id is: {}", vehicle.instance_id);

    Ok(vehicle)
}

/// Find a cabin instance.
///
/// # Arguments
/// * `client` - The digital twin graph client.
/// * `vehicle` - The vehicle instance.
/// # Returns
/// The cabin instance.
async fn find_cabin(
    client: DigitalTwinGraphClient<tonic::transport::Channel>,
    vehicle: &sdv::vehicle::TYPE,
) -> Result<sdv::cabin::TYPE, String> {
    // Get the cabin instance id.
    if vehicle.cabin.is_empty() {
        return Err("The vehicle does not have a cabin".to_string());
    }

    // A vehicle has at most one cabin instance. We will use the first cabin instance.
    let cabin_instance_id = vehicle.cabin[0].instance_id.clone();

    info!("The cabin's instance id is: {:?}", cabin_instance_id);

    // Get the cabin instance.
    let get_cabin_response: GetResponse =
        get(client.clone(), cabin_instance_id.clone(), "".to_string()).await?;

    // Deserialize the cabin instance.
    let cabin: sdv::cabin::TYPE = serde_json::from_str(&get_cabin_response.value).unwrap();

    Ok(cabin)
}

/// Find a seat instance.
///
/// # Arguments
/// * `client` - The digital twin graph client.
/// * `cabin` - The cabin instance.
/// * `seat_row` - The seat row.
/// * `seat_posotion` - The seat position.
/// # Returns
/// The seat instance.
async fn find_seat(
    client: DigitalTwinGraphClient<tonic::transport::Channel>,
    cabin: &sdv::cabin::TYPE,
    seat_row: i32,
    seat_posotion: sdv::cabin::seat::SEAT_POSITION_TYPE,
) -> Result<sdv::seat::TYPE, String> {
    if cabin.seat.is_empty() {
        return Err("The cabin does not have any seats".to_string());
    }

    // Find the specified seat instance.
    for seat_relationship in cabin.seat.iter() {
        if (seat_relationship.seat_row == seat_row)
            && (seat_relationship.seat_position == seat_posotion)
        {
            // Get the seat instance.
            let get_seat_response: GetResponse =
                get(client.clone(), seat_relationship.instance_id.clone(), "".to_string()).await?;

            // Deserialize the seat instance.
            let seat: sdv::seat::TYPE = serde_json::from_str(&get_seat_response.value).unwrap();

            info!("The seat's instance id is: {}", seat.instance_id);

            return Ok(seat);
        }
    }

    Err("The seat was not found".to_string())
}

/// Find a premium airbag seat massager instance.
///
/// # Arguments
/// * `client` - The digital twin graph client.
/// * `seat` - The seat instance.
/// # Returns
/// The premium airbag seat massager instance.
async fn find_premium_airbag_seat_massager(
    client: DigitalTwinGraphClient<tonic::transport::Channel>,
    seat: &sdv::seat::TYPE,
) -> Result<sdv::premium_airbag_seat_massager::TYPE, String> {
    if seat.seat_massager.is_empty() {
        return Err("The seat does not have a seat massage".to_string());
    }

    // Get the seat massager instance id.
    let seat_massager_instance_id = seat.seat_massager[0].instance_id.clone();

    // Get the seat massager instance.
    let get_seat_massager_response: GetResponse =
        get(client.clone(), seat_massager_instance_id.clone(), "".to_string()).await?;

    // Deserialize the seat massager instance to a JSON object.
    let seat_massager_json: serde_json::Value =
        serde_json::from_str(&get_seat_massager_response.value).unwrap();

    // Check that that the seat massager's modei_id (marked by @type) is the expected model (premium_airbag_seat_massager).
    if seat_massager_json["@type"] != sdv::premium_airbag_seat_massager::ID {
        return Err(format!(
            "The seat massager instance is not of the expected model, instead it is a {0}",
            seat_massager_json["@type"]
        ));
    }

    // Deserialize the seat massager instance.
    let seat_massager: sdv::premium_airbag_seat_massager::TYPE =
        serde_json::from_str(&get_seat_massager_response.value).unwrap();

    info!("The seat massager's instance id is: {}", seat_massager.instance_id);

    Ok(seat_massager)
}

/// Perform the perform_step operation.
///
/// # Arguments
/// * `client` - The digital twin graph client.
/// * `seat_massager` - The premium airbag seat massager instance.
/// * `airbag_identifier` - The airbag identifier.
/// * `inflation_level` - The inflation level.
/// * `inflation_duration_in_seconds` - The inflation duration in seconds.
/// # Returns
/// An empty result if the operation is successful.
async fn perform_step(
    client: DigitalTwinGraphClient<tonic::transport::Channel>,
    seat_massager: sdv::premium_airbag_seat_massager::TYPE,
    airbag_identifier: i32,
    inflation_level: i32,
    inflation_duration_in_seconds: i32,
) -> Result<(), String> {
    // Generate the perform_step operation's request payload.
    let request_payload: sdv::airbag_seat_massager::perform_step::request::TYPE =
        sdv::airbag_seat_massager::perform_step::request::TYPE {
            step: vec![sdv::airbag_seat_massager::airbag_adjustment::TYPE {
                airbag_identifier,
                inflation_level,
                inflation_duration_in_seconds,
            }],
            ..Default::default()
        };

    // Serialize the request payload to a JSON string.
    let request_payload_json: String = serde_json::to_string_pretty(&request_payload).unwrap();

    // Invoke the perform_step operation.
    let perform_step_response: InvokeResponse = invoke(
        client.clone(),
        seat_massager.instance_id.clone(),
        sdv::airbag_seat_massager::perform_step::NAME.to_string(),
        request_payload_json.clone(),
    )
    .await?;

    info!("The perform_step operation response is:\n{}", perform_step_response.response_payload);

    Ok(())
}

/// Perform a series of interactions with a vehicle digital twin.
///
/// # Arguments
/// * `invehicle_digital_twin_uri` - The in-vehicle digital twin uri.
/// # Returns
/// An empty result if the interactions are successful.
async fn interact_with_digital_twin(invehicle_digital_twin_uri: String) -> Result<(), String> {
    // Connect to the digital twin graph service.
    let client: DigitalTwinGraphClient<tonic::transport::Channel> =
        connect_to_digital_twin_graph_service(invehicle_digital_twin_uri.clone()).await?;

    // Find the vehicle instance.
    let vehicle: sdv::vehicle::TYPE = find_vehicle(client.clone()).await.unwrap();

    // Find the cabin instance.
    let cabin: sdv::cabin::TYPE = find_cabin(client.clone(), &vehicle).await.unwrap();

    // Find the front left seat instance.
    let front_left_seat: sdv::seat::TYPE =
        find_seat(client.clone(), &cabin, 1, sdv::cabin::seat::SEAT_POSITION_TYPE::left)
            .await
            .unwrap();

    // Find the premium airbag seat massager instance.
    let seat_massager: sdv::premium_airbag_seat_massager::TYPE =
        find_premium_airbag_seat_massager(client.clone(), &front_left_seat).await.unwrap();

    // Randomly generate the airbag adjustment field values.
    let mut rng = StdRng::from_entropy();
    let airbag_identifier = rng.gen_range(1..=15);
    let inflation_level = rng.gen_range(1..=10);
    let inflation_duration_in_seconds = rng.gen_range(1..=5);

    // Perform the perform_step operation.
    perform_step(
        client.clone(),
        seat_massager,
        airbag_identifier,
        inflation_level,
        inflation_duration_in_seconds,
    )
    .await?;

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

    info!("The Consumer has completed.");

    Ok(())
}
