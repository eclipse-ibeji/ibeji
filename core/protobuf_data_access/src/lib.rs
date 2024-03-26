// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

pub mod invehicle_digital_twin {
    pub mod v1 {
        tonic::include_proto!("invehicle_digital_twin");
    }
}

pub mod async_rpc {
    pub mod v1 {
        pub mod respond {
            tonic::include_proto!("async_rpc.v1.respond");
        }
        pub mod request {
            tonic::include_proto!("async_rpc.v1.request");
        }
    }
}

pub mod module {
    pub mod managed_subscribe {
        pub mod v1 {
            tonic::include_proto!("managed_subscribe");
        }
    }
    pub mod digital_twin_graph {
        pub mod v1 {
            tonic::include_proto!("digital_twin_graph.v1.digital_twin_graph");
        }
    }
    pub mod digital_twin_registry {
        pub mod v1 {
            tonic::include_proto!("digital_twin_registry.v1.digital_twin_registry");
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
