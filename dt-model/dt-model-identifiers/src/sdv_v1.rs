// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

// Note: In the future this code could be auto-generated from a DTDL spec.

pub mod vehicle {
    pub mod cabin {
        pub mod hvac {
            pub mod ambient_air_temperature {
                pub const ID: &str = "dtmi:sdv:Vehicle:Cabin:HVAC:AmbientAirTemperature;1";
            }
            pub mod is_air_conditioning_active {
                pub const ID: &str = "dtmi:sdv:Vehicle:Cabin:HVAC:IsAirConditioningActive;1";
            }
        }
        pub mod infotainment {
            pub mod hmi {
                pub mod show_notification {
                    pub const ID: &str =
                        "dtmi:sdv:Vehicle:Cabin:Infotainment:HMI:ShowNotification;1";
                }
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
