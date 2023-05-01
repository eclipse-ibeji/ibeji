// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

pub mod digitaltwin {
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize, Serialize, Debug, Clone)]
    pub struct Endpoint {
        pub protocol: String,
        pub operations: Vec<String>,
        pub uri: String,
        pub context: String,
    }
    
    #[derive(Deserialize, Serialize, Debug, Clone)]
    pub struct Entity {   
        pub name: String,
        pub id: String,
        pub description: String,
        pub endpoints: Vec<Endpoint>,
    }    

    #[derive(Deserialize, Serialize)]
    pub struct FindByIdRequestPayload {
        pub id: String,
    }

    #[derive(Deserialize, Serialize)]
    pub struct FindByIdResponsePayload {
        pub entity: Option<Entity>
    }

    #[derive(Deserialize, Serialize)]
    pub struct RegisterRequestPayload {
        pub entities: Vec<Entity>,
    }

    #[derive(Deserialize, Serialize)]
    pub struct RegisterResponsePayload {
    }

    #[derive(Deserialize, Serialize)]
    pub struct UnregisterRequestPayload {
    }

    #[derive(Deserialize, Serialize)]
    pub struct UnregisterResponsePayload {
    }
}
