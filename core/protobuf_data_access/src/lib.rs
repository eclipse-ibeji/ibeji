// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

pub mod invehicle_digital_twin {
    pub mod v1 {
        tonic::include_proto!("invehicle_digital_twin");
    }
}

pub mod extension {
    pub mod managed_subscribe {
        pub mod v1 {
            tonic::include_proto!("managed_subscribe");
        }
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

pub mod agemo {
    pub mod pubsub {
        pub mod v1 {
            tonic::include_proto!("pubsub");
        }
    }
    pub mod publisher {
        pub mod v1 {
            tonic::include_proto!("publisher");
        }
    }
}
