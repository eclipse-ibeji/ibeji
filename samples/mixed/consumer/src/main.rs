// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

mod consumer_impl;

use dt_model_identifiers::sdv_v1 as sdv;
use dtdl_parser::dtmi::{create_dtmi, Dtmi};
use dtdl_parser::model_parser::ModelParser;
use env_logger::{Builder, Target};
use log::{debug, info, warn, LevelFilter};
use proto::consumer::consumer_server::ConsumerServer;
use proto::digitaltwin::digital_twin_client::DigitalTwinClient;
use proto::digitaltwin::FindByIdRequest;
use proto::provider::provider_client::ProviderClient;
use proto::provider::{InvokeRequest, SetRequest, SubscribeRequest};
use std::net::SocketAddr;
use tokio::time::{sleep, Duration};
use tonic::transport::Server;
use uuid::Uuid;

/// Start the show-notification repeater.
///
/// # Arguments
/// `provider_uri` - The provider_uri.
/// `consumer_uri` - The consumer_uri.
fn start_show_notification_repeater(provider_uri: String, consumer_uri: String) {
    debug!("Starting the Consumer's show-notification repeater.");
    tokio::spawn(async move {
        loop {
            let client_result = ProviderClient::connect(provider_uri.clone()).await;
            if client_result.is_err() {
                warn!("Unable to connect. We will retry in a moment.");
                sleep(Duration::from_secs(1)).await;
                continue;
            }
            let mut client = client_result.unwrap();

            let response_id = Uuid::new_v4().to_string();

            let payload: String = String::from("The show-notification request.");

            let request = tonic::Request::new(InvokeRequest {
                entity_id: String::from(
                    sdv::vehicle::cabin::infotainment::hmi::show_notification::ID,
                ),
                consumer_uri: consumer_uri.clone(),
                response_id,
                payload,
            });

            let response = client.invoke(request).await;
            match response {
                Ok(_) => (),
                Err(status) => warn!("{status:?}"),
            }

            info!("Invoked the show-notification command on endpoint {provider_uri}");

            sleep(Duration::from_secs(5)).await;
        }
    });
}

/// Start the activate-air-conditioing repeater.
///
/// # Arguments
/// `provider_uri` - The provider_uri..
fn start_activate_air_conditioning_repeater(provider_uri: String) {
    debug!("Starting the Consumer's activate-air-conditioning repeater.");
    tokio::spawn(async move {
        let mut is_active = true;
        loop {
            let client_result = ProviderClient::connect(provider_uri.clone()).await;
            if client_result.is_err() {
                warn!("Unable to connect. We will retry in a moment.");
                sleep(Duration::from_secs(1)).await;
                continue;
            }
            let mut client = client_result.unwrap();

            let value: String = format!("{is_active}");

            let request = tonic::Request::new(SetRequest {
                entity_id: String::from(sdv::vehicle::cabin::hvac::is_air_conditioning_active::ID),
                value,
            });

            let response = client.set(request).await;
            match response {
                Ok(_) => is_active = !is_active,
                Err(status) => warn!("{status:?}"),
            }

            info!("Set the is_activate_air_conditioning property on endpoint {provider_uri}");

            sleep(Duration::from_secs(20)).await;
        }
    });
}

async fn get_provider_uri(entity_id: &str) -> Result<String, String> {
    // Obtain the DTDL for the send_notification command.
    debug!("Sending a find_by_id request to the Digital Twin Service for the DTDL for the send_notification command.");
    let mut client = DigitalTwinClient::connect("http://[::1]:50010") // Devskim: ignore DS137138
        .await
        .map_err(|error| format!("{error}"))?;
    let request = tonic::Request::new(FindByIdRequest { entity_id: String::from(entity_id) });
    let response = client.find_by_id(request).await.map_err(|error| format!("{error}"))?;
    let dtdl = response.into_inner().dtdl;
    debug!("Received the response for the find_by_id request. The DTDL is:\n{dtdl}");

    debug!("Parsing the DTDL.");
    let mut parser = ModelParser::new();
    let json_texts = vec![dtdl];
    let model_dict_result = parser.parse(&json_texts);
    if let Err(error) = model_dict_result {
        return Err(format!("Failed to parse the DTDL: {error}"));
    }
    let model_dict = model_dict_result.unwrap();
    debug!("The DTDL parser has successfully parsed the DTDL.");

    // Create the id (as a DTMI) for the send_notification command.
    let dtmi_id: Option<Dtmi> = create_dtmi(entity_id);
    if dtmi_id.is_none() {
        return Err(String::from("Unable to create the dtmi"));
    }

    // Get the entity from the DTDL for the dtmi id.
    let entity_result = model_dict.get(&dtmi_id.unwrap());
    if entity_result.is_none() {
        return Err(String::from("Unable to find the entity"));
    }
    let entity = entity_result.unwrap();

    // Get the URI property from the entity.
    let uri_property_result = entity.undefined_properties().get(sdv::property::uri::ID);
    if uri_property_result.is_none() {
        return Err(String::from("Unable to find the URI property"));
    }
    let uri_property = uri_property_result.unwrap();

    // Get the value for the URI property.
    let uri_property_value_result = uri_property.get("@value");
    if uri_property_value_result.is_none() {
        return Err(String::from("Unable to find the value for the URI."));
    }
    let uri_property_value = uri_property_value_result.unwrap();
    let uri_str_option = uri_property_value.as_str();
    let provider_uri = String::from(uri_str_option.unwrap());
    info!("The provider URI for entity {entity_id} is {provider_uri}");

    Ok(provider_uri)
}

async fn send_subscribe_request(
    provider_uri: &str,
    entity_id: &str,
    consumer_uri: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    debug!("Sending a subscribe request for {entity_id}.");
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

    let show_notification_command_provider_uri =
        get_provider_uri(sdv::vehicle::cabin::infotainment::hmi::show_notification::ID)
            .await
            .unwrap();

    let ambient_air_temperature_property_provider_uri =
        get_provider_uri(sdv::vehicle::cabin::hvac::ambient_air_temperature::ID).await.unwrap();

    let is_air_conditioning_active_property_uri =
        get_provider_uri(sdv::vehicle::cabin::hvac::is_air_conditioning_active::ID).await.unwrap();

    let hybrid_battery_remaining_property_uri =
        get_provider_uri(sdv::vehicle::obd::hybrid_battery_remaining::ID).await.unwrap();

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

    start_activate_air_conditioning_repeater(is_air_conditioning_active_property_uri);

    start_show_notification_repeater(
        show_notification_command_provider_uri.clone(),
        consumer_uri.clone(),
    );

    server_future.await?;

    debug!("The Consumer has conpleted.");

    Ok(())
}
