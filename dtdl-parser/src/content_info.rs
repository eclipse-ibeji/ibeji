// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use crate::named_entity_info::NamedEntityInfo;

/// An abstract trait that represents entites that have content.
pub trait ContentInfo: NamedEntityInfo {}
