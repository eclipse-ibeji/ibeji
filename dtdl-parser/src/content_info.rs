// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use crate::named_entity_info::NamedEntityInfo;

/// An abstract trait that represents entites that have content.
pub trait ContentInfo: NamedEntityInfo {}
