// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

pub mod invehicle_digital_twin {
    pub mod v1 {
        tonic::include_proto!("invehicle_digital_twin");
    }
}

pub mod module {
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

pub mod tutorial_grpc {
    pub mod v1 {
        tonic::include_proto!("digital_twin_provider_tutorial");
    }
}

pub mod async_rpc {
    pub mod v1 {
        pub mod common {
            tonic::include_proto!("async_rpc.v1.common");
        }
        pub mod respond {
            tonic::include_proto!("async_rpc.v1.respond");
        }
        pub mod request {
            tonic::include_proto!("async_rpc.v1.request");
        }
    }
}