// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

pub mod digital_twin {
    pub mod v1 {
        tonic::include_proto!("digital_twin");
    }
}

pub mod chariott {
    pub mod common {
        pub mod v1 {
            tonic::include_proto!("chariott.common.v1");
        }
    }
    pub mod provider {
        pub mod v1 {
            tonic::include_proto!("chariott.provider.v1");
        }
    }
    pub mod runtime {
        pub mod v1 {
            tonic::include_proto!("chariott.runtime.v1");
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
