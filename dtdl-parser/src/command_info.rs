// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use crate::content_info::ContentInfo;
use crate::command_payload_info::CommandPayloadInfo;

pub trait CommandInfo  : ContentInfo {
    /// Returns the request.
    fn request(&self) -> &Option<Box<dyn CommandPayloadInfo>>;

    /// Returns the response.
    fn response(&self) -> &Option<Box<dyn CommandPayloadInfo>>;
}
