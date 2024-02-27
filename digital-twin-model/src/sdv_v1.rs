// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

// Note: In the future this code should be generated from a DTDL spec.

//use serde_derive::{Deserialize, Serialize};

#[allow(dead_code)]
pub mod camera {
    pub mod feed {
        pub const ID: &str = "dtmi:sdv:camera:feed;1";
        pub const NAME: &str = "feed";
        pub const DESCRIPTION: &str = "The camera feed inside of the cabin.";
        pub struct TYPE {
            pub media_type: String,
            pub media_content: Vec<u8>,
        }
    }
}

pub mod hmi {
    pub mod show_notification {
        pub const ID: &str = "dtmi:sdv:hmi:show_notification;1";
        pub const NAME: &str = "show_notification";
        pub const DESCRIPTION: &str = "Show a notification on the HMI.";
        pub mod request {
            pub const ID: &str = "dtmi:sdv:hmi:show_notification::request;1";
            pub const NAME: &str = "request";
            pub const DESCRIPTION: &str = "The notification to show on the HMI.";
            pub type TYPE = String;
        }
        pub mod response {
            pub const ID: &str = "dtmi:sdv:hmi:show_notification::response;1";
        }
    }
}

pub mod hvac {
    pub mod ambient_air_temperature {
        pub const ID: &str = "dtmi:sdv:hvac:ambient_air_temperature;1";
        pub const NAME: &str = "ambient_air_temperature";
        pub const DESCRIPTION: &str = "The immediate surroundings air temperature (in Fahrenheit).";
        pub type TYPE = i32;
    }

    pub mod is_air_conditioning_active {
        pub const ID: &str = "dtmi:sdv:HVAC:is_air_conditioning_active;1";
        pub const NAME: &str = "is_air_conditioning_active";
        pub const DESCRIPTION: &str = "Is air conditioning active?";
        pub type TYPE = bool;
    }
}

pub mod obd {
    pub const ID: &str = "dtmi:sdv:obd;1";
    pub const DESCRIPTION: &str = "On-board Diagnostics Interface";  
    pub mod hybrid_battery_remaining {
        pub const ID: &str = "dtmi:sdv:obd:hybrid_battery_remaining;1";
        pub const NAME: &str = "hybrid_battery_remaining";
        pub const DESCRIPTION: &str = "The remaining hybrid battery life.";
        pub type TYPE = i32;
    }
}

pub mod seat_massager {
    pub const ID: &str = "dtmi:sdv:seatmassager;1";
    pub const DESCRIPTION: &str = "Seat Massager Interface";      
    pub mod sequence_names {
        pub const ID: &str = "dtmi:sdv:massage_seat:sequence_names;1";
        pub const NAME: &str = "sequence_names";
        pub const DESCRIPTION: &str = "The name of each of the stored sequences.";
        pub type TYPE = Vec<String>;
    }
}

pub mod basic_airbag_seat_massager {
    pub const ID: &str = "dtmi:sdv:basic_airbag_seat_massager;1";
    pub const DESCRIPTION: &str = "Basic Airbag Seat Massager Interface";      
}

pub mod premium_airbag_seat_massager {
    pub const ID: &str = "dtmi:sdv:premium_airbag_seat_massager;1";
    pub const DESCRIPTION: &str = "Premium Airbag Seat Massager Interface";    
}

#[allow(dead_code)]
pub mod airbag_seat_massager {
    pub const ID: &str = "dtmi:sdv:airbag_seat_massager;1";
    pub const DESCRIPTION: &str = "Airbag Seat Massager Interface";  
    pub mod perform_step {
        pub const ID: &str = "dtmi:sdv:airbag_seat_massager:perform_step;1";
        pub const NAME: &str = "perform_step";
        pub const DESCRIPTION: &str = "Perform a step in the sequence.";
        pub mod request {
            pub const ID: &str = "dtmi:sdv:airbag_seat_massager:perform_step:request;1";
            pub const NAME: &str = "request";
            pub const DESCRIPTION: &str = "The request to perform a step in the sequence.";
            #[derive(serde_derive::Serialize, serde_derive::Deserialize)]
            pub struct TYPE {
                pub step: crate::sdv_v1::airbag_seat_massager::massage_step::TYPE
            }
        }
        pub mod response {
            pub const ID: &str = "dtmi:sdv:airbag_seat_massager:perform_step:response;1";
            pub const NAME: &str = "response";
            pub const DESCRIPTION: &str = "The response to performing a step in the sequence.";
            #[derive(serde_derive::Serialize, serde_derive::Deserialize)]
            pub struct TYPE {
                pub status: crate::sdv_v1::airbag_seat_massager::status::TYPE
            }
        }
    }
    pub mod airbag_adjustment {
        pub const ID: &str = "dtmi:sdv:airbag_seat_massager:airbag_adjustment;1";
        pub const NAME: &str = "airbag_adjustment";
        pub const DESCRIPTION: &str = "The airbag adjustments.";
        #[derive(serde_derive::Serialize, serde_derive::Deserialize)]
        pub struct TYPE {
            pub airbag_identifier: i32,
            pub inflation_level: i32
        }
    }    
    pub mod airbag_adjustments {
        pub const ID: &str = "dtmi:sdv:airbag_seat_massager:airbag_adjustments;1";
        pub const NAME: &str = "airbag_adjustments";
        pub const DESCRIPTION: &str = "The airbag adjustments.";
        pub type TYPE = Vec<crate::sdv_v1::airbag_seat_massager::airbag_adjustment::TYPE>;
    }
    pub mod massage_step {
        pub const ID: &str = "dtmi:sdv:airbag_seat_massager:massage_step;1";
        pub const NAME: &str = "massage_step";
        pub const DESCRIPTION: &str = "The massage step.";
        pub type TYPE = Vec<crate::sdv_v1::airbag_seat_massager::airbag_adjustments::TYPE>;
    }
    pub mod status {
        pub const ID: &str = "dtmi:sdv:airbag_seat_massager:status;1";
        pub const NAME: &str = "status";
        pub const DESCRIPTION: &str = "The status.";
        #[derive(serde_derive::Serialize, serde_derive::Deserialize)]
        pub struct TYPE {
            pub code: i32,
            pub message: String
        }
    }      
}

pub mod seat {
    pub const ID: &str = "dtmi:sdv:seat;1";
    pub const DESCRIPTION: &str = "Seat Interface";  
}

pub mod cabin {
    pub const ID: &str = "dtmi:sdv:cabin;1";
    pub const DESCRIPTION: &str = "Cabin Interface";
}

pub mod vehicle {
    pub const ID: &str = "dtmi:sdv:vehcile;1";
    pub const DESCRIPTION: &str = "Vehicle Interface";
    pub mod vehicle_identification {
        pub const ID: &str = "dtmi:sdv:vehicle:vehicle_identification;1";
        pub const NAME: &str = "vehicle_identification";
        pub const DESCRIPTION: &str = "Vehicle Identification";
        pub mod vin {
            pub const ID: &str = "dtmi:sdv:vehicle:vehicle_identification:vin;1";
            pub const NAME: &str = "vin";
            pub const DESCRIPTION: &str = "Vehicle Identification Number";
            pub type TYPE = String;
        }
    }
}