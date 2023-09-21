// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::collections::HashMap;

use core_protobuf_data_access::extension::managed_subscribe::v1::Constraint;
use log::warn;

#[derive(Clone, Debug)]
pub struct CallbackInfo {
    pub uri: String,
    pub protocol: String,
}

#[derive(Clone, Debug)]
pub struct TopicInfo {
    pub uri: String,
    pub protocol: String,
    pub constraints: Vec<Constraint>,
}

#[derive(Clone, Debug)]
pub struct EntityMetadata {
    pub callback: CallbackInfo,
    pub topics: HashMap<String, TopicInfo>,
}

#[derive(Clone, Debug)]
pub struct ManagedSubscribeStore {
    topic_entity_map: HashMap<String, String>,
    entity_metadata_map: HashMap<String, EntityMetadata>,
}

impl Default for ManagedSubscribeStore {
    fn default() -> Self {
        Self::new()
    }
}

impl ManagedSubscribeStore {
    /// Creates a new instance of a subscription store.
    pub fn new() -> Self {
        ManagedSubscribeStore {
            topic_entity_map: HashMap::new(),
            entity_metadata_map: HashMap::new(),
        }
    }

    /// Adds an entity id with associated metadata to the store.
    ///
    /// # Arguments
    /// * `entity_id` - The entity to add.
    /// * `metadata` - The relevant metadata for the entity.
    pub fn add_entity(&mut self, entity_id: &str, metadata: EntityMetadata) {
        self.entity_metadata_map.insert(entity_id.to_string(), metadata);
    }

    /// Returns whether a specific entity is in the store.
    ///
    /// # Arguments
    /// * `entity_id` - The entity to find.
    pub fn contains_entity(&self, entity_id: &str) -> bool {
        self.entity_metadata_map.contains_key(entity_id)
    }

    /// Gets a specific entity's metadata from the store.
    ///
    /// # Arguments
    /// * `entity_id` - The entity to get information about.
    pub fn get_entity_metadata(&self, entity_id: &str) -> Option<&EntityMetadata> {
        self.entity_metadata_map.get(entity_id)
    }

    /// Gets the entity id associated with a specific topic.
    ///
    /// # Arguments
    /// * `topic` - The topic to get an entity id for.
    pub fn get_entity_id(&self, topic: &str) -> Option<&String> {
        self.topic_entity_map.get(topic)
    }

    /// Adds a topic to the store.
    ///
    /// # Arguments
    /// * `entity_id` - The entity id to associate with the topic.
    /// * `topic` - The topic to add.
    /// * `topic_info` - The associated topic info to add.
    pub fn add_topic(&mut self, entity_id: &str, topic: &str, topic_info: TopicInfo) {
        // Add map between topic and entity.
        self.topic_entity_map.insert(topic.to_string(), entity_id.to_string());

        // Add topic information to entity metadata.
        let metadata = self.entity_metadata_map.get_mut(entity_id).unwrap();
        metadata.topics.insert(topic.to_string(), topic_info);
    }

    /// Removes a topic from the store.
    ///
    /// # Arguments
    /// * `topic` - The topic to remove.
    pub fn remove_topic(&mut self, topic: &str) {
        // remove topic from topic and entity map.
        if let Some(entity_id) = self.topic_entity_map.remove(topic) {
            // remove topic and info from entity metadata map;
            if let Some(metadata) = self.entity_metadata_map.get_mut(&entity_id) {
                metadata.topics.remove(topic);
            } else {
                warn!("Unable to find an entry for {entity_id}");
            }
        } else {
            warn!("Unable to find an entry for {topic}.");
        }
    }
}
