// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1
// Contact: info@arthiq.net | https://vedaksha.net

//! The lunar nodes, end to end: the value the pipeline reports, and the
//! frame every node longitude is referred to.
//!
//! Both properties were unguarded before this file existed, and both were
//! wrong. The pipeline treated a node as a body one AU away and subtracted
//! Earth's position from it, moving the reported node by up to 75°, while the
//! only test on that path asserted the state vector was *finite*. Separately,
//! `true_node_osculating` was referred to J2000 while `mean_node` and the
//! then-public true-node function (since renamed and made private — see
//! `true_node_meeus_truncated` below) were referred to the equinox of date,
//! so the three arms of the `Body` node selector were not in the same frame
//! — a difference that grows at the precession rate and reached 1.5° at
//! 1900, and which the existing bound of 0.5° over epochs ending in 2026
//! could not distinguish from the bounded difference between a smoothed
//! series and an instantaneous element.

use vedaksha_ephem_core::analytical::AnalyticalProvider;
use vedaksha_ephem_core::bodies::Body;
use vedaksha_ephem_core::coordinates::apparent_position;
use vedaksha_ephem_core::delta_t;
use vedaksha_ephem_core::error::ComputeError;
use vedaksha_ephem_core::jpl::EphemerisProvider;
use vedaksha_ephem_core::nodes;

/// 1900 through 2100, the interval the ephemeris is validated over.
const EPOCHS: [(f64, &str); 6] = [
    (2_415_020.5, "1900-01-01"),
    (2_433_282.5, "1950-01-01"),
    (2_451_545.0, "2000-01-01"),
    (2_458_849.5, "2020-01-01"),
    (2_469_807.5, "2050-01-01"),
    (2_488_069.5, "2100-01-01"),
];

/// Nutation in longitude never exceeds ≈17.2″, and it is the *only* thing
/// that may separate a reported node longitude from the node function.
const NUTATION_BOUND_DEG: f64 = 20.0 / 3600.0;

fn wrap180(mut d: f64) -> f64 {
    while d > 180.0 {
        d -= 360.0;
    }
    while d < -180.0 {
        d += 360.0;
    }
    d
}

/// Independent replica of the 5-term truncated true-node series used only by
/// [`every_node_method_shares_one_frame`] below.
///
/// `vedaksha_ephem_core::nodes` keeps its own copy of this series private and
/// test-only, precisely so it stays an independently-transcribed cross-check for
/// the osculating computation rather than a production code path — which
/// means this integration test needs its own copy to keep doing the same
/// cross-check from outside the crate. Deliberately uses `f64::sin` here
/// rather than the `libm::sin` the module uses internally, so the two
/// implementations share no code path at all.
///
/// Source: Meeus, *Astronomical Algorithms*, 2nd ed., Ch. 47.
fn true_node_meeus_truncated(jd: f64) -> f64 {
    let t = (jd - 2_451_545.0) / 36525.0;
    let mean = nodes::mean_node(jd);

    let d = (297.850_192_1 + 445_267.111_403_4 * t).to_radians();
    let m = (357.529_109_2 + 35_999.050_290_9 * t).to_radians();
    let mp = (134.963_396_4 + 477_198.867_505_5 * t).to_radians();
    let f = (93.272_095_0 + 483_202.017_523_3 * t).to_radians();

    let correction = -1.4979 * (2.0 * (d - f)).sin() - 0.1500 * m.sin() - 0.1226 * (2.0 * d).sin()
        + 0.1176 * (2.0 * f).sin()
        - 0.0801 * (2.0 * (mp - f)).sin();

    let lon = mean + correction;
    lon.rem_euclid(360.0)
}

/// Apparent ecliptic longitude in degrees, through the production pipeline.
///
/// Takes the provider by reference to read like every other call site here,
/// even though `AnalyticalProvider` is zero-sized.
#[allow(clippy::trivially_copy_pass_by_ref)]
fn ecliptic_longitude_deg(provider: &AnalyticalProvider, body: Body, jd_ut: f64) -> f64 {
    apparent_position(provider, body, jd_ut)
        .unwrap_or_else(|e| panic!("{body:?} should compute: {e}"))
        .ecliptic
        .longitude
        .to_degrees()
}

/// The reported longitude must be the node's own longitude.
///
/// This is the test whose absence let a 75° error ship: the previous guard
/// checked that the node's state vector held finite numbers, which stayed
/// true no matter what the pipeline did with it afterwards.
#[test]
fn pipeline_reports_the_node_longitude_itself() {
    let provider = AnalyticalProvider::new();

    for (jd_ut, label) in EPOCHS {
        let jd_tt = delta_t::ut1_to_tt(jd_ut);
        let expected = [
            (Body::MeanNode, nodes::mean_node(jd_tt), "mean"),
            (Body::TrueNode, nodes::true_node_osculating(jd_tt), "true"),
            (
                Body::TrueNodeOsculating,
                nodes::true_node_osculating(jd_tt),
                "osculating",
            ),
        ];

        for (body, node_longitude, name) in expected {
            let pos = apparent_position(&provider, body, jd_ut)
                .unwrap_or_else(|e| panic!("{label}: {name} node should compute: {e}"));
            let reported = pos.ecliptic.longitude.to_degrees();
            let diff = wrap180(reported - node_longitude).abs();

            assert!(
                diff < NUTATION_BOUND_DEG,
                "{label}: {name} node reported as {reported:.4}° but the node is at \
                 {node_longitude:.4}° — off by {diff:.4}° ({:.1}″). Only nutation in \
                 longitude (≤17.2″) may separate these; anything larger means the node \
                 is being routed through the geocentric/light-time path again.",
                diff * 3600.0
            );

            // A node lies in the ecliptic by definition and is a direction,
            // not a place. Both were nonzero while the node was being treated
            // as a body: latitude drifted and `distance` reported the
            // meaningless Earth-to-unit-vector separation.
            assert!(
                pos.ecliptic.latitude == 0.0,
                "{label}: {name} node latitude should be exactly 0, got {}",
                pos.ecliptic.latitude
            );
            assert!(
                pos.ecliptic.distance == 0.0,
                "{label}: {name} node distance should be exactly 0, got {}",
                pos.ecliptic.distance
            );
        }
    }
}

/// The nodes regress along the ecliptic once per draconic period.
///
/// An independent physical check on the reported motion: the nodal
/// regression period is 6798.38 days, so the nodes drift backwards at
/// −360/6798.38 = −0.05295°/day. The pipeline reported +0.38°/day (and
/// +0.48, and −19.2) while it was differencing the apparent motion of a
/// fictitious body one AU away.
///
/// Only the *mean* node moves at that rate instantaneously. The true and
/// osculating nodes librate about it — by ≈1.5° and ≈0.3° respectively, over
/// roughly half a month — so their daily motion swings either side of the
/// mean rate and is genuinely direct for part of each cycle. Their secular
/// rate is what must match, so they are averaged over a span long enough to
/// contain many librations.
#[test]
fn nodes_regress_at_the_draconic_rate() {
    let provider = AnalyticalProvider::new();
    let draconic = -360.0 / 6798.383;

    // The mean node, instantaneously, at every epoch.
    for (jd_ut, label) in EPOCHS {
        let speed = apparent_position(&provider, Body::MeanNode, jd_ut)
            .expect("mean node should compute")
            .longitude_speed;
        assert!(
            (speed - draconic).abs() < 0.005 * draconic.abs(),
            "{label}: mean node speed {speed:+.6}°/day is not the draconic rate \
             ({draconic:+.6}°/day)"
        );
    }

    // All three nodes, by the distance they actually travel. Averaging
    // instantaneous speeds instead would alias: any step near the 27.2-day
    // draconic month resamples the same libration phase and biases the mean.
    // A displacement over a fixed baseline has no such sensitivity.
    let baseline = 1000.0;
    let expected_regression = draconic * baseline; // −52.98°
    for body in [Body::MeanNode, Body::TrueNode, Body::TrueNodeOsculating] {
        let start = apparent_position(&provider, body, 2_451_545.0)
            .expect("node should compute")
            .ecliptic
            .longitude
            .to_degrees();
        let end = apparent_position(&provider, body, 2_451_545.0 + baseline)
            .expect("node should compute")
            .ecliptic
            .longitude
            .to_degrees();
        let travelled = wrap180(end - start);

        // The mean node is smooth; the other two are sampled at an arbitrary
        // libration phase at each end, so they carry that amplitude twice.
        let tolerance = if body == Body::MeanNode { 0.01 } else { 2.0 };
        assert!(
            (travelled - expected_regression).abs() < tolerance,
            "{body:?} moved {travelled:+.4}° in {baseline} days; the draconic rate \
             requires {expected_regression:+.4}° (±{tolerance}°)"
        );
    }
}

/// All three node methods must be referred to the same frame.
///
/// The osculating node and the Meeus 5-term series differ because one is an
/// instantaneous orbital element and the other is smoothed — that difference
/// is *bounded*. A frame difference is not: it grows without limit at the
/// precession rate. Pinning the bound at 1900 and 2100, where a frame term
/// would be ±1.5°, is what distinguishes the two.
#[test]
fn every_node_method_shares_one_frame() {
    // Densely sampled rather than spot-checked: the osculating node librates
    // about the smoothed series, so a sparse grid can sit in the quiet part
    // of the cycle. 3-day steps over two centuries.
    let mut jd = 2_415_020.5;
    let (mut worst, mut worst_jd) = (0.0_f64, 0.0_f64);
    while jd <= 2_488_069.5 {
        let diff = wrap180(nodes::true_node_osculating(jd) - true_node_meeus_truncated(jd)).abs();
        if diff > worst {
            worst = diff;
            worst_jd = jd;
        }
        jd += 3.0;
    }

    assert!(
        worst < 0.35,
        "osculating and Meeus true node diverge by {worst:.4}° at JD {worst_jd} — \
         the libration of the instantaneous node about the smoothed series is \
         ≈0.3°, so anything larger is a frame difference, not a method difference"
    );
}

/// The J2000 variant exists, is named for its frame, and carries the frame
/// term the of-date functions do not.
///
/// This is the other half of the guard above: it asserts the difference *is*
/// there in [`nodes::true_node_osculating_j2000`], so the two functions
/// cannot be silently collapsed into one.
#[test]
fn the_j2000_variant_carries_the_precession_term() {
    // Equinox precession alone is ≈50.3″/yr; the reference *plane* also
    // moves, and because the Moon's orbit is inclined 5.14° that tilt is
    // amplified by cot(i) ≈ 11 where it meets the node. The combined term is
    // ≈1.5° per century, which these bounds bracket.
    let cases = [
        (2_415_020.5, "1900-01-01", 1.40, 1.70),
        (2_469_807.5, "2050-01-01", 0.65, 0.90),
        (2_488_069.5, "2100-01-01", 1.30, 1.60),
    ];

    for (jd, label, lo, hi) in cases {
        let of_date = nodes::true_node_osculating(jd);
        let j2000 = nodes::true_node_osculating_j2000(jd);
        let diff = wrap180(j2000 - of_date).abs();
        assert!(
            diff > lo && diff < hi,
            "{label}: J2000 and of-date osculating nodes differ by {diff:.4}°, \
             expected {lo}–{hi}° of accumulated frame term"
        );
    }

    // At J2000 itself the term vanishes, which is exactly why testing there
    // alone proved nothing.
    let at_epoch = wrap180(
        nodes::true_node_osculating_j2000(2_451_545.0) - nodes::true_node_osculating(2_451_545.0),
    )
    .abs();
    assert!(
        at_epoch < 0.001,
        "the two frames must coincide at J2000, differ by {at_epoch:.6}°"
    );
}

/// A node is a direction, so it has no state vector and the provider must
/// say so rather than synthesising one.
#[test]
fn nodes_have_no_state_vector() {
    let provider = AnalyticalProvider::new();
    for body in [Body::MeanNode, Body::TrueNode, Body::TrueNodeOsculating] {
        match provider.compute_state(body, 2_451_545.0) {
            Err(ComputeError::BodyNotAvailable { body_id }) => {
                assert_eq!(body_id, body.naif_id());
            }
            other => panic!(
                "{body:?} must not produce a state vector — a synthetic unit vector \
                 is what the geocentric pipeline mistook for a body. Got {other:?}"
            ),
        }
    }
}

/// Ketu is Rahu's opposite point, and the pipeline must preserve that.
///
/// Through v7.1.1 there was no south-node variant of [`Body`] at all, so the
/// only Ketu available was `nodes::south_node_*`, which is the theory layer and
/// carries no nutation. A caller pairing that with a `Body`-routed Rahu got two
/// values up to 17.2 arcsec off 180 degrees apart. Both ends now go through the
/// same pipeline, so the separation is exact.
#[test]
fn every_ketu_is_exactly_opposite_its_rahu_through_the_pipeline() {
    let provider = AnalyticalProvider::new();

    // The osculating pair costs three ELP/MPP02 series evaluations per call
    // for its finite difference, which is why it gets one epoch and the two
    // cheap methods get four. One epoch is enough for what this test is
    // actually guarding: that the dispatch arm exists and points at the right
    // function. The value itself is `north + 180` by construction.
    let cheap = [
        (Body::MeanNode, Body::MeanSouthNode, "mean"),
        (Body::TrueNode, Body::TrueSouthNode, "true"),
    ];
    let pricey = [(
        Body::TrueNodeOsculating,
        Body::TrueSouthNodeOsculating,
        "osculating",
    )];

    let check = |north, south, label: &str, jd: f64| {
        let n = ecliptic_longitude_deg(&provider, north, jd);
        let s = ecliptic_longitude_deg(&provider, south, jd);
        let sep = (s - n + 360.0) % 360.0;
        assert!(
            (sep - 180.0).abs() < 1e-9,
            "{label} at jd {jd}: rahu {n}, ketu {s}, separation {sep}"
        );
    };

    for jd in [2_415_020.5, 2_451_545.0, 2_460_000.5, 2_488_070.0] {
        for (north, south, label) in cheap {
            check(north, south, label, jd);
        }
    }
    for (north, south, label) in pricey {
        check(north, south, label, 2_451_545.0);
    }
}

/// The south nodes must land in the same frame as the north ones — that is the
/// whole point of putting them behind the enum. If Ketu were resolved anywhere
/// but `nodes::node_longitude`, it would drift from Rahu at the precession
/// rate, which is what v7.0.0 fixed for the north nodes.
#[test]
fn the_south_nodes_regress_with_the_north_ones() {
    let provider = AnalyticalProvider::new();
    let jd0 = 2_451_545.0;
    let jd1 = jd0 + 1000.0;
    for (north, south, label) in [
        (Body::MeanNode, Body::MeanSouthNode, "mean"),
        (Body::TrueNode, Body::TrueSouthNode, "true"),
    ] {
        let dn = ecliptic_longitude_deg(&provider, north, jd1)
            - ecliptic_longitude_deg(&provider, north, jd0);
        let ds = ecliptic_longitude_deg(&provider, south, jd1)
            - ecliptic_longitude_deg(&provider, south, jd0);
        let dn = (dn + 540.0) % 360.0 - 180.0;
        let ds = (ds + 540.0) % 360.0 - 180.0;
        assert!(
            (dn - ds).abs() < 1e-9,
            "{label}: rahu moved {dn}deg over 1000d, ketu moved {ds}deg"
        );
    }
}

/// Ketu is a direction like Rahu: no state vector, from any provider.
#[test]
fn the_south_nodes_have_no_state_vector() {
    let provider = AnalyticalProvider::new();
    for body in [
        Body::MeanSouthNode,
        Body::TrueSouthNode,
        Body::TrueSouthNodeOsculating,
    ] {
        assert!(
            provider.compute_state(body, 2_451_545.0).is_err(),
            "{body:?} returned a state vector"
        );
        assert!(
            body.de441_component_index().is_none(),
            "{body:?} claims DE441 data"
        );
    }
}
