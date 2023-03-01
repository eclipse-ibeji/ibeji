// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use crate::entity_kind::EntityKind;

/// Does the entity kind represent a primitive schema?
///
/// # Arguments
/// * `entity_kind` - The entity kind.
pub fn is_primitive_schema_kind(entity_kind: EntityKind) -> bool {
    entity_kind == EntityKind::Boolean
        || entity_kind == EntityKind::Date
        || entity_kind == EntityKind::DateTime
        || entity_kind == EntityKind::Double
        || entity_kind == EntityKind::Duration
        || entity_kind == EntityKind::Float
        || entity_kind == EntityKind::Integer
        || entity_kind == EntityKind::Long
        || entity_kind == EntityKind::String
        || entity_kind == EntityKind::Time
}
