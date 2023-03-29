// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use dt_model_identifiers::sdv_v1 as sdv;
use dtdl_parser::dtmi::{create_dtmi, Dtmi};
use dtdl_parser::model_parser::ModelParser;
use env_logger::{Builder, Target};
use log::{debug, info, LevelFilter};
use proto::consumer::consumer_server::ConsumerServer;
use proto::digitaltwin::digital_twin_client::DigitalTwinClient;
use proto::digitaltwin::FindByIdRequest;
use proto::provider::provider_client::ProviderClient;
use proto::provider::SubscribeRequest;
use std::net::SocketAddr;
use tonic::transport::Server;

mod consumer_impl;

const IN_VEHICLE_DIGITAL_TWIN_SERVICE_URI: &str = "http://[::1]:50010"; // Devskim: ignore DS137138

const CONSUMER_ADDR: &str = "[::1]:60010"; // Devskim: ignore DS137138

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup logging.
    Builder::new().filter(None, LevelFilter::Info).target(Target::Stdout).init();

    info!("The Consumer has started.");

    // Setup the HTTP server.
    let addr: SocketAddr = CONSUMER_ADDR.parse()?;
    let consumer_impl = consumer_impl::ConsumerImpl::default();
    let server_future =
        Server::builder().add_service(ConsumerServer::new(consumer_impl)).serve(addr);

    // Obtain the DTDL for the ambient air temmpterature.
    info!("Sending a find_by_id request for entity id {} to the In-Vehicle Digital Twin Service URI {IN_VEHICLE_DIGITAL_TWIN_SERVICE_URI}", sdv::vehicle::cabin::hvac::ambient_air_temperature::ID);
    let mut client = DigitalTwinClient::connect(IN_VEHICLE_DIGITAL_TWIN_SERVICE_URI).await?;
    let request = tonic::Request::new(FindByIdRequest {
        entity_id: String::from(sdv::vehicle::cabin::hvac::ambient_air_temperature::ID),
    });
    let response = client.find_by_id(request).await?;
    let dtdl = response.into_inner().dtdl.clone();
    debug!("Received the response for the find_by_id request.");

    debug!("Parsing the DTDL.");
    let mut parser = ModelParser::new();
    let json_texts = vec![dtdl];
    let model_dict_result = parser.parse(&json_texts);
    if let Err(error) = model_dict_result {
        panic!("Failed to parse the DTDL: {error}");
    }
    let model_dict = model_dict_result.unwrap();
    debug!("The DTDL parser has successfully parsed the DTDL.");

    // Create the id (as a DTMI) for the ambient air temperature property.
    let ambient_air_temperature_property_id: Option<Dtmi> =
        create_dtmi(sdv::vehicle::cabin::hvac::ambient_air_temperature::ID);
    if ambient_air_temperature_property_id.is_none() {
        panic!("Unable to create the dtmi");
    }

    // Get the entity from the DTDL for the ambient air temperature property.
    let entity_result = model_dict.get(&ambient_air_temperature_property_id.unwrap());
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
        panic!("Unable to find the value for the URI for ambient air temperature's provider.");
    }
    let uri_property_value = uri_property_value_result.unwrap();
    let uri_str_option = uri_property_value.as_str();
    let uri = String::from(uri_str_option.unwrap());
    info!("The URI for the ambient air temperature's provider is {uri}");

    let consumer_uri = format!("http://{CONSUMER_ADDR}");

    // Subscribing to the ambient air temperature data feed.
    info!(
        "Sending a subscribe request for entity id {} to provider URI {uri}",
        sdv::vehicle::cabin::hvac::ambient_air_temperature::ID
    );
    let mut client = ProviderClient::connect(uri).await?;
    let request = tonic::Request::new(SubscribeRequest {
        entity_id: String::from(sdv::vehicle::cabin::hvac::ambient_air_temperature::ID),
        consumer_uri: String::from(consumer_uri),
    });
    let _response = client.subscribe(request).await?;

    server_future.await?;

    debug!("The Consumer has completed.");

    Ok(())
}
