// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use crate::dtmi::Dtmi;
use crate::entity_info::EntityInfo;

use std::collections::HashMap;

pub type ModelDict = HashMap<Dtmi, Box<dyn EntityInfo>>;
