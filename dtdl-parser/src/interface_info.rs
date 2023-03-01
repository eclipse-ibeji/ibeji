// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use crate::entity_info::EntityInfo;

/// An interface specifies a collection of Commands, Components, Properties, Relationships and Telemetries.
pub trait InterfaceInfo: EntityInfo {}
