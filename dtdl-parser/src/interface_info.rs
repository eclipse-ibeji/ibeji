// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use crate::entity_info::EntityInfo;

pub trait InterfaceInfo : EntityInfo {
    fn as_entity_info(&self) -> &dyn EntityInfo;  
}