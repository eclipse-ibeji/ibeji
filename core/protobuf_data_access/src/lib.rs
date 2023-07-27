// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

pub mod invehicle_digital_twin {
    pub mod v1 {
        tonic::include_proto!("invehicle_digital_twin");
    }
}

pub mod chariott {
    pub mod service_discovery {
        pub mod core {
            pub mod v1 {
                tonic::include_proto!("service_registry");
            }
        }
    }
}
