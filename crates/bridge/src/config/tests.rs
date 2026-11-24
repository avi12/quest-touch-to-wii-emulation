use std::collections::HashMap;
use std::env::VarError;
use std::path::PathBuf;

use openvr_sys::Hand;

use dsu::Port;

use crate::config::{Config, ConfigError, PointerSource};

fn parse(pairs: &[(&str, &str)]) -> Result<Config, ConfigError> {
    let table: HashMap<String, String> = pairs
        .iter()
        .map(|(name, value)| ((*name).to_owned(), (*value).to_owned()))
        .collect();
    Config::from_lookup(|name| table.get(name).cloned().ok_or(VarError::NotPresent))
}

#[test]
fn defaults_to_the_right_controller_on_26760() {
    let config = parse(&[]).unwrap();
    assert_eq!(config.port, Port::DEFAULT);
    assert_eq!(config.port.get(), 26760);
    assert_eq!(config.hand, Hand::Right);
    assert!(!config.is_player2());
    assert_eq!(config.pointer_source, PointerSource::Controller);
    assert_eq!(config.telemetry_path, None);
}

#[test]
fn reads_the_second_instance() {
    let config = parse(&[
        ("BRIDGE_PORT", " 26761 "),
        ("BRIDGE_HAND", "left"),
        ("PTR_TEST", "1"),
        ("BRIDGE_TELEMETRY", "t.jsonl"),
    ])
    .unwrap();
    assert_eq!((config.port.get(), config.hand), (26761, Hand::Left));
    assert!(config.is_player2());
    assert_eq!(config.pointer_source, PointerSource::Sweep);
    assert_eq!(config.telemetry_path, Some(PathBuf::from("t.jsonl")));
}

#[test]
fn empty_flags_count_as_unset() {
    let config = parse(&[("PTR_TEST", ""), ("BRIDGE_TELEMETRY", "")]).unwrap();
    assert_eq!(config.pointer_source, PointerSource::Controller);
    assert_eq!(config.telemetry_path, None);
}

#[test]
fn rejects_what_zod_rejects() {
    for port in ["0", "65536", "1.5", "abc", "", "-1"] {
        let error = parse(&[("BRIDGE_PORT", port)]).unwrap_err();
        assert_eq!(
            (error.variable, error.received.as_str()),
            ("BRIDGE_PORT", port)
        );
    }
    for hand in ["Left", "", "both"] {
        assert_eq!(
            parse(&[("BRIDGE_HAND", hand)]).unwrap_err().variable,
            "BRIDGE_HAND"
        );
    }
    assert_eq!(parse(&[("BRIDGE_PORT", "1e3")]).unwrap().port.get(), 1000);
}

#[test]
fn error_names_the_variable_and_value() {
    let error = parse(&[("BRIDGE_PORT", "abc")]).unwrap_err();
    assert_eq!(
        error.to_string(),
        "BRIDGE_PORT must be an integer from 1 to 65535, got \"abc\""
    );
}
