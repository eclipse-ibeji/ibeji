// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use iref::Iri;
use log::warn;
use regex::Regex;
use std::fmt;

lazy_static! {
    pub static ref DTMI_REGEX: Regex =
        Regex::new(r"^dtmi:[^;]+(;[1-9][0-9]*(\.[0-9][1-9]*)?)?(#[^ ]*)?$").unwrap();
}

/// Digital Twin Model Identifier (DTMI).
#[derive(Debug, Default, Clone, Hash, Eq, PartialEq)]
pub struct Dtmi {
    value: String,
    major_version: Option<u32>,
    minor_version: Option<u32>,
    versionless: String,
    labels: Vec<String>,
    absolute_path: String,
    fragment: String,
}

impl Dtmi {
    /// Returns a new DTMI instance.
    ///
    /// # Arguments
    /// * `value` - The string representation of the DTMI.
    pub fn new(value: &str) -> Result<Self, String> {
        let new_iri_result = Iri::new(value);
        if new_iri_result.is_err() {
            return Err(format!("The value '{value}' does not represent a valid IRI"));
        }
        let iri = new_iri_result.unwrap();

        let mut major_version: Option<u32> = None;
        let mut minor_version: Option<u32> = None;
        let absolute_path: String;

        let iri_path_parts: Vec<&str> = iri.path().into_str().split(';').collect();
        if iri_path_parts.len() == 1 {
            // no version
            absolute_path = String::from(iri_path_parts[0]);
        } else if iri_path_parts.len() == 2 {
            absolute_path = String::from(iri_path_parts[0]);
            let version_parts: Vec<&str> = iri_path_parts[1].split('.').collect();
            if version_parts.len() == 1 {
                // no minor version number
                if let Ok(value) = version_parts[0].parse::<u32>() {
                    major_version = Some(value)
                }
            } else if version_parts.len() == 2 {
                if let Ok(value) = version_parts[0].parse::<u32>() {
                    major_version = Some(value)
                }
                if let Ok(value) = version_parts[1].parse::<u32>() {
                    minor_version = Some(value)
                }
            } else {
                return Err(format!("The value '{value}' has an invalid version"));
            }
        } else {
            return Err(format!("The value '{value}' represents an invalid DTMI"));
        }

        let versionless: String = format!("dtmi:{absolute_path}");

        let labels: Vec<String> = absolute_path.split(':').map(Into::into).collect();

        let fragment = match iri.fragment() {
            Some(fragment) => String::from(fragment.as_str()),
            None => String::new(),
        };

        Ok(Self {
            value: String::from(value),
            major_version,
            minor_version,
            versionless,
            labels,
            absolute_path,
            fragment,
        })
    }

    /// Gets the string representation of the DTMI.
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Gets the major version of the DTMI.
    pub fn major_version(&self) -> &Option<u32> {
        &self.major_version
    }

    /// Gets the minor version of the DTMI.
    pub fn minor_version(&self) -> &Option<u32> {
        &self.minor_version
    }

    /// Gets the major and minor version of the DTMI.
    pub fn complete_version(&self) -> f64 {
        let major_version: f64 = match self.major_version {
            Some(value) => value.into(),
            None => 0.0,
        };

        let minor_version: f64 = match self.minor_version {
            Some(value) => value.into(),
            None => 0.0,
        };

        major_version + minor_version * 0.000001
    }

    /// Gets the portion of the DTMI that preceeds the version number.
    pub fn versionless(&self) -> &str {
        &self.versionless
    }

    /// Gets the sequence of labels in the path portion of the DTMI.
    pub fn labels(&self) -> &Vec<String> {
        &self.labels
    }

    /// Gets the absolute path of the DTMI.
    pub fn absolute_path(&self) -> &str {
        &self.absolute_path
    }

    /// Gets the name of the DTMI's fragment, which is the empty string if there is no fragment.
    pub fn fragment(&self) -> &str {
        &self.fragment
    }
}

impl fmt::Display for Dtmi {
    /// Format support for DTMI.
    ///
    /// # Arguments
    /// * `f` - The associated formatter.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value())
    }
}

/// Create a new DTMI instance.
///
/// # Arguments
/// * `value` - The IRI to copy from.
pub fn create_dtmi(value: &str) -> Option<Dtmi> {
    if !DTMI_REGEX.is_match(value) {
        warn!("The value '{}' does not represent a valid DTMI", value);
        return None;
    }

    let new_dtmi_result = Dtmi::new(value);
    if let Err(error) = new_dtmi_result {
        warn!("{}", error);
        return None;
    }

    Some(new_dtmi_result.unwrap())
}

#[cfg(test)]
mod dmti_tests {
    use super::*;

    #[test]
    fn new_dtmi_test() {
        let mut new_dtmi_result = Dtmi::new("dtmi:com:example:Thermostat;1.234#some-fragment");
        assert!(new_dtmi_result.is_ok());
        let mut dtmi: Dtmi = new_dtmi_result.unwrap();
        assert!(dtmi.major_version().is_some());
        assert!(dtmi.major_version().unwrap() == 1);
        assert!(dtmi.minor_version().is_some());
        assert!(dtmi.minor_version().unwrap() == 234);
        assert!(dtmi.complete_version() == 1.000234);
        assert!(dtmi.versionless() == "dtmi:com:example:Thermostat");
        assert!(dtmi.labels().len() == 3);
        assert!(dtmi.labels()[0] == "com");
        assert!(dtmi.labels()[1] == "example");
        assert!(dtmi.labels()[2] == "Thermostat");
        assert!(dtmi.absolute_path == "com:example:Thermostat");
        assert!(dtmi.fragment() == "some-fragment");
        assert!(format!("{dtmi}") == "dtmi:com:example:Thermostat;1.234#some-fragment");

        new_dtmi_result = Dtmi::new("dtmi:com:example:Thermostat;1.234#");
        assert!(new_dtmi_result.is_ok());
        dtmi = new_dtmi_result.unwrap();
        assert!(dtmi.fragment() == "");
        assert!(format!("{dtmi}") == "dtmi:com:example:Thermostat;1.234#");
    }

    #[test]
    fn create_dtmi_test() {
        let create_dtmi_result: Option<Dtmi> = create_dtmi("dtmi:com:example:Thermostat;1.234567");
        assert!(create_dtmi_result.is_some());
        let dtmi = create_dtmi_result.unwrap();
        assert!(dtmi.major_version().is_some());
        assert!(dtmi.major_version().unwrap() == 1);
        assert!(dtmi.minor_version().is_some());
        assert!(dtmi.minor_version().unwrap() == 234567);
        assert!(dtmi.complete_version() == 1.234567);
        assert!(dtmi.versionless() == "dtmi:com:example:Thermostat");
        assert!(dtmi.labels().len() == 3);
        assert!(dtmi.labels()[0] == "com");
        assert!(dtmi.labels()[1] == "example");
        assert!(dtmi.labels()[2] == "Thermostat");
        assert!(dtmi.absolute_path == "com:example:Thermostat");
    }

    #[test]
    fn bad_create_dtmi_test() {
        let mut create_dtmi_result: Option<Dtmi> =
            create_dtmi("whatever:com:example:Thermostat;1.234567");
        assert!(create_dtmi_result.is_none());

        create_dtmi_result = create_dtmi("dtmi:com:example:Thermostat;1.2.3");
        assert!(create_dtmi_result.is_none());

        create_dtmi_result = create_dtmi("dtmi:;1.2");
        assert!(create_dtmi_result.is_none());
    }

    #[test]
    fn eq_dtmi_test() {
        let first_create_dtmi_result: Option<Dtmi> =
            create_dtmi("dtmi:com:example:Thermostat;1.234567");
        assert!(first_create_dtmi_result.is_some());
        let first_dtmi = first_create_dtmi_result.unwrap();

        let second_create_dtmi_result: Option<Dtmi> =
            create_dtmi("dtmi:com:example:Thermostat;1.234567");
        assert!(second_create_dtmi_result.is_some());
        let second_dtmi = second_create_dtmi_result.unwrap();

        let third_create_dtmi_result: Option<Dtmi> =
            create_dtmi("dtmi:com:example:Barometer;2.987");
        assert!(third_create_dtmi_result.is_some());
        let third_dtmi = third_create_dtmi_result.unwrap();

        assert!(first_dtmi == second_dtmi);
        assert!(first_dtmi != third_dtmi);
    }
}
