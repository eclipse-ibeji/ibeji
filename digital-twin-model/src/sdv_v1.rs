// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

// This file contains the generated code for the Software Defined Vehicle (SDV) model.
// This code is manually generated today, but in the future it should be automatically generated from the DTDL.

/// The context value for all JSON-LD generated by the code in this file.
fn context() -> Vec<String> {
    vec!["dtmi:dtdl:context;3".to_string(), "dtmi:sdv:context;1".to_string()]
}

/// Constants and type definitions (including JSON serialization/deserialization) for each interface in the SDV model.
#[allow(dead_code)]
pub mod airbag_seat_massager {
    pub const ID: &str = "dtmi:sdv:airbag_seat_massager;1";
    pub const DESCRIPTION: &str = "Airbag Seat Massager Interface.";

    pub mod store_sequence {
        pub const ID: &str = "dtmi:sdv:airbag_seat_massager:store_sequence;1";
        pub const NAME: &str = "store_sequence";
        pub const DESCRIPTION: &str = "Store a massage sequence.";

        pub mod request {
            pub const ID: &str = "dtmi:sdv:airbag_seat_massager:store_sequence:request;1";
            pub const NAME: &str = "request";
            pub const DESCRIPTION: &str = "Request.";

            #[derive(derivative::Derivative)]
            #[derivative(Default)]
            #[derive(serde_derive::Serialize, serde_derive::Deserialize, Debug)]
            pub struct TYPE {
                #[serde(rename = "@context")]
                #[derivative(Default(value = "crate::sdv_v1::context()"))]
                pub context: Vec<String>,
                #[serde(rename = "@type")]
                #[derivative(Default(
                    value = "crate::sdv_v1::airbag_seat_massager::store_sequence::request::ID.to_string()"
                ))]
                pub model_id: String,
                pub sequence_name: String,
                pub sequence: crate::sdv_v1::airbag_seat_massager::massage_step::TYPE,
            }
        }

        pub mod response {
            pub const ID: &str = "dtmi:sdv:airbag_seat_massager:store_sequence:response;1";
            pub const NAME: &str = "response";
            pub const DESCRIPTION: &str = "Response.";

            #[derive(derivative::Derivative)]
            #[derivative(Default)]
            #[derive(serde_derive::Serialize, serde_derive::Deserialize, Debug)]
            pub struct TYPE {
                #[serde(rename = "@context")]
                #[derivative(Default(value = "crate::sdv_v1::context()"))]
                pub context: Vec<String>,
                #[serde(rename = "@type")]
                #[derivative(Default(
                    value = "crate::sdv_v1::airbag_seat_massager::store_sequence::response::ID.to_string()"
                ))]
                pub model_id: String,
                pub status: crate::sdv_v1::airbag_seat_massager::status::TYPE,
            }
        }
    }

    pub mod perform_step {
        pub const ID: &str = "dtmi:sdv:airbag_seat_massager:perform_step;1";
        pub const NAME: &str = "perform_step";
        pub const DESCRIPTION: &str = "Perform a step in the sequence.";

        pub mod request {
            pub const ID: &str = "dtmi:sdv:airbag_seat_massager:perform_step:request;1";
            pub const NAME: &str = "request";
            pub const DESCRIPTION: &str = "The request to perform a step in the sequence.";

            #[derive(derivative::Derivative)]
            #[derivative(Default)]
            #[derive(serde_derive::Serialize, serde_derive::Deserialize, Debug)]
            pub struct TYPE {
                #[serde(rename = "@context")]
                #[derivative(Default(value = "crate::sdv_v1::context()"))]
                pub context: Vec<String>,
                #[serde(rename = "@type")]
                #[derivative(Default(
                    value = "crate::sdv_v1::airbag_seat_massager::perform_step::request::ID.to_string()"
                ))]
                pub model_id: String,
                pub step: crate::sdv_v1::airbag_seat_massager::massage_step::TYPE,
            }
        }

        pub mod response {
            pub const ID: &str = "dtmi:sdv:airbag_seat_massager:perform_step:response;1";
            pub const NAME: &str = "response";
            pub const DESCRIPTION: &str = "The response to performing a step in the sequence.";

            #[derive(derivative::Derivative)]
            #[derivative(Default)]
            #[derive(serde_derive::Serialize, serde_derive::Deserialize, Debug)]
            pub struct TYPE {
                #[serde(rename = "@context")]
                #[derivative(Default(value = "crate::sdv_v1::context()"))]
                pub context: Vec<String>,
                #[serde(rename = "@type")]
                #[derivative(Default(
                    value = "crate::sdv_v1::airbag_seat_massager::perform_step::response::ID.to_string()"
                ))]
                pub model_id: String,
                pub status: crate::sdv_v1::airbag_seat_massager::status::TYPE,
            }
        }
    }

    pub mod airbag_adjustment {
        pub const ID: &str = "dtmi:sdv:airbag_seat_massager:airbag_adjustment;1";
        pub const NAME: &str = "airbag_adjustment";
        pub const DESCRIPTION: &str = "An airbag adjustment.";

        #[derive(serde_derive::Serialize, serde_derive::Deserialize, Debug)]
        pub struct TYPE {
            pub airbag_identifier: i32,
            pub inflation_level: i32,
            pub inflation_duration_in_seconds: i32,
        }
    }

    pub mod massage_step {
        pub const ID: &str = "dtmi:sdv:airbag_seat_massager:massage_step;1";
        pub const NAME: &str = "massage_step";
        pub const DESCRIPTION: &str = "The massage step.";

        pub type TYPE = Vec<crate::sdv_v1::airbag_seat_massager::airbag_adjustment::TYPE>;
    }

    pub mod status {
        pub const ID: &str = "dtmi:sdv:airbag_seat_massager:status;1";
        pub const NAME: &str = "status";
        pub const DESCRIPTION: &str = "The status.";

        #[derive(serde_derive::Serialize, serde_derive::Deserialize, Debug, Default)]
        pub struct TYPE {
            pub code: i32,
            pub message: String,
        }
    }
}

pub mod basic_airbag_seat_massager {
    pub const ID: &str = "dtmi:sdv:basic_airbag_seat_massager;1";
    pub const DESCRIPTION: &str = "Basic Airbag Seat Massager Interface.";
}

pub mod cabin {
    pub const ID: &str = "dtmi:sdv:cabin;1";
    pub const DESCRIPTION: &str = "Cabin Interface.";
}

#[allow(dead_code)]
pub mod camera {
    pub const ID: &str = "dtmi:sdv:camera;1";
    pub const DESCRIPTION: &str = "Camera Interface.";

    pub mod feed {
        pub const ID: &str = "dtmi:sdv:camera:feed;1";
        pub const NAME: &str = "feed";
        pub const DESCRIPTION: &str = "The camera feed.";

        #[derive(derivative::Derivative)]
        #[derivative(Default)]
        #[derive(serde_derive::Serialize, serde_derive::Deserialize, Debug)]
        pub struct TYPE {
            #[serde(rename = "@context")]
            #[derivative(Default(value = "crate::sdv_v1::context()"))]
            pub context: Vec<String>,
            #[serde(rename = "@type")]
            #[derivative(Default(value = "crate::sdv_v1::camera::feed::ID.to_string()"))]
            pub model_id: String,
            pub media_type: String,
            pub media_content: Vec<u8>,
        }
    }
}

pub mod hmi {
    pub const ID: &str = "dtmi:sdv:hmi;1";
    pub const DESCRIPTION: &str = "Human Machine Interface.";

    pub mod show_notification {
        pub const ID: &str = "dtmi:sdv:hmi:show_notification;1";
        pub const NAME: &str = "show_notification";
        pub const DESCRIPTION: &str = "Show a notification on the HMI.";

        pub mod request {
            pub const ID: &str = "dtmi:sdv:hmi:show_notification::request;1";
            pub const NAME: &str = "request";
            pub const DESCRIPTION: &str = "Request.";

            #[derive(derivative::Derivative)]
            #[derivative(Default)]
            #[derive(serde_derive::Serialize, serde_derive::Deserialize, Debug)]
            pub struct TYPE {
                #[serde(rename = "@context")]
                #[derivative(Default(value = "crate::sdv_v1::context()"))]
                pub context: Vec<String>,
                #[serde(rename = "@type")]
                #[derivative(Default(
                    value = "crate::sdv_v1::hmi::show_notification::request::ID.to_string()"
                ))]
                pub model_id: String,
                pub notification: String,
            }
        }

        pub mod response {
            pub const ID: &str = "dtmi:sdv:hmi:show_notification::response;1";
            pub const NAME: &str = "response";
            pub const DESCRIPTION: &str = "Response.";

            #[derive(derivative::Derivative)]
            #[derivative(Default)]
            #[derive(serde_derive::Serialize, serde_derive::Deserialize, Debug)]
            pub struct TYPE {
                #[serde(rename = "@context")]
                #[derivative(Default(value = "crate::sdv_v1::context()"))]
                pub context: Vec<String>,
                #[serde(rename = "@type")]
                #[derivative(Default(
                    value = "crate::sdv_v1::hmi::show_notification::response::ID.to_string()"
                ))]
                pub model_id: String,
                pub status: crate::sdv_v1::hmi::status::TYPE,
            }
        }
    }

    pub mod status {
        pub const ID: &str = "dtmi:sdv:hmi:status;1";
        pub const NAME: &str = "status";
        pub const DESCRIPTION: &str = "The status.";

        #[derive(serde_derive::Serialize, serde_derive::Deserialize, Debug, Default)]
        pub struct TYPE {
            pub code: i32,
            pub message: String,
        }
    }
}

pub mod hvac {
    pub const ID: &str = "dtmi:sdv:hvac;1";
    pub const DESCRIPTION: &str = "Heat, Ventilation and Air Conditioning (HVAC) Interface";

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
    pub const DESCRIPTION: &str = "On-board Diagnostics Interface.";

    pub mod hybrid_battery_remaining {
        pub const ID: &str = "dtmi:sdv:obd:hybrid_battery_remaining;1";
        pub const NAME: &str = "hybrid_battery_remaining";
        pub const DESCRIPTION: &str = "The remaining hybrid battery life.";

        pub type TYPE = i32;
    }
}

pub mod premium_airbag_seat_massager {
    pub const ID: &str = "dtmi:sdv:premium_airbag_seat_massager;1";
    pub const DESCRIPTION: &str = "Premium Airbag Seat Massager Interface.";
}

pub mod seat {
    pub const ID: &str = "dtmi:sdv:seat;1";
    pub const DESCRIPTION: &str = "Seat Interface.";
}

pub mod seat_massager {
    pub const ID: &str = "dtmi:sdv:seat_massager;1";
    pub const DESCRIPTION: &str = "Seat Massager Interface.";

    pub mod sequence_names {
        pub const ID: &str = "dtmi:sdv:seat_massager:sequence_names;1";
        pub const NAME: &str = "sequence_names";
        pub const DESCRIPTION: &str = "The name of each of the stored sequences.";

        #[derive(derivative::Derivative)]
        #[derivative(Default)]
        #[derive(serde_derive::Serialize, serde_derive::Deserialize, Debug)]
        pub struct TYPE {
            #[serde(rename = "@context")]
            #[derivative(Default(value = "crate::sdv_v1::context()"))]
            pub context: Vec<String>,
            #[serde(rename = "@type")]
            #[derivative(Default(
                value = "crate::sdv_v1::seat_massager::sequence_names::ID.to_string()"
            ))]
            pub model_id: String,
            pub sequence_names: Vec<String>,
        }
    }

    pub mod load_sequence {
        pub const ID: &str = "dtmi:sdv:seat_massager:load_sequence;1";
        pub const NAME: &str = "load_sequence";
        pub const DESCRIPTION: &str = "Load a sequence of massage steps.";

        pub mod request {
            pub const ID: &str = "dtmi:sdv:seat_massager:load_sequence:request;1";
            pub const NAME: &str = "request";
            pub const DESCRIPTION: &str = "Request.";

            #[derive(derivative::Derivative)]
            #[derivative(Default)]
            #[derive(serde_derive::Serialize, serde_derive::Deserialize, Debug)]
            pub struct TYPE {
                #[serde(rename = "@context")]
                #[derivative(Default(value = "crate::sdv_v1::context()"))]
                pub context: Vec<String>,
                #[serde(rename = "@type")]
                #[derivative(Default(
                    value = "crate::sdv_v1::seat_massager::load_sequence::request::ID.to_string()"
                ))]
                pub model_id: String,
                pub sequence_name: String,
            }
        }

        pub mod response {
            pub const ID: &str = "dtmi:sdv:seat_massager:load_sequence:response;1";
            pub const NAME: &str = "response";
            pub const DESCRIPTION: &str = "Response.";

            #[derive(derivative::Derivative)]
            #[derivative(Default)]
            #[derive(serde_derive::Serialize, serde_derive::Deserialize, Debug)]
            pub struct TYPE {
                #[serde(rename = "@context")]
                #[derivative(Default(value = "crate::sdv_v1::context()"))]
                pub context: Vec<String>,
                #[serde(rename = "@type")]
                #[derivative(Default(
                    value = "crate::sdv_v1::seat_massager::load_sequence::response::ID.to_string()"
                ))]
                pub model_id: String,
                pub status: crate::sdv_v1::seat_massager::status::TYPE,
            }
        }
    }

    pub mod pause {
        pub const ID: &str = "dtmi:sdv:seat_massager:pause;1";
        pub const NAME: &str = "pause";
        pub const DESCRIPTION: &str = "Pause whatever is currently playing.";

        pub mod request {
            pub const ID: &str = "dtmi:sdv:seat_massager:pause:request;1";
            pub const NAME: &str = "request";
            pub const DESCRIPTION: &str = "Request.";

            #[derive(derivative::Derivative)]
            #[derivative(Default)]
            #[derive(serde_derive::Serialize, serde_derive::Deserialize, Debug)]
            pub struct TYPE {
                #[serde(rename = "@context")]
                #[derivative(Default(value = "crate::sdv_v1::context()"))]
                pub context: Vec<String>,
                #[serde(rename = "@type")]
                #[derivative(Default(
                    value = "crate::sdv_v1::seat_massager::pause::request::ID.to_string()"
                ))]
                pub model_id: String,
            }
        }

        pub mod response {
            pub const ID: &str = "dtmi:sdv:seat_massager:pause:response;1";
            pub const NAME: &str = "response";
            pub const DESCRIPTION: &str = "Response.";

            #[derive(derivative::Derivative)]
            #[derivative(Default)]
            #[derive(serde_derive::Serialize, serde_derive::Deserialize, Debug)]
            pub struct TYPE {
                #[serde(rename = "@context")]
                #[derivative(Default(value = "crate::sdv_v1::context()"))]
                pub context: Vec<String>,
                #[serde(rename = "@type")]
                #[derivative(Default(
                    value = "crate::sdv_v1::seat_massager::pause::response::ID.to_string()"
                ))]
                pub model_id: String,
                pub status: crate::sdv_v1::seat_massager::status::TYPE,
            }
        }
    }

    pub mod play {
        pub const ID: &str = "dtmi:sdv:seat_massager:play;1";
        pub const NAME: &str = "play";
        pub const DESCRIPTION: &str = "Start/continue.";

        pub mod request {
            pub const ID: &str = "dtmi:sdv:seat_massager:play:request;1";
            pub const NAME: &str = "request";
            pub const DESCRIPTION: &str = "Request.";

            #[derive(derivative::Derivative)]
            #[derivative(Default)]
            #[derive(serde_derive::Serialize, serde_derive::Deserialize, Debug)]
            pub struct TYPE {
                #[serde(rename = "@context")]
                #[derivative(Default(value = "crate::sdv_v1::context()"))]
                pub context: Vec<String>,
                #[serde(rename = "@type")]
                #[derivative(Default(
                    value = "crate::sdv_v1::seat_massager::play::request::ID.to_string()"
                ))]
                pub model_id: String,
            }
        }

        pub mod response {
            pub const ID: &str = "dtmi:sdv:seat_massager:play:response;1";
            pub const NAME: &str = "response";
            pub const DESCRIPTION: &str = "Response.";

            #[derive(derivative::Derivative)]
            #[derivative(Default)]
            #[derive(serde_derive::Serialize, serde_derive::Deserialize, Debug)]
            pub struct TYPE {
                #[serde(rename = "@context")]
                #[derivative(Default(value = "crate::sdv_v1::context()"))]
                pub context: Vec<String>,
                #[serde(rename = "@type")]
                #[derivative(Default(
                    value = "crate::sdv_v1::seat_massager::play::response::ID.to_string()"
                ))]
                pub model_id: String,
                pub status: crate::sdv_v1::seat_massager::status::TYPE,
            }
        }
    }

    pub mod reset {
        pub const ID: &str = "dtmi:sdv:seat_massager:reset;1";
        pub const NAME: &str = "reset";
        pub const DESCRIPTION: &str = "Reset the seat.";

        pub mod request {
            pub const ID: &str = "dtmi:sdv:seat_massager:reset:request;1";
            pub const NAME: &str = "request";
            pub const DESCRIPTION: &str = "Request.";

            #[derive(derivative::Derivative)]
            #[derivative(Default)]
            #[derive(serde_derive::Serialize, serde_derive::Deserialize, Debug)]
            pub struct TYPE {
                #[serde(rename = "@context")]
                #[derivative(Default(value = "crate::sdv_v1::context()"))]
                pub context: Vec<String>,
                #[serde(rename = "@type")]
                #[derivative(Default(
                    value = "crate::sdv_v1::seat_massager::reset::request::ID.to_string()"
                ))]
                pub model_id: String,
            }
        }

        pub mod response {
            pub const ID: &str = "dtmi:sdv:seat_massager:reset:response;1";
            pub const NAME: &str = "response";
            pub const DESCRIPTION: &str = "Response.";

            #[derive(derivative::Derivative)]
            #[derivative(Default)]
            #[derive(serde_derive::Serialize, serde_derive::Deserialize, Debug)]
            pub struct TYPE {
                #[serde(rename = "@context")]
                #[derivative(Default(value = "crate::sdv_v1::context()"))]
                pub context: Vec<String>,
                #[serde(rename = "@type")]
                #[derivative(Default(
                    value = "crate::sdv_v1::seat_massager::reset::response::ID.to_string()"
                ))]
                pub model_id: String,
                pub status: crate::sdv_v1::seat_massager::status::TYPE,
            }
        }
    }

    pub mod status {
        pub const ID: &str = "dtmi:sdv:seat_massager:status;1";
        pub const NAME: &str = "status";
        pub const DESCRIPTION: &str = "The status.";

        #[derive(serde_derive::Serialize, serde_derive::Deserialize, Debug, Default)]
        pub struct TYPE {
            pub code: i32,
            pub message: String,
        }
    }
}

pub mod vehicle {
    pub const ID: &str = "dtmi:sdv:vehcile;1";
    pub const DESCRIPTION: &str = "Vehicle Interface.";

    pub mod vehicle_identification {
        pub const ID: &str = "dtmi:sdv:vehicle:vehicle_identification;1";
        pub const NAME: &str = "vehicle_identification.";
        pub const DESCRIPTION: &str = "Vehicle Identification";

        pub mod vin {
            pub const ID: &str = "dtmi:sdv:vehicle:vehicle_identification:vin;1";
            pub const NAME: &str = "vin";
            pub const DESCRIPTION: &str = "Vehicle Identification Number.";

            pub type TYPE = String;
        }

        #[derive(derivative::Derivative)]
        #[derivative(Default)]
        #[derive(serde_derive::Serialize, serde_derive::Deserialize, Debug)]
        pub struct TYPE {
            #[serde(rename = "@context")]
            #[derivative(Default(value = "crate::sdv_v1::context()"))]
            pub context: Vec<String>,
            #[serde(rename = "@type")]
            #[derivative(Default(
                value = "crate::sdv_v1::vehicle::vehicle_identification::ID.to_string()"
            ))]
            pub model_id: String,
            pub vin: crate::sdv_v1::vehicle::vehicle_identification::vin::TYPE,
        }
    }
}
