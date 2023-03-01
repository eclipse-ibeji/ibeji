// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

pub mod consumer {
    #![allow(clippy::derive_partial_eq_without_eq)]

    tonic::include_proto!("consumer");
}

pub mod digitaltwin {
    #![allow(clippy::derive_partial_eq_without_eq)]

    tonic::include_proto!("digitaltwin");
}

pub mod provider {
    #![allow(clippy::derive_partial_eq_without_eq)]

    tonic::include_proto!("provider");
}
