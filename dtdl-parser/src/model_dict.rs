// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use crate::dtmi::Dtmi;
use crate::entity_info::EntityInfo;

use std::collections::HashMap;

pub type ModelDict = HashMap<Dtmi, EntityInfo>;
