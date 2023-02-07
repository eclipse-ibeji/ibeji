// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

mod consumer_impl;

use dt_model_identifiers::sdv;
use dtdl_parser::dtmi::{create_dtmi, Dtmi};
use dtdl_parser::model_parser::ModelParser;
use env_logger::{Builder, Target};
use log::{debug, info, LevelFilter};
use proto::consumer::consumer_server::ConsumerServer;
use proto::digitaltwin::digital_twin_client::DigitalTwinClient;
use proto::digitaltwin::FindByIdRequest;
use proto::provider::provider_client::ProviderClient;
use proto::provider::{InvokeRequest, SubscribeRequest};
use std::net::SocketAddr;
use tokio::time::{sleep, Duration};
use tonic::transport::Server;
use uuid::Uuid;

/// Start the send notification repeater.
///
/// # Arguments
/// `provider_uri` - The provider_uri.
/// `consumer_uri` - The consumer_uri.
fn start_send_notification_repeater(provider_uri: String, consumer_uri: String) {
    info!("Starting the Consumer's send notification repeater.");
    tokio::spawn(async move {
        loop {
            debug!("Invoking the send_notification command on endpoint {}", &provider_uri);

            let client_result = ProviderClient::connect(provider_uri.clone()).await;
            if client_result.is_err() {
                continue;
            }
            let mut client = client_result.unwrap();

            let response_id = Uuid::new_v4().to_string();

            let payload: String = String::from("The send_notification request.");

            let request = tonic::Request::new(InvokeRequest {
                entity_id: String::from(sdv::vehicle::cabin::hvac::send_notification::ID),
                consumer_uri: consumer_uri.clone(),
                response_id,
                payload,
            });

            let _response = client.invoke(request).await;

            sleep(Duration::from_secs(5)).await;
        }
    });
}

/// Start the set ui message repeater.
///
/// # Arguments
/// `provider_uri` - The provider_uri.
/// `consumer_uri` - The consumer_uri.
fn start_set_ui_message_repeater(provider_uri: String, consumer_uri: String) {
    info!("Starting the Consumer's send notification repeater.");
    tokio::spawn(async move {
        let array: [&str; 10] =
            ["one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten"];
        let mut index: usize = 0;
        loop {
            // info!("Invoking the set_ui_message command on endpoint {}", &provider_uri);

            let client_result = ProviderClient::connect(provider_uri.clone()).await;
            if client_result.is_err() {
                continue;
            }
            let mut client = client_result.unwrap();

            let response_id = Uuid::new_v4().to_string();

            let request = tonic::Request::new(InvokeRequest {
                entity_id: String::from(sdv::vehicle::cabin::hvac::set_ui_message::ID),
                consumer_uri: consumer_uri.clone(),
                response_id,
                payload: array[index].to_string(),
            });

            let _response = client.invoke(request).await;

            index = (index + 1) % array.len();

            sleep(Duration::from_secs(5)).await;
        }
    });
}

/// Start the set ui message repeater.
///
/// # Arguments
/// `provider_uri` - The provider_uri.
/// `consumer_uri` - The consumer_uri.
fn start_activate_air_conditioning_repeater(provider_uri: String, consumer_uri: String) {
    info!("Starting the Consumer's send notification repeater.");
    tokio::spawn(async move {
        let mut is_active = true;
        loop {
            // info!("Invoking the set_ui_message command on endpoint {}", &provider_uri);

            let client_result = ProviderClient::connect(provider_uri.clone()).await;
            if client_result.is_err() {
                continue;
            }
            let mut client = client_result.unwrap();

            let response_id = Uuid::new_v4().to_string();

            let payload: String = format!("{is_active}");

            let request = tonic::Request::new(InvokeRequest {
                entity_id: String::from(sdv::vehicle::cabin::hvac::activate_air_conditioning::ID),
                consumer_uri: consumer_uri.clone(),
                response_id,
                payload,
            });

            let _response = client.invoke(request).await;

            is_active = !is_active;

            sleep(Duration::from_secs(20)).await;
        }
    });
}

async fn get_provider_uri(entity_id: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Obtain the DTDL for the send_notification command.
    info!("Sending a find_by_id request to the Digital Twin Service for the DTDL for the send_notification command.");
    let mut client = DigitalTwinClient::connect("http://[::1]:50010").await?; // Devskim: ignore DS137138
    let request = tonic::Request::new(FindByIdRequest { entity_id: String::from(entity_id) });
    let response = client.find_by_id(request).await?;
    let dtdl = response.into_inner().dtdl;
    info!("Received the response for the find_by_id request. The DTDL is:\n{}", &dtdl);

    info!("Parsing the DTDL.");
    let mut parser = ModelParser::new();
    let json_texts = vec![dtdl];
    let model_dict_result = parser.parse(&json_texts);
    if let Err(error) = model_dict_result {
        panic!("Failed to parse the DTDL: {error}");
    }
    let model_dict = model_dict_result.unwrap();
    info!("The DTDL parser has successfully parsed the DTDL.");

    // Create the id (as a DTMI) for the send_notification command.
    let send_notification_command_id: Option<Dtmi> = create_dtmi(entity_id);
    if send_notification_command_id.is_none() {
        panic!("Unable to create the dtmi");
    }

    // Get the entity from the DTDL for the send notification command.
    let entity_result = model_dict.get(&send_notification_command_id.unwrap());
    if entity_result.is_none() {
        panic!("Unable to find the entity");
    }
    let entity = entity_result.unwrap();

    // Get the URI property from the entity.
    let uri_property_result = entity.undefined_properties().get(sdv::property::uri::ID);
    if uri_property_result.is_none() {
        panic!("Unable to find the URI property");
    }
    let uri_property = uri_property_result.unwrap();

    // Get the value for the URI property.
    let uri_property_value_result = uri_property.get("@value");
    if uri_property_value_result.is_none() {
        info!("Unable to find the value for the URI for the ambient air temperature's provider.");
    }
    let uri_property_value = uri_property_value_result.unwrap();
    let uri_str_option = uri_property_value.as_str();
    let provider_uri = String::from(uri_str_option.unwrap());
    info!("The URI for the send_notification command's provider is {}", &provider_uri);

    Ok(provider_uri)
}

async fn send_subscribe_request(
    provider_uri: &str,
    entity_id: &str,
    consumer_uri: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Sending a subscribe request for {}.", entity_id);
    let mut client = ProviderClient::connect(provider_uri.to_string()).await?;
    let request = tonic::Request::new(SubscribeRequest {
        entity_id: String::from(entity_id),
        consumer_uri: consumer_uri.to_string(), // Devskim: ignore DS137138
    });
    let _response = client.subscribe(request).await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup logging.
    Builder::new().filter(None, LevelFilter::Info).target(Target::Stdout).init();

    info!("The Consumer has started.");

    // Setup the HTTP server.
    let consumer_authority = String::from("[::1]:60010");
    let addr: SocketAddr = consumer_authority.parse()?;
    let consumer_impl = consumer_impl::ConsumerImpl::default();
    let server_future =
        Server::builder().add_service(ConsumerServer::new(consumer_impl)).serve(addr);

    let activate_air_conditioing_command_provider_uri =
        get_provider_uri(sdv::vehicle::cabin::hvac::activate_air_conditioning::ID).await?;
    let send_notification_command_provider_uri =
        get_provider_uri(sdv::vehicle::cabin::hvac::send_notification::ID).await?;
    let set_ui_message_command_provider_uri =
        get_provider_uri(sdv::vehicle::cabin::hvac::set_ui_message::ID).await?;

    let ambient_air_temperature_property_provider_uri =
        get_provider_uri(sdv::vehicle::cabin::hvac::ambient_air_temperature::ID).await?;
    let is_air_conditioning_active_property_uri =
        get_provider_uri(sdv::vehicle::cabin::hvac::is_air_conditioning_active::ID).await?;
    let hybrid_battery_remaining_property_uri =
        get_provider_uri(sdv::vehicle::obd::hybrid_battery_remaining::ID).await?;

    let consumer_uri = format!("http://{consumer_authority}"); // Devskim: ignore DS137138

    send_subscribe_request(
        &ambient_air_temperature_property_provider_uri,
        sdv::vehicle::cabin::hvac::ambient_air_temperature::ID,
        &consumer_uri,
    )
    .await?;

    send_subscribe_request(
        &is_air_conditioning_active_property_uri,
        sdv::vehicle::cabin::hvac::is_air_conditioning_active::ID,
        &consumer_uri,
    )
    .await?;

    send_subscribe_request(
        &hybrid_battery_remaining_property_uri,
        sdv::vehicle::obd::hybrid_battery_remaining::ID,
        &consumer_uri,
    )
    .await?;

    start_activate_air_conditioning_repeater(
        activate_air_conditioing_command_provider_uri,
        consumer_uri.clone(),
    );

    start_send_notification_repeater(
        send_notification_command_provider_uri.clone(),
        consumer_uri.clone(),
    );

    start_set_ui_message_repeater(set_ui_message_command_provider_uri.clone(), consumer_uri);

    server_future.await?;

    info!("The Consumer has conpleted.");

    Ok(())
}
