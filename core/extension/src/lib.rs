// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

#[cfg(feature = "managed_subscribe")]
/// Extension that communicates with a managed subscribe service to offer dynamically created
/// subscriptions on demand for Ibeji providers.
pub mod managed_subscribe {
    pub mod managed_subscribe_ext;
}