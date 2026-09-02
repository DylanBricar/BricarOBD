use bricarobd_lib::commands::ecu_scan::{get_ecu_probes, is_valid_ecu_response};
use bricarobd_lib::models::DtcStatus;
use bricarobd_lib::obd::dtc::parse_dtc_response;
use bricarobd_lib::obd::ecu_profiles::{get_did_definitions_for_manufacturer, parse_did_key};
use bricarobd_lib::obd::safety::SafetyGuard;

#[test]
fn rejects_command_separator_injection() {
    for command in ["01 00\r11", "0100\n27", "01\t00", "0100\u{000b}11"] {
        assert!(
            SafetyGuard::validate_hex(command).is_err(),
            "control characters must be rejected: {command:?}"
        );
    }
}

#[test]
fn parses_database_did_keys_as_decimal() {
    assert_eq!(parse_did_key("8288"), Ok(0x2060));
    assert_eq!(parse_did_key("4097"), Ok(0x1001));
    assert_eq!(parse_did_key("257"), Ok(0x0101));
    assert_eq!(parse_did_key("61840"), Ok(0xF190));
}

#[test]
fn manufacturer_dids_are_typed_sorted_and_include_injectors() {
    let dids = get_did_definitions_for_manufacturer("Peugeot");
    assert!(!dids.is_empty());
    assert!(dids.windows(2).all(|pair| pair[0].id <= pair[1].id));

    let injector = dids
        .iter()
        .find(|did| did.name.contains("Injector Correction Cylinder 1"))
        .expect("PSA injector correction DID must exist");
    assert_eq!(injector.id, 0x2060);
}

#[test]
fn parses_each_dtc_ecu_frame_independently() {
    let response = "7E8 06 43 01 23 04 56 00 00\n7E9 04 43 02 00 00 00";
    let dtcs = parse_dtc_response(response, DtcStatus::Active, "OBD", "en");
    let codes: Vec<&str> = dtcs.iter().map(|dtc| dtc.code.as_str()).collect();

    assert_eq!(codes, vec!["P0123", "P0456", "P0200"]);
}

#[test]
fn ecu_scan_rejects_negative_responses_and_rx_addresses() {
    assert!(!is_valid_ecu_response("7F 22 31"));
    assert!(!is_valid_ecu_response("7E8 03 7F 22 31"));
    assert!(!get_ecu_probes().iter().any(|probe| probe.tx_addr == "7E8"));
}
