// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

/// Supported digital twin operations.
pub mod digital_twin_operation {
    pub const GET: &str = "Get";
    pub const SET: &str = "Set";
    pub const SUBSCRIBE: &str = "Subscribe";
    pub const UNSUBSCRIBE: &str = "Unsubscribe";
    pub const INVOKE: &str = "Invoke";
}

// Supported gitial twin protocols.
pub mod digital_twin_protocol {
    pub const GRPC: &str = "grpc";
}

/// Is the provided subset a subset of the provided superset?
///
/// # Arguments
/// `subset` - The provided subset.
/// `superset` - The provided superset.
pub fn is_subset(subset: &[String], superset: &[String]) -> bool {
    subset.iter().all(|subset_member| {
        superset.iter().any(|supserset_member| subset_member == supserset_member)
    })
}

#[cfg(test)]
mod ibeji_common_tests {
    use super::*;

    #[test]
    fn is_subset_test() {
        assert!(is_subset(&vec!(), &vec!()));
        assert!(is_subset(&vec!(), &vec!("one".to_string())));
        assert!(is_subset(&vec!(), &vec!("one".to_string(), "two".to_string())));
        assert!(is_subset(&vec!("one".to_string()), &vec!("one".to_string(), "two".to_string())));
        assert!(is_subset(
            &vec!("one".to_string(), "two".to_string()),
            &vec!("one".to_string(), "two".to_string())
        ));
        assert!(!is_subset(
            &vec!("one".to_string(), "two".to_string(), "three".to_string()),
            &vec!("one".to_string(), "two".to_string())
        ));
        assert!(!is_subset(
            &vec!("one".to_string(), "two".to_string(), "three".to_string()),
            &vec!("one".to_string())
        ));
        assert!(!is_subset(
            &vec!("one".to_string(), "two".to_string(), "three".to_string()),
            &vec!()
        ));
    }
}
