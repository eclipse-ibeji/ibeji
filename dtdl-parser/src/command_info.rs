// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use crate::command_payload_info::CommandPayloadInfo;
use crate::content_info::ContentInfo;

/// A command is a method that can be invoked on a digital twin.
pub trait CommandInfo: ContentInfo {
    /// Returns the request.
    fn request(&self) -> &Option<Box<dyn CommandPayloadInfo>>;

    /// Returns the response.
    fn response(&self) -> &Option<Box<dyn CommandPayloadInfo>>;
}
