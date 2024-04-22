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

const BACKOFF_BASE_DURATION_IN_MILLIS: u64 = 100;
const MAX_RETRIES: usize = 10;

/// Connect to the digital twin graph service.
///
/// # Arguments
/// * `invehicle_digital_twin_uri` - The in-vehicle digital twin uri.
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

/// Perform a series of interactions with a vehicle digital twin.
///
/// # Arguments
/// * `invehicle_digital_twin_uri` - The in-vehicle digital twin uri.
async fn interact_with_digital_twin(invehicle_digital_twin_uri: String) -> Result<(), String> {
    // Connect to the digital twin graph service.
    let client: DigitalTwinGraphClient<tonic::transport::Channel> =
        connect_to_digital_twin_graph_service(invehicle_digital_twin_uri.clone()).await?;

    // Find all vehicle instances.
    let find_vehicle_response: FindResponse =
        find(client.clone(), sdv::vehicle::ID.to_string()).await?;
    if find_vehicle_response.values.is_empty() {
        return Err("Unable to find vehicle instances".to_string());
    }
    // For now, we will just use the first vehicle instance.
    let vehicle: sdv::vehicle::TYPE =
        serde_json::from_str(&find_vehicle_response.values[0]).unwrap();
    info!("The vehicle's instance id is: {}", vehicle.instance_id);

    // Get the cabin instance id.
    if vehicle.cabin.is_empty() {
        return Err("The vehicle does not have a cabin".to_string());
    }
    let cabin_instance_id = vehicle.cabin[0].instance_id.clone();
    info!("The cabin's instance id is: {:?}", cabin_instance_id);
    // Get the cabin instance.
    let get_cabin_response: GetResponse =
        get(client.clone(), cabin_instance_id.clone(), "".to_string()).await?;
    // Deserialize the cabin instance.
    let cabin: sdv::cabin::TYPE = serde_json::from_str(&get_cabin_response.value).unwrap();

    // Find the front left seat's instance id.
    let mut front_left_seat_instance_id_option: Option<String> = None;
    for seat_relationship in cabin.seat.iter() {
        if (seat_relationship.seat_row == 1)
            && (seat_relationship.seat_position == sdv::cabin::seat::SEAT_POSITION_TYPE::left)
        {
            info!("The front left seat's instance id is: {:?}", seat_relationship.instance_id);
            front_left_seat_instance_id_option = Some(seat_relationship.instance_id.clone());
        }
    }
    if front_left_seat_instance_id_option.is_none() {
        return Err("The front left seat was not found".to_string());
    }
    let front_left_seat_instance_id = front_left_seat_instance_id_option.unwrap();

    // Get the seat instance.
    let get_seat_response: GetResponse =
        get(client.clone(), front_left_seat_instance_id.clone(), "".to_string()).await?;
    // Deserialize the seat instance.
    let seat: sdv::seat::TYPE = serde_json::from_str(&get_seat_response.value).unwrap();

    // Get the seat massager instance id.
    if seat.seat_massager.is_empty() {
        return Err("The seat does not have a seat massage".to_string());
    }
    let seat_massager_instance_id = seat.seat_massager[0].instance_id.clone();
    info!("The seat massager's instance id is: {:?}", seat_massager_instance_id);
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
    // Let's make sure that we can fully serserialize the seat massager instance.  This in only a check and it is optional.
    let _seat_massager: sdv::premium_airbag_seat_massager::TYPE =
        serde_json::from_value(seat_massager_json.clone()).unwrap();

    // Randomly generate the airbag adjustment field values.
    let mut rng = StdRng::from_entropy();
    let airbag_identifier = rng.gen_range(1..=15);
    let inflation_level = rng.gen_range(1..=10);
    let inflation_duration_in_seconds = rng.gen_range(1..=5);
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
    let _perform_step_response: InvokeResponse = invoke(
        client.clone(),
        seat_massager_instance_id.clone(),
        sdv::airbag_seat_massager::perform_step::NAME.to_string(),
        request_payload_json.clone(),
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
