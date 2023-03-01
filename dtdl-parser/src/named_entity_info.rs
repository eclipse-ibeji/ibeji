// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use crate::entity_info::EntityInfo;

/// An abstract trait that represents named entites.
pub trait NamedEntityInfo: EntityInfo {
    /// Returns the name.
    fn name(&self) -> &Option<String>;
}
