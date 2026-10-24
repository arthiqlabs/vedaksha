// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1
// Contact: info@arthiq.net | https://vedaksha.net

//! The Moon's light-time-corrected reads: unchanged answers, fewer series
//! evaluations, and the analytic-speed read.
//!
//! The light-time iteration for the Moon stopped re-evaluating the series at
//! an instant it had already evaluated, and takes its first step from the
//! distance series alone where the provider offers it
//! (`EphemerisProvider::moon_distance`). Both are claimed to change no output
//! bit. The bit-for-bit comparison against a verbatim copy of the old
//! iteration lives in `coordinates.rs` (`moon_light_time_tests`), where it can
//! reach the private frame; this file holds what the public API can show:
//!
//! - a golden table of 9.4.1's own output, captured before the change;
//! - the evaluation counts the change exists to lower;
//! - the accuracy of `moon_apparent_position`'s analytic speed.

use std::sync::atomic::Ordering;
use std::sync::{Mutex, MutexGuard};

use vedaksha_ephem_core::analytical::AnalyticalProvider;
use vedaksha_ephem_core::analytical::elp_mpp02::{ELP_DISTANCE_CALLS, ELP_GEOCENTRIC_CALLS};
use vedaksha_ephem_core::bodies::Body;
use vedaksha_ephem_core::coordinates::{
    apparent_position, apparent_positions, ecliptic_position, ecliptic_position_tt,
    moon_apparent_position,
};
use vedaksha_ephem_core::delta_t;

/// `n` UT1 instants spread over 1900 to 2100, the fractional day stepped by
/// the golden ratio so no two share a time of day.
fn instants(n: u32) -> impl Iterator<Item = f64> {
    let start = 2_415_020.5;
    let span = 73_048.0;
    (0..n).map(move |i| {
        let frac = (f64::from(i) * 0.618_033_988_749_894_9).fract();
        start + span * f64::from(i) / f64::from(n) + frac
    })
}

/// 9.4.1's Moon output at 48 instants, 1900 to 2100, captured on aarch64 from
/// the published 9.4.1 code before this change: `(jd bits, ecliptic_position
/// [longitude, latitude, distance] bits, apparent_positions [longitude,
/// latitude, distance, longitude_speed] bits)`.
///
/// aarch64 only, because the series' SIMD trig kernel rounds differently on
/// x86_64 (see `analytical_bit_digest.rs`). The architecture-independent proof
/// is the comparison against the verbatim old iteration in `coordinates.rs`;
/// this table guards that copy against drifting along with the code.
#[cfg(target_arch = "aarch64")]
const GOLDEN_9_4_1: [(u64, [u64; 3], [u64; 4]); 48] = [
    (
        0x41426cd640000000,
        [0x401304ab571a2234, 0x3f93ce77bec038ad, 0x3f642c4ef47c17df],
        [
            0x401304ab571a2234,
            0x3f93ce77bec038ad,
            0x3f642c4ef47c17df,
            0x402ca38259dbd4f3,
        ],
    ),
    (
        0x41426fcf2f5c28f6,
        [0x400816253c9ee718, 0xbf806ddf058fc7a3, 0x3f63a98ddbd0fafd],
        [
            0x400816253c9ee718,
            0xbf806ddf058fc7a3,
            0x3f63a98ddbd0fafd,
            0x402e165db5a799a5,
        ],
    ),
    (
        0x414272c81eb851ec,
        [0x3ff2e0afc22c0e46, 0xbfa60060e4db4652, 0x3f65443d2157d5df],
        [
            0x3ff2e0afc22c0e46,
            0xbfa60060e4db4652,
            0x3f65443d2157d5df,
            0x4029ce5b793d3b5e,
        ],
    ),
    (
        0x414275c10e147ae1,
        [0x4015e636743ea948, 0xbfb41223e5b14586, 0x3f663aba5aa13246],
        [
            0x4015e636743ea948,
            0xbfb41223e5b14586,
            0x3f663aba5aa13246,
            0x4027b4386e263d46,
        ],
    ),
    (
        0x414278b9fd70a3d7,
        [0x400c43b44fae58a6, 0xbfb6f28f64270317, 0x3f6535df20731641],
        [
            0x400c43b44fae58a6,
            0xbfb6f28f64270317,
            0x3f6535df20731641,
            0x402a0b61759b8e94,
        ],
    ),
    (
        0x41427bb2eccccccd,
        [0x3ffca3973f8cf63b, 0xbfb59a026e8c64c4, 0x3f641d5fd08eae5c],
        [
            0x3ffca3973f8cf63b,
            0xbfb59a026e8c64c4,
            0x3f641d5fd08eae5c,
            0x402cd6aa7258a728,
        ],
    ),
    (
        0x41427eabdc28f5c3,
        [0x3f96c6d719c227bf, 0xbfb1102ed6eb69c6, 0x3f6509686aec4627],
        [
            0x3f96c6d719c227bf,
            0xbfb1102ed6eb69c6,
            0x3f6509686aec4627,
            0x402a37791af94d65,
        ],
    ),
    (
        0x414281a450c49ba6,
        [0x401083f6d70ee7fd, 0xbf6e329831b65903, 0x3f65fcc60ffe81c1],
        [
            0x401083f6d70ee7fd,
            0xbf6e329831b65903,
            0x3f65fcc60ffe81c1,
            0x4027eabbef5b4f5a,
        ],
    ),
    (
        0x4142849d4020c49c,
        [0x4000d1ffa756237e, 0x3fa9dd0663e6b00c, 0x3f65d07f734cef05],
        [
            0x4000d1ffa756237e,
            0x3fa9dd0663e6b00c,
            0x3f65d07f734cef05,
            0x40285a1b684c842d,
        ],
    ),
    (
        0x414287962f7ced92,
        [0x3fcecbc32796c30f, 0x3fb48b9ff1dd45de, 0x3f6477a23c4159bc],
        [
            0x3fcecbc32796c30f,
            0x3fb48b9ff1dd45de,
            0x3f6477a23c4159bc,
            0x402bca7f8b2d371a,
        ],
    ),
    (
        0x41428a8f1ed91687,
        [0x4013462fe98754ad, 0x3fb6e75ceab28914, 0x3f6485da3fb74c53],
        [
            0x4013462fe98754ad,
            0x3fb6e75ceab28914,
            0x3f6485da3fb74c53,
            0x402bc028d01f3957,
        ],
    ),
    (
        0x41428d880e353f7d,
        [0x4007c3ab8b3fcdf2, 0x3fb6515f8df183d7, 0x3f65f30eb2ff976a],
        [
            0x4007c3ab8b3fcdf2,
            0x3fb6515f8df183d7,
            0x3f65f30eb2ff976a,
            0x402848409a44ec7a,
        ],
    ),
    (
        0x41429080fd916873,
        [0x3feec80c0fc1df31, 0x3fae5a95f335815d, 0x3f65f664a1108f26],
        [
            0x3feec80c0fc1df31,
            0x3fae5a95f335815d,
            0x3f65f664a1108f26,
            0x40282d4c979b033f,
        ],
    ),
    (
        0x41429379eced9169,
        [0x401532d3ab9b5e34, 0x3f90eed84537b1a2, 0x3f646b1c3f1e8921],
        [
            0x401532d3ab9b5e34,
            0x3f90eed84537b1a2,
            0x3f646b1c3f1e8921,
            0x402bdcfbd144a11f,
        ],
    ),
    (
        0x414296726189374c,
        [0x400a5c784b07cbac, 0xbfa221f9935e5d5a, 0x3f63bc32c20b1cc9],
        [
            0x400a5c784b07cbac,
            0xbfa221f9935e5d5a,
            0x3f63bc32c20b1cc9,
            0x402ddd352a00547e,
        ],
    ),
    (
        0x4142996b50e56042,
        [0x3ff8a5a951ae75cf, 0xbfae801a48eb9082, 0x3f64f9a3875a0d38],
        [
            0x3ff8a5a951ae75cf,
            0xbfae801a48eb9082,
            0x3f64f9a3875a0d38,
            0x402a9c052828b2d9,
        ],
    ),
    (
        0x41429c6440418938,
        [0x40178822f43a780d, 0xbfb54e9e58f73bb1, 0x3f663c8f15361368],
        [
            0x40178822f43a780d,
            0xbfb54e9e58f73bb1,
            0x3f663c8f15361368,
            0x4027c8a419cce30f,
        ],
    ),
    (
        0x41429f5d2f9db22d,
        [0x400f4a4a4663c1d6, 0xbfb59abba22264dc, 0x3f656cd937afbffa],
        [
            0x400f4a4a4663c1d6,
            0xbfb59abba22264dc,
            0x3f656cd937afbffa,
            0x40299f5637d3ee3f,
        ],
    ),
    (
        0x4142a2561ef9db23,
        [0x4000d18837e79a05, 0xbfb1f83ef4a74189, 0x3f63d02384123e2f],
        [
            0x4000d18837e79a05,
            0xbfb1f83ef4a74189,
            0x3f63d02384123e2f,
            0x402dc95b4abdbc8b,
        ],
    ),
    (
        0x4142a54f0e560419,
        [0x3fd6c283cb0d943a, 0xbfa9c3c0c068e620, 0x3f64404e0efa69b9],
        [
            0x3fd6c283cb0d943a,
            0xbfa9c3c0c068e620,
            0x3f64404e0efa69b9,
            0x402c5d07ecf7b8f2,
        ],
    ),
    (
        0x4142a847fdb22d0f,
        [0x4012de76d96ca0db, 0xbf7c89d8c2326079, 0x3f65d338bdcd4500],
        [
            0x4012de76d96ca0db,
            0xbf7c89d8c2326079,
            0x3f65d338bdcd4500,
            0x40285e347e93b3a6,
        ],
    ),
    (
        0x4142ab40724dd2f2,
        [0x400408bbfb889ee3, 0x3fae9c9fabcdd378, 0x3f6624c6ea8684e9],
        [
            0x400408bbfb889ee3,
            0x3fae9c9fabcdd378,
            0x3f6624c6ea8684e9,
            0x4027c656c9c3f763,
        ],
    ),
    (
        0x4142ae3961a9fbe8,
        [0x3fe35830513abb29, 0x3fb5f0b5d1a1520f, 0x3f64df1dcc2b855d],
        [
            0x3fe35830513abb29,
            0x3fb5f0b5d1a1520f,
            0x3f64df1dcc2b855d,
            0x402ad34e5076da2e,
        ],
    ),
    (
        0x4142b132510624de,
        [0x4014b0427c3e2b0a, 0x3fb79868b0d9d091, 0x3f64454a2a633a13],
        [
            0x4014b0427c3e2b0a,
            0x3fb79868b0d9d091,
            0x3f64454a2a633a13,
            0x402c661999fd4030,
        ],
    ),
    (
        0x4142b42b40624dd3,
        [0x400ae7149086740e, 0x3fb5d08f1529e369, 0x3f657c136c8d33df],
        [
            0x400ae7149086740e,
            0x3fb5d08f1529e369,
            0x3f657c136c8d33df,
            0x4029330b4333f9c6,
        ],
    ),
    (
        0x4142b7242fbe76c9,
        [0x3ff5a554bb55982f, 0x3fa97386926ac0c2, 0x3f662177deeaf99a],
        [
            0x3ff5a554bb55982f,
            0x3fa97386926ac0c2,
            0x3f662177deeaf99a,
            0x4027a73d5018877a,
        ],
    ),
    (
        0x4142ba1d1f1a9fbf,
        [0x401691e915c7fcd8, 0xbf5f4419a37dd233, 0x3f65241912d92a9f],
        [
            0x401691e915c7fcd8,
            0xbf5f4419a37dd233,
            0x3f65241912d92a9f,
            0x4029dddbdefc2e43,
        ],
    ),
    (
        0x4142bd160e76c8b5,
        [0x400ed8542b2886e7, 0xbfa2dbbaeff2846e, 0x3f6402e6d0dfdbed],
        [
            0x400ed8542b2886e7,
            0xbfa2dbbaeff2846e,
            0x3f6402e6d0dfdbed,
            0x402cfc49b05f633e,
        ],
    ),
    (
        0x4142c00e83126e98,
        [0x3ffe7327f0cd27cf, 0xbfb3088a52689a60, 0x3f64cccdb4c69c14],
        [
            0x3ffe7327f0cd27cf,
            0xbfb3088a52689a60,
            0x3f64cccdb4c69c14,
            0x402b00a0c6da1987,
        ],
    ),
    (
        0x4142c307726e978e,
        [0x3f795d0c558d5fd4, 0xbfb69ae56efcd871, 0x3f6624bb2d8d8870],
        [
            0x3f795d0c558d5fd4,
            0xbfb69ae56efcd871,
            0x3f6624bb2d8d8870,
            0x4027eb3bed5cef31,
        ],
    ),
    (
        0x4142c60061cac084,
        [0x401122d5172efa12, 0xbfb41621f3bdca76, 0x3f65b01643bb2525],
        [
            0x401122d5172efa12,
            0xbfb41621f3bdca76,
            0x3f65b01643bb2525,
            0x4028ea309975d97f,
        ],
    ),
    (
        0x4142c8f95126e979,
        [0x40031c39534de621, 0xbfa9b5a2f9d1ddb3, 0x3f63fadb24a01f01],
        [
            0x40031c39534de621,
            0xbfa9b5a2f9d1ddb3,
            0x3f63fadb24a01f01,
            0x402d357339ca1f1c,
        ],
    ),
    (
        0x4142cbf24083126e,
        [0x3fe4ed2de4f489e6, 0xbf9a6536bfbd74b1, 0x3f63d5a39dfae872],
        [
            0x3fe4ed2de4f489e6,
            0xbf9a6536bfbd74b1,
            0x3f63d5a39dfae872,
            0x402d9c4300323807,
        ],
    ),
    (
        0x4142ceeb2fdf3b64,
        [0x4014640a2601f79a, 0x3f865850cdc796fd, 0x3f6584a04c1f6946],
        [
            0x4014640a2601f79a,
            0x3f865850cdc796fd,
            0x3f6584a04c1f6946,
            0x4029319563606f91,
        ],
    ),
    (
        0x4142d1e41f3b645a,
        [0x4008e42d1677631e, 0x3facea46b082f8aa, 0x3f6636fcaa94b5b9],
        [
            0x4008e42d1677631e,
            0x3facea46b082f8aa,
            0x3f6636fcaa94b5b9,
            0x4027b4149fd7e4ab,
        ],
    ),
    (
        0x4142d4dc93d70a3d,
        [0x3fef283b0e4524e7, 0x3fb649babc992e42, 0x3f6514ed161a0fe1],
        [
            0x3fef283b0e4524e7,
            0x3fb649babc992e42,
            0x3f6514ed161a0fe1,
            0x402a71c63c216869,
        ],
    ),
    (
        0x4142d7d583333333,
        [0x4015fb52cab9f5a0, 0x3fb65e9ff9e2172f, 0x3f63d2adfa97501f],
        [
            0x4015fb52cab9f5a0,
            0x3fb65e9ff9e2172f,
            0x3f63d2adfa97501f,
            0x402dc7b36d49654b,
        ],
    ),
    (
        0x4142dace728f5c29,
        [0x400dc11a3edf7b67, 0x3fb3922fa5cfc313, 0x3f64b98233e777cd],
        [
            0x400dc11a3edf7b67,
            0x3fb3922fa5cfc313,
            0x3f64b98233e777cd,
            0x402b1e29d3240e95,
        ],
    ),
    (
        0x4142ddc761eb851e,
        [0x3ffbf733902cd988, 0x3fa3634fdce8124c, 0x3f660f490bcab804],
        [
            0x3ffbf733902cd988,
            0x3fa3634fdce8124c,
            0x3f660f490bcab804,
            0x4027d5b9d85056f1,
        ],
    ),
    (
        0x4142e0c05147ae14,
        [0x401819e06343d81a, 0xbf90517c8d51c024, 0x3f65b9893f8b8e85],
        [
            0x401819e06343d81a,
            0xbf90517c8d51c024,
            0x3f65b9893f8b8e85,
            0x402887b096033cfd,
        ],
    ),
    (
        0x4142e3b940a3d70a,
        [0x4010cb488d184668, 0xbfab3a733244ffb5, 0x3f6469f22a98801d],
        [
            0x4010cb488d184668,
            0xbfab3a733244ffb5,
            0x3f6469f22a98801d,
            0x402bdb753409f00f,
        ],
    ),
    (
        0x4142e6b230000000,
        [0x4003f63ec26b4f3d, 0xbfb2eb3ca94653bc, 0x3f64b99307173f62],
        [
            0x4003f63ec26b4f3d,
            0xbfb2eb3ca94653bc,
            0x3f64b99307173f62,
            0x402b19f296f1abc6,
        ],
    ),
    (
        0x4142e9aaa49ba5e3,
        [0x3fda008ac18e08e5, 0xbfb7838799d1529b, 0x3f65d88d1bba68a4],
        [
            0x3fda008ac18e08e5,
            0xbfb7838799d1529b,
            0x3f65d88d1bba68a4,
            0x40286ca5a96d2252,
        ],
    ),
    (
        0x4142eca393f7ced9,
        [0x4012a8ee87bec5d8, 0xbfb29110f298d194, 0x3f66023de7d44ca1],
        [
            0x4012a8ee87bec5d8,
            0xbfb29110f298d194,
            0x3f66023de7d44ca1,
            0x402801fb490e6f82,
        ],
    ),
    (
        0x4142ef9c8353f7cf,
        [0x4005a5bf482f79df, 0xbfa05596f4424dea, 0x3f64a793526024f0],
        [
            0x4005a5bf482f79df,
            0xbfa05596f4424dea,
            0x3f64a793526024f0,
            0x402b292c14adb0c4,
        ],
    ),
    (
        0x4142f29572b020c4,
        [0x3feeb8a892777d78, 0xbf4d07775dcf23fe, 0x3f63ed029618b9d1],
        [
            0x3feeb8a892777d78,
            0xbf4d07775dcf23fe,
            0x3f63ed029618b9d1,
            0x402d41adb860b504,
        ],
    ),
    (
        0x4142f58e620c49ba,
        [0x4015e57c69d5f8dd, 0x3f9eea484b2361e6, 0x3f654929dabbf25a],
        [
            0x4015e57c69d5f8dd,
            0x3f9eea484b2361e6,
            0x3f654929dabbf25a,
            0x4029c1ace5afb63c,
        ],
    ),
    (
        0x4142f887516872b0,
        [0x400c283282690738, 0x3fb1610f74686da5, 0x3f6642a819f47518],
        [
            0x400c283282690738,
            0x3fb1610f74686da5,
            0x3f6642a819f47518,
            0x4027aae986cfab57,
        ],
    ),
];

#[cfg(target_arch = "aarch64")]
#[test]
fn moon_reads_match_the_9_4_1_golden_table() {
    let _guard = lock();
    let p = AnalyticalProvider::new();
    for (jd_bits, ecl, app) in GOLDEN_9_4_1 {
        let jd = f64::from_bits(jd_bits);
        let e = ecliptic_position(&p, Body::Moon, jd).unwrap();
        assert_eq!(
            [
                e.longitude.to_bits(),
                e.latitude.to_bits(),
                e.distance.to_bits()
            ],
            ecl,
            "ecliptic_position(Moon, {jd}) moved off 9.4.1"
        );
        let a = apparent_positions(&p, &[Body::Moon], jd)
            .remove(0)
            .1
            .unwrap();
        assert_eq!(
            [
                a.ecliptic.longitude.to_bits(),
                a.ecliptic.latitude.to_bits(),
                a.ecliptic.distance.to_bits(),
                a.longitude_speed.to_bits(),
            ],
            app,
            "apparent_positions(Moon, {jd}) moved off 9.4.1"
        );
    }
}

/// The ELP counters are process-global and libtest runs tests on threads of
/// one process, so every counting test holds this lock.
static COUNTER: Mutex<()> = Mutex::new(());

fn lock() -> MutexGuard<'static, ()> {
    COUNTER
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Full and distance-only ELP/MPP02 evaluations made by `read`.
fn count(read: impl FnOnce()) -> (u64, u64) {
    ELP_GEOCENTRIC_CALLS.store(0, Ordering::Relaxed);
    ELP_DISTANCE_CALLS.store(0, Ordering::Relaxed);
    read();
    (
        ELP_GEOCENTRIC_CALLS.load(Ordering::Relaxed),
        ELP_DISTANCE_CALLS.load(Ordering::Relaxed),
    )
}

/// A longitude-only Moon read costs one full series evaluation plus one
/// distance-only one, where 9.4.1 averaged 2.756 full evaluations and made
/// three in most reads.
#[test]
fn longitude_only_moon_read_evaluates_the_series_about_once() {
    let _guard = lock();
    let p = AnalyticalProvider::new();
    let n = 4_000_u32;
    let (mut full_total, mut most_full) = (0_u64, 0_u64);
    for jd in instants(n) {
        let (full, distance) = count(|| {
            ecliptic_position(&p, Body::Moon, jd).unwrap();
        });
        assert_eq!(distance, 1, "jd {jd}: {distance} distance-only evaluations");
        full_total += full;
        most_full = most_full.max(full);
    }
    let mean = full_total as f64 / f64::from(n);
    println!("ecliptic_position(Moon): {mean} full evaluations per read, at most {most_full}");
    assert!(most_full <= 2, "a read made {most_full} full evaluations");
    assert!(mean <= 1.01, "{mean} full evaluations per read");
}

/// `apparent_positions(Moon)` makes three light-time-corrected reads, each
/// now one full evaluation and one distance-only one, where 9.4.1 made six
/// full evaluations.
#[test]
fn apparent_positions_moon_evaluates_the_series_three_times() {
    let _guard = lock();
    let p = AnalyticalProvider::new();
    let n = 1_000_u32;
    let (mut full_total, mut most_full) = (0_u64, 0_u64);
    for jd in instants(n) {
        let (full, distance) = count(|| {
            apparent_positions(&p, &[Body::Moon], jd)
                .remove(0)
                .1
                .unwrap();
        });
        assert_eq!(distance, 3, "jd {jd}: {distance} distance-only evaluations");
        full_total += full;
        most_full = most_full.max(full);
    }
    let mean = full_total as f64 / f64::from(n);
    println!("apparent_positions(Moon): {mean} full evaluations per read, at most {most_full}");
    assert!(most_full <= 4, "a read made {most_full} full evaluations");
    assert!(mean <= 3.03, "{mean} full evaluations per read");
}

/// `moon_apparent_position` costs one position-and-velocity evaluation and
/// one distance-only one.
#[test]
fn moon_apparent_position_evaluates_the_series_about_once() {
    let _guard = lock();
    let p = AnalyticalProvider::new();
    let n = 1_000_u32;
    let (mut full_total, mut most_full) = (0_u64, 0_u64);
    for jd in instants(n) {
        let (full, distance) = count(|| {
            moon_apparent_position(&p, jd).unwrap();
        });
        assert_eq!(distance, 1, "jd {jd}: {distance} distance-only evaluations");
        full_total += full;
        most_full = most_full.max(full);
    }
    let mean = full_total as f64 / f64::from(n);
    println!("moon_apparent_position: {mean} full evaluations per read, at most {most_full}");
    assert!(most_full <= 2, "a read made {most_full} full evaluations");
    assert!(mean <= 1.01, "{mean} full evaluations per read");
}

/// The analytic speed against two difference quotients of the same pipeline.
///
/// **Against a Richardson-extrapolated TT difference** of `ecliptic_position_tt`
/// (the function `moon_apparent_position` differentiates): `(4·D(h/2) − D(h))
/// / 3` with `D` the central difference and `h` = 0.08 day. A plain fine
/// difference cannot serve as the reference. The light-time iteration's
/// retarded instant `jd − τ` sits on the double grid, about 4.7e-10 day apart
/// near a modern `jd`, so the read longitude steps by up to about 6e-9° and a
/// central difference over `±h` carries noise near 6e-9 / 2h: 1.5e-4 °/day at
/// h = 2.5e-5 day, and still 1e-6 °/day at the step (about 0.005 day) where
/// it balances truncation. Extrapolation removes the `h²` truncation term and
/// allows a step wide enough that the grid noise is near 1e-7 °/day. Measured
/// over 1,500 instants, 1900 to 2100: [`RICHARDSON_TOL`] holds it with
/// margin, and is below what either part of the speed contributes on its own
/// (a dropped light-time factor reaches a few times 1e-6 °/day, a dropped
/// frame rate about 4e-5), so it would catch either one missing.
///
/// **Against `apparent_position`'s ±0.5-day UT1 difference**, the speed this
/// read sits beside: they differ by that difference's own truncation error,
/// worst 8.5e-3 °/day measured, with no bias.
#[test]
fn moon_apparent_position_speed_matches_difference_quotients() {
    let _guard = lock();
    let p = AnalyticalProvider::new();
    let longitude_tt = |jd_tt: f64| {
        ecliptic_position_tt(&p, Body::Moon, jd_tt)
            .unwrap()
            .longitude
    };
    let central = |jd_tt: f64, h: f64| {
        let mut d = longitude_tt(jd_tt + h) - longitude_tt(jd_tt - h);
        if d > std::f64::consts::PI {
            d -= 2.0 * std::f64::consts::PI;
        } else if d < -std::f64::consts::PI {
            d += 2.0 * std::f64::consts::PI;
        }
        d.to_degrees() / (2.0 * h)
    };
    let h = 0.08;
    let (mut worst_fine, mut worst_day, mut sum_day) = (0.0_f64, 0.0_f64, 0.0_f64);
    let n = 1_500_u32;
    for jd in instants(n) {
        let analytic = moon_apparent_position(&p, jd).unwrap().longitude_speed;

        let jd_tt = delta_t::ut1_to_tt(jd);
        let extrapolated = (4.0 * central(jd_tt, h / 2.0) - central(jd_tt, h)) / 3.0;
        worst_fine = worst_fine.max((analytic - extrapolated).abs());

        let day = apparent_position(&p, Body::Moon, jd)
            .unwrap()
            .longitude_speed;
        worst_day = worst_day.max((analytic - day).abs());
        sum_day += analytic - day;
    }
    let bias = sum_day / f64::from(n);
    println!(
        "analytic speed vs Richardson (h = {h} day): worst {worst_fine:e} °/day; \
         vs ±0.5-day: worst {worst_day:e} °/day, mean {bias:e} °/day"
    );
    assert!(
        worst_fine < RICHARDSON_TOL,
        "worst {worst_fine:e} °/day against the extrapolated difference"
    );
    assert!(
        worst_day < DAY_TOL,
        "worst {worst_day:e} °/day against ±0.5 day"
    );
    assert!(
        bias.abs() < DAY_TOL / 100.0,
        "mean difference {bias:e} °/day is a bias"
    );
}

/// Largest disagreement, °/day, between the analytic speed and the
/// Richardson-extrapolated difference.
const RICHARDSON_TOL: f64 = 5e-7;

/// Largest disagreement, °/day, between the analytic speed and
/// `apparent_position`'s ±0.5-day central difference: that difference's
/// truncation error, worst 8.5e-3 measured.
const DAY_TOL: f64 = 1e-2;
