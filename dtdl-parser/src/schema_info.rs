// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use crate::entity_info::EntityInfo;

/// A schema is the base trait for all primitive and complex schemas.
pub trait SchemaInfo: EntityInfo {}
