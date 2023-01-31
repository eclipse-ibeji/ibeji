// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use crate::content_info::ContentInfo;
use crate::schema_info::SchemaInfo;

/// A relationshp specifies an assoication with an interface.  It allows graphs to be built.
pub trait RelationshipInfo: ContentInfo {
    /// Returns the schema.
    fn schema(&self) -> &Option<Box<dyn SchemaInfo>>;

    /// Returns whether the property is writable.
    fn writable(&self) -> bool;
}
