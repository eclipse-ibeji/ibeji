// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

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
