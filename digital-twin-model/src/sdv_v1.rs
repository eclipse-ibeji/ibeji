// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

// Note: In the future this code should be generated from a DTDL spec.

#[allow(dead_code)]
pub mod camera {
    pub mod feed {
        pub const ID: &str = "dtmi:sdv:Camera:Feed;1";
        pub const NAME: &str = "Feed";
        pub const DESCRIPTION: &str = "The camera feed inside of the cabin.";
        pub struct Media {
            media_type: String,
            media_content: Vec<u8>,
        }
        pub type TYPE = Media;
    }
}

pub mod hmi {
    pub mod show_notification {
        pub const ID: &str = "dtmi:sdv:HMI:ShowNotification;1";
        pub const NAME: &str = "Show Notification";
        pub const DESCRIPTION: &str = "Show a notification on the HMI.";
        pub mod request {
            pub const ID: &str = "dtmi:sdv:HMI:ShowNotification::request;1";
            pub const NAME: &str = "Notification";
            pub const DESCRIPTION: &str = "The notification to show on the HMI.";
            pub type TYPE = String;
        }
        pub mod response {
            pub const ID: &str = "dtmi:sdv:HMI:ShowNotification::response;1";
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
    pub mod massage_airbags {
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

pub mod airbag_seat_massager {
    pub const ID: &str = "dtmi:sdv:airbag_seat_massager;1";
    pub const DESCRIPTION: &str = "Airbag Seat Massager Interface";
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