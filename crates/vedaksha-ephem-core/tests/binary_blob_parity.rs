// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.

//! Numerical-parity test for the v2.6.0 binary-blob refactor.
//!
//! Pins ten records from each of three representative tables — VSOP87A
//! Mercury X0, ELP/MPP02 lunar-distance MAIN, ELP/MPP02 lunar-longitude
//! PERT_0 — against the values produced by the [`LazyLock`]-driven blob
//! loader. The hard-coded samples are copies of the original tuple
//! literals that lived in the pre-refactor `.rs` tables (extracted from
//! the committed `.bin` blobs at refactor time), so this test catches any
//! drift between the on-disk blob format and the loader, as well as any
//! accidental regeneration of the blobs with a different threshold.
//!
//! Fast: parses 10 records out of each table and runs in well under a
//! second. Safe for routine CI.

use vedaksha_ephem_core::analytical::coefficients::loader::{ElpMainTerm, ElpPertTerm, Vsop87Term};
use vedaksha_ephem_core::analytical::coefficients::{mercury, moon_distance, moon_longitude};

#[test]
fn vsop87a_mercury_x0_first_10_records_match_committed_values() {
    let expected: [Vsop87Term; 10] = [
        Vsop87Term {
            amplitude: 3.754_629_172_799_999_75e-01,
            phase: 4.396_515_069_420_000_37e+00,
            frequency: 2.608_790_314_157_419_93e+04,
        },
        Vsop87Term {
            amplitude: 3.825_746_672_000_000_10e-02,
            phase: 1.164_856_043_389_999_93e+00,
            frequency: 5.217_580_628_314_839_85e+04,
        },
        Vsop87Term {
            amplitude: 2.625_615_963_000_000_12e-02,
            phase: 3.141_592_653_590_000_06e+00,
            frequency: 0.000_000_000_000_000_00e+00,
        },
        Vsop87Term {
            amplitude: 5.842_613_330_000_000_39e-03,
            phase: 4.215_993_947_570_000_34e+00,
            frequency: 7.826_370_942_472_259_05e+04,
        },
        Vsop87Term {
            amplitude: 1.057_166_949_999_999_90e-03,
            phase: 9.837_903_318_199_999_75e-01,
            frequency: 1.043_516_125_662_967_82e+05,
        },
        Vsop87Term {
            amplitude: 2.101_173_000_000_000_02e-04,
            phase: 4.034_693_539_230_000_07e+00,
            frequency: 1.304_395_157_078_709_89e+05,
        },
        Vsop87Term {
            amplitude: 4.433_373_000_000_000_08e-05,
            phase: 8.023_667_452_699_999_59e-01,
            frequency: 1.565_274_188_494_451_81e+05,
        },
        Vsop87Term {
            amplitude: 9.749_670_000_000_000_46e-06,
            phase: 3.853_196_745_360_000_01e+00,
            frequency: 1.826_153_219_910_193_87e+05,
        },
        Vsop87Term {
            amplitude: 7.003_269_999_999_999_95e-06,
            phase: 4.454_787_253_670_000_17e+00,
            frequency: 2.497_852_458_948_079_95e+04,
        },
        Vsop87Term {
            amplitude: 6.264_679_999_999_999_79e-06,
            phase: 1.185_634_920_010_000_04e+00,
            frequency: 2.719_728_169_366_759_90e+04,
        },
    ];

    let actual = &mercury::X0;
    assert!(
        actual.len() >= expected.len(),
        "mercury X0 has only {} records",
        actual.len()
    );
    for (i, e) in expected.iter().enumerate() {
        let a = actual[i];
        assert_eq!(
            a.amplitude.to_bits(),
            e.amplitude.to_bits(),
            "mercury X0[{i}] amplitude bits differ"
        );
        assert_eq!(
            a.phase.to_bits(),
            e.phase.to_bits(),
            "mercury X0[{i}] phase bits differ"
        );
        assert_eq!(
            a.frequency.to_bits(),
            e.frequency.to_bits(),
            "mercury X0[{i}] frequency bits differ"
        );
    }
}

#[test]
fn elp_moon_distance_main_first_10_records_match_committed_values() {
    let expected: [ElpMainTerm; 10] = [
        ElpMainTerm {
            i1: 0,
            i2: 0,
            i3: 0,
            i4: 0,
            amp: 385_000.527_19,
            b1: -7_992.63,
            b2: -11.06,
            b3: 21_578.08,
            b4: -4.53,
            b5: 11.39,
            b6: -0.06,
        },
        ElpMainTerm {
            i1: 0,
            i2: 2,
            i3: 0,
            i4: 0,
            amp: -3.148_37,
            b1: -204.48,
            b2: -138.94,
            b3: 159.64,
            b4: -0.39,
            b5: 0.12,
            b6: 0.00,
        },
        ElpMainTerm {
            i1: 0,
            i2: 4,
            i3: 0,
            i4: 0,
            amp: -3.0e-05,
            b1: 0.00,
            b2: 0.00,
            b3: 0.00,
            b4: 0.00,
            b5: 0.00,
            b6: 0.00,
        },
        ElpMainTerm {
            i1: 0,
            i2: -4,
            i3: 1,
            i4: 0,
            amp: 0.000_38,
            b1: 0.03,
            b2: 0.03,
            b3: -0.03,
            b4: 0.00,
            b5: 0.00,
            b6: 0.00,
        },
        ElpMainTerm {
            i1: 0,
            i2: -2,
            i3: 1,
            i4: 0,
            amp: 79.661_83,
            b1: -359.45,
            b2: 3_583.79,
            b3: 1_454.02,
            b4: -2.37,
            b5: 0.85,
            b6: 0.00,
        },
        ElpMainTerm {
            i1: 0,
            i2: 0,
            i3: 1,
            i4: 0,
            amp: -20_905.322_06,
            b1: 6_888.23,
            b2: -35.83,
            b3: -380_331.74,
            b4: 22.31,
            b5: 1.77,
            b6: 0.00,
        },
        ElpMainTerm {
            i1: 0,
            i2: 2,
            i3: 1,
            i4: 0,
            amp: -0.103_26,
            b1: -9.14,
            b2: -4.53,
            b3: 8.08,
            b4: 0.00,
            b5: 0.02,
            b6: 0.00,
        },
        ElpMainTerm {
            i1: 0,
            i2: -4,
            i3: 2,
            i4: 0,
            amp: -0.008_37,
            b1: 0.07,
            b2: -0.75,
            b3: -0.31,
            b4: 0.00,
            b5: 0.00,
            b6: 0.00,
        },
        ElpMainTerm {
            i1: 0,
            i2: -2,
            i3: 2,
            i4: 0,
            amp: -4.421_24,
            b1: 18.29,
            b2: -198.91,
            b3: -161.90,
            b4: 0.14,
            b5: -0.04,
            b6: 0.00,
        },
        ElpMainTerm {
            i1: 0,
            i2: 0,
            i3: 2,
            i4: 0,
            amp: -569.923_32,
            b1: 374.44,
            b2: -1.99,
            b3: -20_737.33,
            b4: 5.79,
            b5: 0.44,
            b6: 0.00,
        },
    ];

    let actual = &moon_distance::MAIN;
    for (i, e) in expected.iter().enumerate() {
        let a = actual[i];
        assert_eq!(a.i1, e.i1, "moon_distance MAIN[{i}] i1");
        assert_eq!(a.i2, e.i2, "moon_distance MAIN[{i}] i2");
        assert_eq!(a.i3, e.i3, "moon_distance MAIN[{i}] i3");
        assert_eq!(a.i4, e.i4, "moon_distance MAIN[{i}] i4");
        assert_eq!(
            a.amp.to_bits(),
            e.amp.to_bits(),
            "moon_distance MAIN[{i}] amp"
        );
        assert_eq!(a.b1.to_bits(), e.b1.to_bits(), "moon_distance MAIN[{i}] b1");
        assert_eq!(a.b2.to_bits(), e.b2.to_bits(), "moon_distance MAIN[{i}] b2");
        assert_eq!(a.b3.to_bits(), e.b3.to_bits(), "moon_distance MAIN[{i}] b3");
        assert_eq!(a.b4.to_bits(), e.b4.to_bits(), "moon_distance MAIN[{i}] b4");
        assert_eq!(a.b5.to_bits(), e.b5.to_bits(), "moon_distance MAIN[{i}] b5");
        assert_eq!(a.b6.to_bits(), e.b6.to_bits(), "moon_distance MAIN[{i}] b6");
    }
}

#[test]
fn elp_moon_longitude_pert_0_first_10_records_match_committed_values() {
    let expected: [ElpPertTerm; 10] = [
        ElpPertTerm {
            s: -12.749_215_540_86,
            c: 6.368_794_709_728,
            i1: 0,
            i2: 0,
            i3: 1,
            i4: 0,
            i5: 0,
            i6: -18,
            i7: 16,
            i8: 0,
            i9: 0,
            i10: 0,
            i11: 0,
            i12: 0,
            i13: 0,
        },
        ElpPertTerm {
            s: -7.062_989_999_049,
            c: 1.158_760_846_981e-04,
            i1: 0,
            i2: 1,
            i3: 0,
            i4: 0,
            i5: 0,
            i6: 0,
            i7: 0,
            i8: 0,
            i9: 0,
            i10: 0,
            i11: 0,
            i12: 0,
            i13: -1,
        },
        ElpPertTerm {
            s: -1.142_992_395_166,
            c: -2.364_916_989_22e-03,
            i1: 2,
            i2: 0,
            i3: -1,
            i4: 0,
            i5: 0,
            i6: 0,
            i7: 2,
            i8: 0,
            i9: -2,
            i10: 0,
            i11: 0,
            i12: 0,
            i13: 0,
        },
        ElpPertTerm {
            s: 0.236_351_883_255,
            c: -0.843_625_983_051_5,
            i1: 0,
            i2: 0,
            i3: 0,
            i4: 0,
            i5: 0,
            i6: 0,
            i7: 4,
            i8: -8,
            i9: 3,
            i10: 0,
            i11: 0,
            i12: 0,
            i13: 0,
        },
        ElpPertTerm {
            s: -0.705_249_093_238_2,
            c: 0.352_301_595_746_9,
            i1: 0,
            i2: 0,
            i3: 2,
            i4: 0,
            i5: 0,
            i6: -18,
            i7: 16,
            i8: 0,
            i9: 0,
            i10: 0,
            i11: 0,
            i12: 0,
            i13: 0,
        },
        ElpPertTerm {
            s: 0.661_604_270_163_3,
            c: 0.330_511_860_642_9,
            i1: 0,
            i2: 0,
            i3: 0,
            i4: 0,
            i5: 0,
            i6: 18,
            i7: -16,
            i8: 0,
            i9: 0,
            i10: 0,
            i11: 0,
            i12: 0,
            i13: 0,
        },
        ElpPertTerm {
            s: -0.821_631_727_287_4,
            c: -1.489_776_891_248e-04,
            i1: 0,
            i2: 0,
            i3: 0,
            i4: 0,
            i5: 0,
            i6: 1,
            i7: -1,
            i8: 0,
            i9: 0,
            i10: 0,
            i11: 0,
            i12: 0,
            i13: 0,
        },
        ElpPertTerm {
            s: -0.519_936_838_805_2,
            c: -0.220_976_862_400_9,
            i1: 0,
            i2: 0,
            i3: 1,
            i4: 0,
            i5: 0,
            i6: -10,
            i7: 3,
            i8: 0,
            i9: 0,
            i10: 0,
            i11: 0,
            i12: 0,
            i13: 0,
        },
        ElpPertTerm {
            s: 0.638_690_045_411_9,
            c: 1.372_171_387_329e-02,
            i1: 0,
            i2: 0,
            i3: 0,
            i4: 0,
            i5: 0,
            i6: 0,
            i7: 1,
            i8: 0,
            i9: -1,
            i10: 0,
            i11: 0,
            i12: 0,
            i13: 0,
        },
        ElpPertTerm {
            s: -0.643_749_334_223_8,
            c: 1.506_773_061_91e-04,
            i1: 2,
            i2: 0,
            i3: -1,
            i4: 0,
            i5: 0,
            i6: 3,
            i7: -3,
            i8: 0,
            i9: 0,
            i10: 0,
            i11: 0,
            i12: 0,
            i13: 0,
        },
    ];

    let actual = &moon_longitude::PERT_0;
    for (i, e) in expected.iter().enumerate() {
        let a = actual[i];
        assert_eq!(a.s.to_bits(), e.s.to_bits(), "moon_longitude PERT_0[{i}] s");
        assert_eq!(a.c.to_bits(), e.c.to_bits(), "moon_longitude PERT_0[{i}] c");
        assert_eq!(a.i1, e.i1, "moon_longitude PERT_0[{i}] i1");
        assert_eq!(a.i2, e.i2, "moon_longitude PERT_0[{i}] i2");
        assert_eq!(a.i3, e.i3, "moon_longitude PERT_0[{i}] i3");
        assert_eq!(a.i4, e.i4, "moon_longitude PERT_0[{i}] i4");
        assert_eq!(a.i5, e.i5, "moon_longitude PERT_0[{i}] i5");
        assert_eq!(a.i6, e.i6, "moon_longitude PERT_0[{i}] i6");
        assert_eq!(a.i7, e.i7, "moon_longitude PERT_0[{i}] i7");
        assert_eq!(a.i8, e.i8, "moon_longitude PERT_0[{i}] i8");
        assert_eq!(a.i9, e.i9, "moon_longitude PERT_0[{i}] i9");
        assert_eq!(a.i10, e.i10, "moon_longitude PERT_0[{i}] i10");
        assert_eq!(a.i11, e.i11, "moon_longitude PERT_0[{i}] i11");
        assert_eq!(a.i12, e.i12, "moon_longitude PERT_0[{i}] i12");
        assert_eq!(a.i13, e.i13, "moon_longitude PERT_0[{i}] i13");
    }
}
