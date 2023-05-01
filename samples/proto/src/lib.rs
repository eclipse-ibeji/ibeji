// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

#![allow(clippy::derive_partial_eq_without_eq)]

pub mod sample_grpc {
    pub mod v1 {
        pub mod digital_twin_consumer {
            tonic::include_proto!("digital_twin_consumer");
        }

        pub mod digital_twin_provider {
            tonic::include_proto!("digital_twin_provider");
        }
    }
}
