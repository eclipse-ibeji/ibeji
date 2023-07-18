// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

// Note: In the future this code should be generated from a DTDL spec.

pub mod airbag_seat_massager {
    pub mod massage_airbags {
        pub const ID: &str = "dtmi::sdv::airbag_seat_massager::massage_airbags";
        pub const NAME: &str = "MassageAirbags";
        pub const DESCRIPTION: &str = "The inflation level (0..100) for each massage airbag.";
        pub type TYPE = Vec<i32>;
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
    }
}

pub mod hvac {
    pub mod ambient_air_temperature {
        pub const ID: &str = "dtmi:sdv:HVAC:AmbientAirTemperature;1";
        pub const NAME: &str = "AmbientAirTemperature";
        pub const DESCRIPTION: &str = "The immediate surroundings air temperature (in Fahrenheit).";
        pub type TYPE = i32;
    }

    pub mod is_air_conditioning_active {
        pub const ID: &str = "dtmi:sdv:Vehicle:Cabin:HVAC:IsAirConditioningActive;1";
        pub const NAME: &str = "IsAirConditioningActive";
        pub const DESCRIPTION: &str = "Is air conditioning active?";
        pub type TYPE = bool;
    }
}

pub mod obd {
    pub mod hybrid_battery_remaining {
        pub const ID: &str = "dtmi:sdv::OBD:HybridBatteryRemaining;1";
        pub const NAME: &str = "HybridBatteryRemaining";
        pub const DESCRIPTION: &str = "The remaining hybrid battery life.";
        pub type TYPE = i32;
    }
}
