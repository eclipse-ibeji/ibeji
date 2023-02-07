// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

// Note: In the future this code could be auto-generated from a DTDL spec.

pub mod vehicle {
    pub mod cabin {
        pub mod hvac {
            pub mod activate_air_conditioning {
                pub const V1_ID: &str = "dtmi:sdv:Vehicle:Cabin:HVAC:ActivateAirConditioning;1";
                pub const ID: &str = V1_ID;
            }
            pub mod send_notification {
                pub const V1_ID: &str = "dtmi:sdv:Vehicle:Cabin:HVAC:SendNotification;1";
                pub const ID: &str = V1_ID;
            }
            pub mod set_ui_message {
                pub const V1_ID: &str = "dtmi:sdv:Vehicle:Cabin:HVAC:SetUiMessage;1";
                pub const ID: &str = V1_ID;
            }
            pub mod ambient_air_temperature {
                pub const V1_ID: &str = "dtmi:sdv:Vehicle:Cabin:HVAC:AmbientAirTemperature;1";
                pub const ID: &str = V1_ID;
            }
            pub mod is_air_conditioning_active {
                pub const V1_ID: &str = "dtmi:sdv:Vehicle:Cabin:HVAC:IsAirConditioningActive;1";
                pub const ID: &str = V1_ID;
            }
        }
    }
    pub mod obd {
        pub mod hybrid_battery_remaining {
            pub const V1_ID: &str = "dtmi:sdv:Vehicle:OBD:HybridBatteryRemaining;1";
            pub const ID: &str = V1_ID;
        }
    }
}

pub mod property {
    pub mod uri {
        pub const V1_ID: &str = "dtmi:sdv:property:uri;1";
        pub const ID: &str = V1_ID;
    }
}
