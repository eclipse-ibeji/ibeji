// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

// Note: In the future this code could be auto-generated from a DTDL spec.

pub mod vehicle {
    pub mod cabin {
        pub mod hvac {
            pub mod activate_air_conditioning {
                pub const ID: &str = "dtmi:sdv:Vehicle:Cabin:HVAC:ActivateAirConditioning;1";
            }
            pub mod send_notification {
                pub const ID: &str = "dtmi:sdv:Vehicle:Cabin:HVAC:SendNotification;1";
            }
            pub mod set_ui_message {
                pub const ID: &str = "dtmi:sdv:Vehicle:Cabin:HVAC:SetUiMessage;1";
            }
            pub mod ambient_air_temperature {
                pub const ID: &str = "dtmi:sdv:Vehicle:Cabin:HVAC:AmbientAirTemperature;1";
            }
            pub mod is_air_conditioning_active {
                pub const ID: &str = "dtmi:sdv:Vehicle:Cabin:HVAC:IsAirConditioningActive;1";
            }
        }
    }
    pub mod obd {
        pub mod hybrid_battery_remaining {
            pub const ID: &str = "dtmi:sdv:Vehicle:OBD:HybridBatteryRemaining;1";
        }
    }
}

pub mod property {
    pub mod uri {
        pub const ID: &str = "dtmi:sdv:property:uri;1";
    }
}
