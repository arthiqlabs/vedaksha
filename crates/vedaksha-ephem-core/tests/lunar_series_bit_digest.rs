// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1

//! Bit-exact digest of the **raw ELP/MPP02 lunar series** output — **pinned**.
//!
//! `analytical_bit_digest.rs` fingerprints the whole analytical provider, but
//! only 2,435 of its 21,915 rows are the Moon, all of them at fixture dates
//! chosen for a Horizons comparison, and every one of them passes through
//! `apparent_position` — precession, nutation, aberration, light time — before
//! it is printed. That is the right instrument for "did any published number
//! move". It is a *coarse* instrument for "did `elp_mpp02.rs` itself move":
//! 2,435 samples of a function whose inputs span 5,000 years, downstream of
//! several rotations that can absorb the low bits.
//!
//! This test is the fine one. It calls the three public entry points of
//! `analytical::elp_mpp02` directly, prints the raw IEEE-754 bit patterns of
//! all six returned components with nothing downstream of them, and pins a
//! digest over the lot:
//!
//! | block | JDs | step | why |
//! |---|---|---|---|
//! | full range | 200,001 | 9.13 d | `AnalyticalProvider::time_range()` end to end |
//! | contemporary | 10,000 | 0.01 d | dense sampling where charts actually land |
//!
//! × 3 surfaces (`elp_geocentric`, `elp_geocentric_with_fit(De405)`,
//! `elp_geocentric_of_date`) = **630,003 lunar-series evaluations**, 3,780,018
//! pinned `f64` bit patterns.
//!
//! A second, companion test ([`lunar_series_position_only_bit_digest`]) runs
//! the same JD sweep over the two position-only surfaces added in Wave 2
//! (`elp_geocentric_position`, `elp_geocentric_position_of_date`), pinning
//! its own digest — kept separate from the digest above rather than folded
//! in, so neither pinned value has to move when the other surface set
//! changes.
//!
//! # What it is for
//!
//! It was added with the sparse-multiplier rewrite of `term_poc`
//! (`docs/audit/2026-08-29-perf-investigation.md` #2 + #4), whose whole design
//! claim is that skipping exact-zero multipliers is *provably* bit-identical
//! rather than merely close. A claim of that shape needs an instrument dense
//! enough to falsify it. The digest below is the value measured on the commit
//! **before** that rewrite; the rewrite leaves it unchanged.
//!
//! Both fits are covered because the `Args` memoization in the same change is
//! keyed on [`Fit`], and `elp_geocentric_of_date` because it is a separate
//! public surface through the same series.
//!
//! # Digest method — this is the definition of [`EXPECTED_DIGEST`]
//!
//! SHA-256 over the `LROW` lines **in emission order** (not sorted — emission
//! order is a deterministic `for` loop here, unlike `analytical_bit_digest`'s
//! fixture-driven rows), each terminated by `\n`. Setting
//! `VEDAKSHA_DUMP_LUNAR_ROWS=1` prints the rows to stdout so the value is
//! reproducible from a shell and two commits can be diffed row by row:
//!
//! ```text
//! VEDAKSHA_DUMP_LUNAR_ROWS=1 cargo test -p vedaksha-ephem-core --release \
//!     --test lunar_series_bit_digest -- --ignored --nocapture \
//!   | grep '^LROW ' | shasum -a 256
//! ```
//!
//! Printing is behind the variable rather than unconditional (the choice
//! `analytical_bit_digest` makes) only because 630,003 rows is ~76 MB, which
//! libtest would otherwise capture into memory on every run. The digest is
//! computed from exactly the same bytes either way — the flag controls
//! printing, never hashing.
//!
//! SHA-256 is vendored here for the same reason it is vendored in
//! `analytical_bit_digest.rs`: the workspace has no hash crate and adding one
//! to pin a digest would put a new transitive tree through `deny.toml`. This
//! copy is a *streaming* implementation (the row blob does not fit comfortably
//! in memory) and is checked against the FIPS 180-4 Appendix B vectors below.
//!
//! # Why this test is `#[ignore]`d
//!
//! Same tier decision, and same reasoning, as `analytical_bit_digest`: it is
//! release-only work that belongs in Full Validation, not on every push.
//! Measured on this machine (aarch64, `--release`): **~170 s**. Per-push CI
//! runs the `debug` profile, where the analytical path is ~53× slower — hours,
//! not minutes — so it carries the same `require_release_profile` panic guard,
//! which PANICS rather than skipping. A test that quietly skips reports green
//! having verified nothing.

use vedaksha_ephem_core::analytical::AnalyticalProvider;
use vedaksha_ephem_core::analytical::elp_mpp02::{
    Fit, elp_geocentric, elp_geocentric_of_date, elp_geocentric_position,
    elp_geocentric_position_of_date, elp_geocentric_with_fit,
};
use vedaksha_ephem_core::jpl::EphemerisProvider;

/// Uniform samples across the provider's entire supported JD range.
const FULL_RANGE_SAMPLES: u32 = 200_001;

/// Dense samples at 0.01-day steps from a contemporary epoch (≈ 2025-01-01).
const CONTEMPORARY_SAMPLES: u32 = 10_000;

/// Start of the dense contemporary block (the epoch `benches/ephemeris.rs` uses).
const CONTEMPORARY_START_JD: f64 = 2_460_676.5;

/// Step of the dense contemporary block, in days.
const CONTEMPORARY_STEP_DAYS: f64 = 0.01;

/// Rows the sweep must emit: (200,001 + 10,000) JDs × 3 surfaces.
///
/// Pinned as its own assertion for the reason `analytical_bit_digest` pins its
/// row count: a digest over a shorter list is not a weaker signal, it is a
/// different measurement, and one that would otherwise be silently accepted.
const EXPECTED_ROWS: u64 = (FULL_RANGE_SAMPLES as u64 + CONTEMPORARY_SAMPLES as u64) * 3;

/// Digest of all 630,003 rows.
///
/// # Measured 2026-08-29, before the sparse-multiplier rewrite
///
/// This value was produced on the commit preceding
/// "perf(ephem-core): skip the zero multipliers ELP spends 70% of its
/// perturbation dot products on" — i.e. against the dense 13-multiply /
/// 12-add `term_poc`, the per-call `build_args`, the per-call
/// `corrected_main_amplitude`, and the triplicated `reduce_arg` sweeps. The
/// rewrite leaves it **unchanged**, which is the point of the test: it is the
/// independent, 630,003-evaluation confirmation of the bit-identity claim, on
/// top of `analytical_bit_digest`'s 21,915 end-to-end rows.
///
/// **If this assertion fails, `elp_mpp02.rs`'s output bits moved.** Establish
/// what moved and by how much — re-run both commits with
/// `VEDAKSHA_DUMP_LUNAR_ROWS=1 … --nocapture` and diff the `LROW` lines —
/// decide whether it is acceptable, and re-pin in a commit that records the
/// reason. Do not re-pin to make a red test green.
///
/// **Pinned per architecture.** This digest fingerprints `sincos_f64x4`, and
/// `wide`'s `f64x4` computes differently on AVX2 than on NEON, so there is no
/// single correct value — see `analytical_bit_digest.rs`'s `EXPECTED_DIGEST`
/// doc for the full explanation and for what that means for the phrase
/// "bit-identical" elsewhere in this repository. Add an architecture by running
/// the test there and adding an arm; never reuse another architecture's value.
#[cfg(target_arch = "aarch64")]
const EXPECTED_DIGEST: &str = "e8b21c8863fa0065e125e16a4b3396371f7de6a386d19f35f233c07ac5e1394b";

/// x86_64 counterpart of [`EXPECTED_DIGEST`], measured 2026-08-31.
#[cfg(target_arch = "x86_64")]
const EXPECTED_DIGEST: &str = "a2af33c87957fd15249d6f5b714749a8623f0ad4593a1fbc1ce0d4b51c0cccfe";

/// Sentinel for an unpinned architecture — the test panics on it. See
/// [`EXPECTED_DIGEST`].
#[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
const EXPECTED_DIGEST: &str = "";

/// Refuse to run in a `debug` build, for the reason
/// `analytical_bit_digest::require_release_profile` gives: this is
/// `#[ignore]`d, the obvious way to reach an `#[ignore]`d test is
/// `cargo test -- --include-ignored`, and that runs the DEFAULT profile.
///
/// It PANICS rather than skipping. This test's entire product is a digest; a
/// skip that looks like a pass destroys the evidence while claiming to supply
/// it.
fn require_release_profile() {
    assert!(
        !cfg!(debug_assertions),
        "lunar_series_bit_digest is a `--release`-only test and will not be run \
         in a debug build.\n\
         \n\
         It evaluates the ELP/MPP02 series 630,003 times: ~170 s in --release, \
         hours in debug.\n\
         \n\
         Run it as:\n\
         \n    cargo test -p vedaksha-ephem-core --release --test \
         lunar_series_bit_digest -- --ignored\n"
    );
}

/// Every JD the sweep visits, in emission order.
fn sweep_jds(jd_min: f64, jd_max: f64) -> Vec<f64> {
    let mut jds = Vec::with_capacity((FULL_RANGE_SAMPLES + CONTEMPORARY_SAMPLES) as usize);

    // Block A — the full supported range, endpoints included.
    let step = (jd_max - jd_min) / f64::from(FULL_RANGE_SAMPLES - 1);
    for i in 0..FULL_RANGE_SAMPLES {
        jds.push(jd_min + f64::from(i) * step);
    }

    // Block B — 100 contemporary days at 0.01-day resolution.
    for i in 0..CONTEMPORARY_SAMPLES {
        jds.push(CONTEMPORARY_START_JD + f64::from(i) * CONTEMPORARY_STEP_DAYS);
    }

    jds
}

#[test]
#[ignore = "release-only bit fingerprint of the raw lunar series; ~170 s in --release, hours in debug — see the module comment"]
fn lunar_series_bit_digest() {
    require_release_profile();

    let dump = std::env::var_os("VEDAKSHA_DUMP_LUNAR_ROWS").is_some();

    let (jd_min, jd_max) = AnalyticalProvider.time_range();
    let jds = sweep_jds(jd_min, jd_max);

    let mut hasher = sha256::Sha256::new();
    let mut rows = 0u64;
    let mut line = String::with_capacity(160);

    // The three public surfaces of `analytical::elp_mpp02`, each over the whole
    // JD list. Kept as three separate passes (rather than three rows per JD)
    // so a divergence confined to one surface is obvious in a row diff.
    for surface in 0..3u8 {
        let tag = match surface {
            0 => "J2000_LLR",
            1 => "J2000_DE405",
            _ => "OFDATE_LLR",
        };
        for &jd in &jds {
            let m = match surface {
                0 => elp_geocentric(jd),
                1 => elp_geocentric_with_fit(jd, Fit::De405),
                _ => elp_geocentric_of_date(jd),
            };
            line.clear();
            use std::fmt::Write as _;
            write!(
                line,
                "LROW {tag} {jd:.6} {:016x} {:016x} {:016x} {:016x} {:016x} {:016x}",
                m.x.to_bits(),
                m.y.to_bits(),
                m.z.to_bits(),
                m.vx.to_bits(),
                m.vy.to_bits(),
                m.vz.to_bits(),
            )
            .expect("writing to a String cannot fail");
            if dump {
                println!("{line}");
            }
            hasher.update(line.as_bytes());
            hasher.update(b"\n");
            rows += 1;
        }
    }

    let digest = hasher.finalize_hex();

    eprintln!(
        "lunar_series_bit_digest: {rows} rows over {} JDs \
         (range {jd_min} .. {jd_max})\n\
         lunar_series_bit_digest: sha256 {digest}",
        jds.len()
    );

    assert_eq!(
        rows, EXPECTED_ROWS,
        "row count moved: the digest is only comparable across commits that \
         emit the same rows"
    );
    assert!(
        !EXPECTED_DIGEST.is_empty(),
        "\n\nNo lunar-series digest is pinned for this architecture ({}).\n\n\
         measured sha256 {digest}\n\n\
         The digest is architecture-specific; see analytical_bit_digest.rs's \
         EXPECTED_DIGEST doc. This PANICS rather than skipping, because a skip \
         would report green having verified nothing. To pin: confirm the value \
         above is stable across two runs, then add a cfg(target_arch) arm.\n",
        std::env::consts::ARCH
    );
    assert_eq!(
        digest, EXPECTED_DIGEST,
        "\n\nThe raw ELP/MPP02 lunar series' output bits MOVED.\n\n\
         expected {EXPECTED_DIGEST}\n\
         actual   {digest}\n\n\
         Find out what moved and by how much — re-run this test on both \
         commits with VEDAKSHA_DUMP_LUNAR_ROWS=1 and --nocapture and diff the \
         LROW lines — decide whether it is acceptable, and re-pin \
         EXPECTED_DIGEST in a commit that records the reason. Do not re-pin to \
         make a red test green.\n"
    );
}

/// Rows the position-only sweep must emit: (200,001 + 10,000) JDs × 2 surfaces.
const POSITION_ONLY_EXPECTED_ROWS: u64 =
    (FULL_RANGE_SAMPLES as u64 + CONTEMPORARY_SAMPLES as u64) * 2;

/// Digest of the position-only pipeline's 420,002 rows.
///
/// # Measured 2026-08-31, position-only ELP (Wave 2 item #5)
///
/// Companion to [`EXPECTED_DIGEST`], added when `elp_geocentric_position` and
/// `elp_geocentric_position_of_date` were added as new sibling entry points
/// that skip the velocity/omega computation entirely (`docs/audit/2026-08-29-perf-investigation.md`
/// #5). Those two functions are claimed to return exactly the position half
/// of `elp_geocentric`/`elp_geocentric_of_date` at the same `(jd)` — this
/// digest is the independent, 420,002-evaluation confirmation of that claim,
/// over the same JD sweep [`lunar_series_bit_digest`] uses for the full
/// six-component surfaces.
///
/// **If this assertion fails, the position-only path diverged from the
/// position half of the full computation.** Same investigation procedure as
/// [`EXPECTED_DIGEST`]'s doc comment. Do not re-pin to make a red test green.
///
/// **Pinned per architecture**, for the same reason as [`EXPECTED_DIGEST`].
#[cfg(target_arch = "aarch64")]
const POSITION_ONLY_EXPECTED_DIGEST: &str =
    "6edec0b1dd362d83a0a31e993b940b1119d8a49a9062a4afbf9ecd53ec255e93";

/// x86_64 counterpart of [`POSITION_ONLY_EXPECTED_DIGEST`], measured 2026-08-31.
#[cfg(target_arch = "x86_64")]
const POSITION_ONLY_EXPECTED_DIGEST: &str =
    "57821af832a46e6de2cb9c6a20d1bbee6c4e654f55d7ec950850c8a699784363";

/// Sentinel for an unpinned architecture — the test panics on it.
#[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
const POSITION_ONLY_EXPECTED_DIGEST: &str = "";

#[test]
#[ignore = "release-only bit fingerprint of the position-only lunar series; see the module comment"]
fn lunar_series_position_only_bit_digest() {
    require_release_profile();

    let dump = std::env::var_os("VEDAKSHA_DUMP_LUNAR_ROWS").is_some();

    let (jd_min, jd_max) = AnalyticalProvider.time_range();
    let jds = sweep_jds(jd_min, jd_max);

    let mut hasher = sha256::Sha256::new();
    let mut rows = 0u64;
    let mut line = String::with_capacity(96);

    // The two position-only public surfaces of `analytical::elp_mpp02`, each
    // over the whole JD list. Kept as two separate passes, matching
    // `lunar_series_bit_digest`'s per-surface layout, so a divergence
    // confined to one surface is obvious in a row diff.
    for surface in 0..2u8 {
        let tag = match surface {
            0 => "POS_J2000_LLR",
            _ => "POS_OFDATE_LLR",
        };
        for &jd in &jds {
            let (x, y, z) = match surface {
                0 => elp_geocentric_position(jd),
                _ => elp_geocentric_position_of_date(jd),
            };
            line.clear();
            use std::fmt::Write as _;
            write!(
                line,
                "LROW {tag} {jd:.6} {:016x} {:016x} {:016x}",
                x.to_bits(),
                y.to_bits(),
                z.to_bits(),
            )
            .expect("writing to a String cannot fail");
            if dump {
                println!("{line}");
            }
            hasher.update(line.as_bytes());
            hasher.update(b"\n");
            rows += 1;
        }
    }

    let digest = hasher.finalize_hex();

    eprintln!(
        "lunar_series_position_only_bit_digest: {rows} rows over {} JDs \
         (range {jd_min} .. {jd_max})\n\
         lunar_series_position_only_bit_digest: sha256 {digest}",
        jds.len()
    );

    assert_eq!(
        rows, POSITION_ONLY_EXPECTED_ROWS,
        "row count moved: the digest is only comparable across commits that \
         emit the same rows"
    );
    assert!(
        !POSITION_ONLY_EXPECTED_DIGEST.is_empty(),
        "\n\nNo position-only lunar digest is pinned for this architecture \
         ({}).\n\n\
         measured sha256 {digest}\n\n\
         See analytical_bit_digest.rs's EXPECTED_DIGEST doc. This PANICS rather \
         than skipping. To pin: confirm the value is stable across two runs, \
         then add a cfg(target_arch) arm.\n",
        std::env::consts::ARCH
    );
    assert_eq!(
        digest, POSITION_ONLY_EXPECTED_DIGEST,
        "\n\nThe position-only ELP/MPP02 surfaces' output bits MOVED.\n\n\
         expected {POSITION_ONLY_EXPECTED_DIGEST}\n\
         actual   {digest}\n\n\
         Find out what moved and by how much — re-run this test on both \
         commits with VEDAKSHA_DUMP_LUNAR_ROWS=1 and --nocapture and diff the \
         LROW lines — decide whether it is acceptable, and re-pin \
         POSITION_ONLY_EXPECTED_DIGEST in a commit that records the reason. \
         Do not re-pin to make a red test green.\n"
    );
}

/// Streaming SHA-256, FIPS 180-4.
///
/// Vendored for the reason given in the module comment. This is the same
/// algorithm as `analytical_bit_digest.rs`'s copy, restructured to accept the
/// message in chunks: this test's blob is ~76 MB, which the one-shot form
/// would hold twice.
mod sha256 {
    /// FIPS 180-4 §4.2.2 round constants: the first 32 bits of the fractional
    /// parts of the cube roots of the first 64 primes.
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

    pub struct Sha256 {
        h: [u32; 8],
        buf: [u8; 64],
        buf_len: usize,
        len_bytes: u64,
    }

    impl Sha256 {
        pub fn new() -> Self {
            Self {
                h: H0,
                buf: [0u8; 64],
                buf_len: 0,
                len_bytes: 0,
            }
        }

        pub fn update(&mut self, mut data: &[u8]) {
            self.len_bytes += data.len() as u64;

            if self.buf_len > 0 {
                let take = (64 - self.buf_len).min(data.len());
                self.buf[self.buf_len..self.buf_len + take].copy_from_slice(&data[..take]);
                self.buf_len += take;
                data = &data[take..];
                if self.buf_len < 64 {
                    // `data` is exhausted and the block is still partial.
                    // Falling through would run `chunks_exact` over an empty
                    // slice and reset `buf_len` to 0, discarding the partial
                    // block — which is exactly the defect the 1-byte-chunk
                    // case in `matches_the_published_vectors` caught.
                    return;
                }
                let block = self.buf;
                compress(&mut self.h, &block);
                self.buf_len = 0;
            }

            let mut chunks = data.chunks_exact(64);
            for chunk in chunks.by_ref() {
                let block: &[u8; 64] = chunk.try_into().expect("chunks_exact(64) yields 64 bytes");
                compress(&mut self.h, block);
            }
            let rest = chunks.remainder();
            self.buf[..rest.len()].copy_from_slice(rest);
            self.buf_len = rest.len();
        }

        pub fn finalize_hex(mut self) -> String {
            // FIPS 180-4 §5.1.1: append 0x80, then zeros so the length is
            // 56 mod 64, then the message length in BITS, big-endian.
            let bit_len = self.len_bytes * 8;

            // `update` compresses as soon as the buffer fills, so `buf_len`
            // is always in 0..=63 here and this index is in bounds.
            self.buf[self.buf_len] = 0x80;
            self.buf_len += 1;

            if self.buf_len > 56 {
                self.buf[self.buf_len..64].fill(0);
                let block = self.buf;
                compress(&mut self.h, &block);
                self.buf_len = 0;
            }
            self.buf[self.buf_len..56].fill(0);
            self.buf[56..64].copy_from_slice(&bit_len.to_be_bytes());
            let block = self.buf;
            compress(&mut self.h, &block);

            self.h.iter().map(|word| format!("{word:08x}")).collect()
        }
    }

    fn compress(h: &mut [u32; 8], chunk: &[u8; 64]) {
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

        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh] = *h;
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

    #[cfg(test)]
    mod tests {
        /// The two FIPS 180-4 Appendix B vectors, the empty string, and a
        /// multi-block message fed in awkward chunk sizes — the last is what
        /// makes the *streaming* restructuring trustworthy rather than only
        /// the compression function.
        #[test]
        fn matches_the_published_vectors() {
            fn one_shot(data: &[u8]) -> String {
                let mut h = super::Sha256::new();
                h.update(data);
                h.finalize_hex()
            }

            assert_eq!(
                one_shot(b""),
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
            );
            assert_eq!(
                one_shot(b"abc"),
                "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
            );
            assert_eq!(
                one_shot(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"),
                "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
            );

            // 1,000,000 'a' — FIPS 180-4's third Appendix B vector, fed in
            // chunk sizes that straddle every block boundary alignment.
            for chunk in [1usize, 7, 63, 64, 65, 1000] {
                let msg = vec![b'a'; 1_000_000];
                let mut h = super::Sha256::new();
                for part in msg.chunks(chunk) {
                    h.update(part);
                }
                assert_eq!(
                    h.finalize_hex(),
                    "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0",
                    "streaming digest wrong at chunk size {chunk}"
                );
            }
        }
    }
}
