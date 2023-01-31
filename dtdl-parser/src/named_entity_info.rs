// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use crate::entity_info::EntityInfo;

/// An abstract trait that represents named entites.
pub trait NamedEntityInfo: EntityInfo {
    /// Returns the name.
    fn name(&self) -> &Option<String>;
}
