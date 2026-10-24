// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1
// Contact: info@arthiq.net | https://vedaksha.net

//! The wire shape of a Yogini period across the `round` field.
//!
//! `YoginiPeriod::round` was added after the type shipped, and the MCP
//! `compute_dasha` tool serves the type as JSON. Output stored before the
//! field existed must still read back, as round 1 (the only round the tool
//! produced), and output written now must carry the field.

use vedaksha_vedic::dasha::yogini::{self, YoginiPeriod};

#[test]
fn period_json_written_before_the_round_field_reads_back_as_round_one() {
    let legacy = serde_json::json!({
        "lord": "Bhramari",
        "level": 1,
        "start_jd": 2451545.0,
        "end_jd": 2453006.0,
        "duration_days": 1461.0,
        "sub_periods": []
    });
    let period: YoginiPeriod = serde_json::from_value(legacy).expect("legacy JSON reads back");
    assert_eq!(period.round, 1);
    assert_eq!(period.end_jd, 2453006.0);
}

#[test]
fn serialised_periods_carry_their_round() {
    let tree = yogini::compute_yogini_rounds(0.0, 2451545.0, 2, 2);
    let value = serde_json::to_value(&tree).expect("serialises");
    let mahas = value["maha_periods"].as_array().expect("maha_periods");
    assert_eq!(mahas.len(), 16);
    assert_eq!(mahas[0]["round"], 1);
    assert_eq!(mahas[8]["round"], 2);
    // Sub-periods sit in their maha's round.
    assert_eq!(mahas[8]["sub_periods"][0]["round"], 2);

    // And what is written reads back.
    let back: yogini::YoginiDasha = serde_json::from_value(value).expect("round trips");
    assert_eq!(back.maha_periods[15].round, 2);
}
