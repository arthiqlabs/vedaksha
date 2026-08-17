// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.

//! Bit-exact digest of the analytical provider's per-row output — **pinned**.
//!
//! This exists because `oracle_comparison.rs`'s digest is a **blind instrument**
//! for anything in `analytical/`. That test drives a `SpkReader` over the DE440s
//! binary kernel and never calls `analytical/elp_mpp02.rs`; an FMA change that
//! rewrote 90.67% of ELP's output bits left its digest byte-identical. Any
//! change to the lunar/planetary theory — or to a dependency those modules use,
//! such as `wide` via `analytical/simd_trig.rs` — has to be gated here instead.
//!
//! It **asserts**. The digest of all 21,915 rows is pinned as
//! [`EXPECTED_DIGEST`] and compared, so a bit that moves fails this test rather
//! than merely printing a number somebody has to notice. Accuracy is
//! `analytical_oracle.rs`'s job; this is the bit-level fingerprint, and its
//! whole value is that drift has to be re-pinned deliberately, in a commit,
//! with a reason.
//!
//! The rows are still printed to **stdout**, because that is what makes the
//! pinned value independently reproducible from a shell (see below) and what
//! lets two commits be diffed row-by-row. libtest captures and discards stdout
//! for a passing test, so a routine `--include-ignored` run stays quiet; you
//! only see the rows when you ask for them with `--nocapture`.
//!
//! # Digest method — this is the definition of [`EXPECTED_DIGEST`]
//!
//! Per-row lines → squeeze runs of spaces → sort bytewise → SHA-256 over the
//! sorted lines, each terminated by `\n`. [`canonical_digest`] implements
//! exactly that, so the value the test asserts is byte-for-byte the value this
//! shell pipeline prints:
//!
//! ```text
//! cargo test -p vedaksha-ephem-core --release --test analytical_bit_digest \
//!     -- --ignored --nocapture \
//!   | grep '^ROW ' | tr -s ' ' | LC_ALL=C sort | shasum -a 256
//! ```
//!
//! `tr -s ' '` is `squeeze_spaces`, `LC_ALL=C sort` is a bytewise sort (which
//! is what Rust's `Ord for str` already is), and `shasum -a 256` is
//! [`sha256::hex`]. SHA-256 is implemented in this file rather than taken as a
//! dependency: the workspace has no hash crate, and adding one to pin a digest
//! would put a new transitive tree through `deny.toml` for 60 lines of FIPS
//! 180-4 that cannot drift.
//!
//! Restrict to the lunar theory — the only consumer of `simd_trig`/`wide` — by
//! inserting `| grep ' Moon '` before the `tr`.
//!
//! # Why this test is `#[ignore]`d — measured, not assumed
//!
//! Deliberate, and the measurement is the reason — the choice is between
//! per-push and Full Validation, and the profile decides it. Timed on this
//! machine (aarch64), one run of this test costs
//!
//! | profile | wall | user |
//! |---|---|---|
//! | `--release` | **30.4 s** | 30.1 s |
//! | `debug` | **1 624.2 s** (27 min) | 1 420.6 s |
//!
//! A **53x** ratio, consistent with the ~46x that `vedaksha-astro`'s
//! `require_release_profile` measured for the same `AnalyticalProvider` call
//! path. (The debug figure was taken with the dump before the assertion was
//! added; collecting 21,915 short strings and hashing ~2 MB once is negligible
//! against 21,915 evaluations of the full lunar and planetary theory.)
//!
//! Per-push CI runs `cargo test --workspace --locked` in the DEFAULT profile,
//! which is `debug`. Running this test per-push would therefore add **27
//! minutes to every push** — to guard against a dependency bump, which arrives
//! in a reviewed commit, not on a random push. So it stays `#[ignore]`d, and
//! Full Validation's `cargo test --workspace --release --locked --
//! --include-ignored` runs it on the weekly cron and at release, in the profile
//! it is sized for, for 30 s. That is the right tier, and it is reachable —
//! which was the requirement. Reachability is not incidental: `--include-ignored`
//! is what makes `#[ignore]` a schedule rather than a grave, which is exactly
//! why `riseset`'s derivation sweeps needed a *feature* gate instead.
//!
//! `require_release_profile` guards the trap that follows from that choice: the
//! obvious local way to reach an `#[ignore]`d test is `cargo test --
//! --include-ignored`, which runs `debug`. It PANICS rather than skipping, for
//! the reason given in `vedaksha-astro`'s copy — a test that quietly skips
//! reports green having verified nothing.

mod common;

use vedaksha_ephem_core::analytical::AnalyticalProvider;
use vedaksha_ephem_core::bodies::Body;
use vedaksha_ephem_core::coordinates;
use vedaksha_ephem_core::jpl::EphemerisProvider;

/// Rows the dump must emit: the fixture's 24,350 rows less Pluto's 2,435.
///
/// 9 bodies × 2,435 dates. Pluto is absent because `AnalyticalProvider` has no
/// theory for it and returns `BodyNotAvailable`, so `body_from_name` drops it
/// before the provider is ever called. Pinned as its own assertion because a
/// digest over a SHORTER list is not a weaker signal, it is a different
/// measurement — and one that would otherwise be silently accepted.
const EXPECTED_ROWS: u32 = 21_915;

/// Digest of all 21,915 rows, pinned at `4999d52`.
///
/// Re-derived here from a clean run rather than carried over; identical to the
/// value recorded beside the `wide` declaration in `Cargo.toml`, which two
/// earlier sessions reproduced independently.
///
/// **If this assertion fails, the analytical path's output bits moved.** That
/// is not automatically a defect — see [`PRE_WIDE_DIGEST`] for a case where it
/// was benign — but it is never nothing. Establish what moved and why, decide,
/// and re-pin in a commit that says so. Do not re-pin to make a red test green.
const EXPECTED_DIGEST: &str = "f943337e6dbfe1d7881a001749009e7aa322cbbff2c4aa4e89c4e1db4c266b80";

/// The digest before the `wide` upgrade, at `ee85cdf^` (= `391367d`).
///
/// Recorded so the one drift this pin has already survived is legible rather
/// than folklore. `ee85cdf` ("chore(deps)!: raise MSRV to 1.85 -> 1.89 and take
/// wide 1.6.1") took `wide` 0.7.33 -> 1.6.1 (and `safe_arch` 0.7.4 -> 1.2.0),
/// and that bump is NOT bit-identical: it moves **6 of 21,915 rows**, every one
/// of them the Moon — i.e. confined to ELP/MPP02, the only consumer of `wide`.
/// All eight other bodies are byte-exact. Measured magnitudes, re-derived from
/// the raw bit patterns of the two dumps:
///
/// | quantity | max delta |
/// |---|---|
/// | ecliptic longitude | **1 ULP** (1 row) |
/// | ecliptic latitude | 4 ULP (3 rows) |
/// | distance | 0 ULP |
/// | longitude_speed | **28 ULP** (2 rows) — a differenced quantity, so it amplifies |
///
/// One ULP of longitude is 1.11e-16 rad, about 2.3e-5 microarcseconds — some
/// ten orders of magnitude below ELP/MPP02's own ~0.169" disagreement with
/// DE441, so no accuracy claim in the repo moves. Attribution is exact, not
/// inferred: the only commit in `ee85cdf^..HEAD` touching
/// `crates/vedaksha-ephem-core/src` is `57e74cc`, and every changed line in it
/// is a `//!` comment.
const PRE_WIDE_DIGEST: &str = "e35c5e3ab95dcb35a816d77313a35fa8e773bd7637568bd450e98e3eaef7bb81";

#[derive(serde::Deserialize)]
struct OracleFixture {
    rows: Vec<OracleDataPoint>,
}

#[derive(serde::Deserialize)]
struct OracleDataPoint {
    date: String,
    jd: f64,
    body: String,
}

/// Same mapping as `analytical_oracle.rs`: Pluto has no analytical theory, so
/// the fixture's 24,350 rows reduce to 9 bodies' worth.
fn body_from_name(name: &str) -> Option<Body> {
    match name {
        "Sun" => Some(Body::Sun),
        "Moon" => Some(Body::Moon),
        "Mercury" => Some(Body::Mercury),
        "Venus" => Some(Body::Venus),
        "Mars" => Some(Body::Mars),
        "Jupiter" => Some(Body::Jupiter),
        "Saturn" => Some(Body::Saturn),
        "Uranus" => Some(Body::Uranus),
        "Neptune" => Some(Body::Neptune),
        _ => None,
    }
}

/// Refuse to run in a `debug` build. Same rule, and same reasoning, as
/// `vedaksha-astro`'s `require_release_profile`: this is `#[ignore]`d, the
/// obvious way to reach an `#[ignore]`d test is `cargo test --
/// --include-ignored`, and that runs the DEFAULT profile. Measured above at
/// 30.4 s in `--release` against 1 624.2 s — 27 minutes — in `debug`.
///
/// It PANICS rather than skipping. A test that quietly skips reports green
/// having verified nothing, which is strictly worse than not running it: this
/// test's entire product is a digest, and a skip that looks like a pass
/// destroys the evidence while claiming to supply it.
fn require_release_profile() {
    assert!(
        !cfg!(debug_assertions),
        "analytical_bit_digest is a `--release`-only test and will not be run in \
         a debug build.\n\
         \n\
         It evaluates the full analytical theory for 21,915 rows: 30.4 s in \
         --release, 1624 s (27 min) in debug — a factor of 53.\n\
         \n\
         Run it as:\n\
         \n    cargo test -p vedaksha-ephem-core --release --test \
         analytical_bit_digest -- --ignored\n"
    );
}

/// `tr -s ' '` — collapse each run of spaces to a single space.
///
/// A no-op for the rows this test emits today, since every field is formatted
/// with a fixed width and separated by exactly one space. It is applied anyway
/// because it is part of the published pipeline, and a future field that could
/// render empty must canonicalise the same way on both sides of the comparison
/// rather than silently diverge from the shell command.
fn squeeze_spaces(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut prev_was_space = false;
    for ch in line.chars() {
        if ch == ' ' && prev_was_space {
            continue;
        }
        prev_was_space = ch == ' ';
        out.push(ch);
    }
    out
}

/// `tr -s ' ' | LC_ALL=C sort | shasum -a 256` over the emitted rows.
///
/// `LC_ALL=C sort` is a bytewise ascending sort of whole lines, which is
/// precisely `Ord for str`; `sort` terminates every line it writes with `\n`,
/// including the last, so the hashed blob does too.
fn canonical_digest(rows: &[String]) -> String {
    let mut lines: Vec<String> = rows.iter().map(|r| squeeze_spaces(r)).collect();
    lines.sort_unstable();

    let mut blob = String::with_capacity(lines.iter().map(|l| l.len() + 1).sum());
    for line in &lines {
        blob.push_str(line);
        blob.push('\n');
    }
    sha256::hex(blob.as_bytes())
}

/// SHA-256, FIPS 180-4.
///
/// Vendored rather than taken as a dependency: the workspace has no hash crate,
/// and pulling one in to pin a digest would push a new transitive tree through
/// `deny.toml` and the release audit in exchange for an algorithm that is
/// frozen by standard and verified on every run of this test — the pinned
/// values were produced by the system `shasum -a 256`, so agreement with them
/// is itself the implementation's conformance check.
mod sha256 {
    /// FIPS 180-4 §4.2.2 round constants: the first 32 bits of the fractional
    /// parts of the cube roots of the first 64 primes. Laid out 8 per line so
    /// the table can be read against the standard — the same reason
    /// `nutation.rs` skips rustfmt over its coefficient table.
    #[rustfmt::skip]
    const K: [u32; 64] = [
        0x428a_2f98, 0x7137_4491, 0xb5c0_fbcf, 0xe9b5_dba5,
        0x3956_c25b, 0x59f1_11f1, 0x923f_82a4, 0xab1c_5ed5,
        0xd807_aa98, 0x1283_5b01, 0x2431_85be, 0x550c_7dc3,
        0x72be_5d74, 0x80de_b1fe, 0x9bdc_06a7, 0xc19b_f174,
        0xe49b_69c1, 0xefbe_4786, 0x0fc1_9dc6, 0x240c_a1cc,
        0x2de9_2c6f, 0x4a74_84aa, 0x5cb0_a9dc, 0x76f9_88da,
        0x983e_5152, 0xa831_c66d, 0xb003_27c8, 0xbf59_7fc7,
        0xc6e0_0bf3, 0xd5a7_9147, 0x06ca_6351, 0x1429_2967,
        0x27b7_0a85, 0x2e1b_2138, 0x4d2c_6dfc, 0x5338_0d13,
        0x650a_7354, 0x766a_0abb, 0x81c2_c92e, 0x9272_2c85,
        0xa2bf_e8a1, 0xa81a_664b, 0xc24b_8b70, 0xc76c_51a3,
        0xd192_e819, 0xd699_0624, 0xf40e_3585, 0x106a_a070,
        0x19a4_c116, 0x1e37_6c08, 0x2748_774c, 0x34b0_bcb5,
        0x391c_0cb3, 0x4ed8_aa4a, 0x5b9c_ca4f, 0x682e_6ff3,
        0x748f_82ee, 0x78a5_636f, 0x84c8_7814, 0x8cc7_0208,
        0x90be_fffa, 0xa450_6ceb, 0xbef9_a3f7, 0xc671_78f2,
    ];

    /// FIPS 180-4 §5.3.3 initial hash value: the first 32 bits of the
    /// fractional parts of the square roots of the first 8 primes.
    #[rustfmt::skip]
    const H0: [u32; 8] = [
        0x6a09_e667, 0xbb67_ae85, 0x3c6e_f372, 0xa54f_f53a,
        0x510e_527f, 0x9b05_688c, 0x1f83_d9ab, 0x5be0_cd19,
    ];

    /// Lowercase hex SHA-256 of `data`.
    pub fn hex(data: &[u8]) -> String {
        let mut h = H0;

        // Pad: 0x80, then zeros to 56 mod 64, then the length in BITS, big-endian.
        let mut msg = Vec::with_capacity(data.len() + 72);
        msg.extend_from_slice(data);
        msg.push(0x80);
        while msg.len() % 64 != 56 {
            msg.push(0);
        }
        let bit_len = (data.len() as u64) * 8;
        msg.extend_from_slice(&bit_len.to_be_bytes());

        for chunk in msg.chunks_exact(64) {
            let mut w = [0_u32; 64];
            for (word, bytes) in w.iter_mut().zip(chunk.chunks_exact(4)) {
                *word = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
            }
            for i in 16..64 {
                let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
                let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
                w[i] = w[i - 16]
                    .wrapping_add(s0)
                    .wrapping_add(w[i - 7])
                    .wrapping_add(s1);
            }

            let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh] = h;
            for (k, word) in K.iter().zip(w.iter()) {
                let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
                let ch = (e & f) ^ ((!e) & g);
                let t1 = hh
                    .wrapping_add(s1)
                    .wrapping_add(ch)
                    .wrapping_add(*k)
                    .wrapping_add(*word);
                let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
                let maj = (a & b) ^ (a & c) ^ (b & c);
                let t2 = s0.wrapping_add(maj);

                hh = g;
                g = f;
                f = e;
                e = d.wrapping_add(t1);
                d = c;
                c = b;
                b = a;
                a = t1.wrapping_add(t2);
            }

            for (slot, delta) in h.iter_mut().zip([a, b, c, d, e, f, g, hh]) {
                *slot = slot.wrapping_add(delta);
            }
        }

        h.iter().map(|word| format!("{word:08x}")).collect()
    }

    #[cfg(test)]
    mod tests {
        /// The two FIPS 180-4 Appendix B vectors, plus the empty string.
        ///
        /// This is what makes the vendored implementation trustworthy on its
        /// own terms rather than only by agreeing with the pinned dump digest.
        #[test]
        fn matches_the_published_vectors() {
            assert_eq!(
                super::hex(b""),
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
            );
            assert_eq!(
                super::hex(b"abc"),
                "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
            );
            assert_eq!(
                super::hex(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"),
                "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
            );
        }
    }
}

/// Dump every analytical row, then assert the digest of the dump.
///
/// The tiers are all measured before any is asserted: the row count, the
/// out-of-range count and the digest are computed first and asserted last, so a
/// failure reports every number rather than stopping at the first.
#[test]
#[ignore = "release-only bit fingerprint of the analytical path; 30.4 s in --release, >10 min in debug — see the module comment"]
fn analytical_bit_digest() {
    require_release_profile();

    let oracle_path = common::horizons_oracle_path();
    assert!(
        oracle_path.exists(),
        "fixture missing at {} — this dump has nothing to fingerprint",
        oracle_path.display()
    );

    let fixture: OracleFixture =
        serde_json::from_str(&std::fs::read_to_string(&oracle_path).unwrap()).unwrap();

    let provider = AnalyticalProvider;
    let (jd_min, jd_max) = provider.time_range();

    let mut rows: Vec<String> = Vec::with_capacity(EXPECTED_ROWS as usize);
    let mut skipped_body = 0u32;
    let mut skipped_range = 0u32;

    for dp in &fixture.rows {
        let Some(body) = body_from_name(&dp.body) else {
            skipped_body += 1;
            continue;
        };
        if dp.jd < jd_min || dp.jd > jd_max {
            skipped_range += 1;
            continue;
        }

        // An error is emitted as a row rather than skipped: a body that starts
        // failing must change the digest, not quietly shrink it.
        let row = match coordinates::apparent_position(&provider, body, dp.jd) {
            Ok(p) => format!(
                "ROW {} {} {:.6} {:016x} {:016x} {:016x} {:016x}",
                dp.date,
                dp.body,
                dp.jd,
                p.ecliptic.longitude.to_bits(),
                p.ecliptic.latitude.to_bits(),
                p.ecliptic.distance.to_bits(),
                p.longitude_speed.to_bits(),
            ),
            Err(e) => format!("ROW {} {} {:.6} ERR {e:?}", dp.date, dp.body, dp.jd),
        };
        println!("{row}");
        rows.push(row);
    }

    let emitted = rows.len() as u32;
    let digest = canonical_digest(&rows);

    eprintln!(
        "analytical_bit_digest: {emitted} rows emitted \
         ({skipped_body} skipped as body-without-theory, \
         {skipped_range} skipped as out-of-range, \
         {} in fixture)\n\
         analytical_bit_digest: sha256 {digest}",
        fixture.rows.len()
    );

    // Every row the fixture offers for a body with a theory must reach the
    // provider. `AnalyticalProvider`'s range covers the whole fixture today, so
    // a non-zero count here means the range shrank and the digest silently
    // became a fingerprint of a smaller set.
    assert_eq!(
        skipped_range, 0,
        "{skipped_range} rows fell outside the provider's time range \
         ({jd_min} .. {jd_max}); the digest below no longer covers the fixture"
    );
    assert_eq!(
        emitted, EXPECTED_ROWS,
        "row count moved: the digest is only comparable across commits that \
         emit the same rows"
    );
    assert_eq!(
        digest, EXPECTED_DIGEST,
        "\n\nThe analytical path's output bits MOVED.\n\n\
         expected {EXPECTED_DIGEST}\n\
         actual   {digest}\n\
         (for reference, before the `wide` 0.7.33 -> 1.6.1 upgrade: \
         {PRE_WIDE_DIGEST})\n\n\
         This is a real change to ELP/MPP02, VSOP87A, or something they call. \
         Find out what moved and by how much — re-run this test on both commits \
         with --nocapture and diff the ROW lines — decide whether it is \
         acceptable, and re-pin EXPECTED_DIGEST in a commit that records the \
         reason. Do not re-pin to make a red test green.\n"
    );
}
