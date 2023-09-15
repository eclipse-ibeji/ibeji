// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::collections::HashMap;
use std::sync::Arc;

use core_protobuf_data_access::agemo::publisher::v1::publisher_callback_server::PublisherCallbackServer;
use core_protobuf_data_access::extensions::managed_subscribe::v1::managed_subscribe_server::ManagedSubscribeServer;
use parking_lot::RwLock;
use tonic::transport::{Server, server::Router};

use crate::extension_config;
use crate::managed_subscribe::managed_subscribe_ext::{self, SubscriptionStore, EntityMetadata, CallbackInfo};

fn intitialize_managed_subscribe_ext(address: &str) -> managed_subscribe_ext::ManagedSubscribeExt {
    let sub_store = Arc::new(RwLock::new(SubscriptionStore::new()));

    let entity_metadata = EntityMetadata {
        callback: CallbackInfo {
            uri: String::from("http://0.0.0.0:4010"),
            protocol: String::from("grpc"),
        },
        topics: HashMap::new(),
    };

    // Add entity to subscription store.
    {
        sub_store.write().add_entity("dtmi:sdv:HVAC:AmbientAirTemperature;1", entity_metadata);
    }

    // Create extensions.
    managed_subscribe_ext::ManagedSubscribeExt::new(
        &address,
        sub_store.clone(),
    )
}

pub fn service_extension_builder() -> Router {
    // Load the config.
    let settings = extension_config::load_settings();
    let invehicle_digital_twin_authority = settings.invehicle_digital_twin_authority;
    let invehicle_digital_twin_address = format!("http://{invehicle_digital_twin_authority}"); // Devskim: ignore DS137138


    let managed_subscribe_ext = intitialize_managed_subscribe_ext(&invehicle_digital_twin_address);

    Server::builder()
        .add_service(ManagedSubscribeServer::new(managed_subscribe_ext.clone()))
        .add_service(PublisherCallbackServer::new(managed_subscribe_ext))
}
