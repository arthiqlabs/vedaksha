// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1
// Contact: info@arthiq.net | https://vedaksha.net

//! Vimshopaka bala — a graha's twenty-point strength across a varga scheme.
//!
//! Each varga of a scheme carries a full weight, its *swaviswa*; the weights
//! of a scheme sum to 20. In each varga the graha earns that weight scaled by
//! the relation it holds to the rashi it stands in there, the *varga viswa*
//! (out of 20):
//!
//! ```text
//! anka = sum over the scheme's vargas of (swaviswa * varga viswa / 20)
//! ```
//!
//! # Source
//!
//! BPHS chapter 7 (the *varga viveka* chapter), verses 17 to 27, read at the
//! verses in the recension that numbers it chapter 7. The Sanskrit was read in
//! two recensions; a third edition,
//! which runs the rashi descriptions and the vargas together in the third
//! chapter of its first part, numbers the same verses 119 to 129.
//! Grades follow [`crate::drishti`]: (a) read at the verse, editions agree;
//! (b) read at the verse in one recension only; (c) a choice the verse leaves
//! open, made and stated here.
//!
//! * **7.17 and the first half of 7.18, the shadvarga (grade a).** The six
//!   vargas are named in 7.17 (in one recension by their numbers): *lagna*,
//!   *hora*, *dreshkana*, *anka-bhaga* (the part of nine, navamsha),
//!   *surya-amsha* (the part of twelve) and *trimshamsha*. Their weights,
//!   `kramat`, are 6, 2, 4, 5, 2, 1 (`rasa-netra-abdhi-pancha-akshi-
//!   bhumayah`, [`SHADVARGA`]).
//! * **7.18-19, the saptavarga (grade a for the numbers, c for which varga
//!   takes which).** "With the saptamsha", the weights are 5, 2, 3, 2.5, 4.5,
//!   2, 1 (`pancha locanam trayam sardha-dvayam sardha-vedam dvau
//!   ratri-nayakah`). The verse gives the seven numbers and does not say
//!   which varga takes which. Two readings exist. [`SAPTAVARGA`], the
//!   default, gives them in the order of the vargas' numbers (rashi 5, hora 2,
//!   drekkana 3, saptamsha 2.5, navamsha 4.5, dvadashamsha 2, trimshamsha 1),
//!   the order in which the shadvarga verse lists its six with the saptamsha
//!   in its numerical place; it is the reading under which the navamsha keeps
//!   the second-largest weight it has in the shadvarga (5 there, 4.5 here) and
//!   under which no varga's weight moves by more than one point from its
//!   shadvarga weight, the saptamsha being the one that is added.
//!   [`SAPTAVARGA_SEQUENCE`]
//!   reads the seven numbers in the order of the six of 7.17 and then the
//!   saptamsha (navamsha 2.5, dvadashamsha 4.5, trimshamsha 2, saptamsha 1);
//!   some printed tables give this reading. No classical commentary consulted
//!   settles it.
//! * **7.20, the dashavarga (grade a).** The saptavarga with the dashamsha
//!   (`dik-amsha`), shodashamsha (`kalamsha`) and shashtiamsha
//!   (`shashti-bhaga`) added: rashi 3, shashtiamsha 5, each of the other eight
//!   1.5 ([`DASHAVARGA`]).
//! * **7.21-24, the shodashavarga (grade a).** Named in the verses, not by
//!   position: hora 1, trimshamsha 1 and drekkana 1 (one recension names the
//!   trimshamsha, the other calls it `amsha-bhaga`, which its commentators read
//!   as the same), shodashamsha 2, navamsha 3, rashi 3.5 (`sardham ca
//!   tritayam`), shashtiamsha 4, and half each for the remaining nine: D4, D7,
//!   D10, D12, D20, D24, D27, D40, D45 ([`SHODASHAVARGA`]). They sum to 20.
//! * **7.24-25, the relation scale (grade a).** Own varga 20; extreme friend
//!   (*adhimitra*) 18; friend 15; neutral 10; enemy 7; extreme enemy
//!   (*adhishatru*) 5 ([`VargaRelation::viswa`]). These are the five names of
//!   the compound relation of chapter 3, verses 57-58 (natural joined with
//!   temporary), so the relation is the compound one. Which chart the
//!   temporary half is read from the verses do not say; the relation is the
//!   caller's, passed as `viswa`. **The scale has no value of its own for
//!   exaltation or moolatrikona**: the verses give exactly these six. (Verses
//!   7.11-12, before the schemes, say a graha in its own house or exaltation
//!   has full strength, nil in the seventh from it, proportional between;
//!   that is a statement about the rashi, not a seventh viswa, and is left to
//!   a caller who wishes to fold it into `viswa`.)
//! * **7.26-27, the sum (grade a).** Each varga's weight is multiplied by its
//!   viswa and divided by 20. The same verses read the total in bands (under 5
//!   fruitless, to 10 little, to 15 middling, above 15 whole) and 7.30-32 name
//!   eight grades; those are a reading of the score, not part of the
//!   computation, and are not here.
//!
//! # Contract
//!
//! The crate supplies the varga placement and the sum; the scheme weights and
//! the relation come from the caller, the text's weights being available as
//! constants. Longitudes are sidereal degrees. The vargas are those of
//! [`crate::varga`], with the default tradition for the vargas on which
//! traditions diverge (D16, D20, D30, D45); use
//! [`compute_vimshopaka_with_tradition`] for the other.

use crate::graha::Graha;
use crate::varga::{VargaTradition, VargaType, varga_sign_with_tradition};
use serde::Serialize;

/// The shadvarga weights of BPHS 7.17: rashi 6, hora 2, drekkana 4,
/// navamsha 5, dvadashamsha 2, trimshamsha 1.
pub const SHADVARGA: [(VargaType, f64); 6] = [
    (VargaType::Rashi, 6.0),
    (VargaType::Hora, 2.0),
    (VargaType::Drekkana, 4.0),
    (VargaType::Navamsha, 5.0),
    (VargaType::Dwadashamsha, 2.0),
    (VargaType::Trimshamsha, 1.0),
];

/// The saptavarga weights of BPHS 7.18-19 in the order of the vargas'
/// numbers: rashi 5, hora 2, drekkana 3, saptamsha 2.5, navamsha 4.5,
/// dvadashamsha 2, trimshamsha 1. The default reading; see the module
/// documentation for the other, [`SAPTAVARGA_SEQUENCE`].
pub const SAPTAVARGA: [(VargaType, f64); 7] = [
    (VargaType::Rashi, 5.0),
    (VargaType::Hora, 2.0),
    (VargaType::Drekkana, 3.0),
    (VargaType::Saptamsha, 2.5),
    (VargaType::Navamsha, 4.5),
    (VargaType::Dwadashamsha, 2.0),
    (VargaType::Trimshamsha, 1.0),
];

/// The other reading of BPHS 7.18-19: the seven numbers 5, 2, 3, 2.5, 4.5,
/// 2, 1 taken in the order of the six of 7.17 and then the saptamsha: rashi
/// 5, hora 2, drekkana 3, navamsha 2.5, dvadashamsha 4.5, trimshamsha 2,
/// saptamsha 1.
pub const SAPTAVARGA_SEQUENCE: [(VargaType, f64); 7] = [
    (VargaType::Rashi, 5.0),
    (VargaType::Hora, 2.0),
    (VargaType::Drekkana, 3.0),
    (VargaType::Navamsha, 2.5),
    (VargaType::Dwadashamsha, 4.5),
    (VargaType::Trimshamsha, 2.0),
    (VargaType::Saptamsha, 1.0),
];

/// The dashavarga weights of BPHS 7.20: rashi 3, shashtiamsha 5, each of the
/// other eight (hora, drekkana, saptamsha, navamsha, dashamsha,
/// dvadashamsha, shodashamsha, trimshamsha) 1.5.
pub const DASHAVARGA: [(VargaType, f64); 10] = [
    (VargaType::Rashi, 3.0),
    (VargaType::Hora, 1.5),
    (VargaType::Drekkana, 1.5),
    (VargaType::Saptamsha, 1.5),
    (VargaType::Navamsha, 1.5),
    (VargaType::Dashamsha, 1.5),
    (VargaType::Dwadashamsha, 1.5),
    (VargaType::Shodashamsha, 1.5),
    (VargaType::Trimshamsha, 1.5),
    (VargaType::Shashtiamsha, 5.0),
];

/// The shodashavarga weights of BPHS 7.21-24: rashi 3.5, hora 1, drekkana 1,
/// navamsha 3, shodashamsha 2, trimshamsha 1, shashtiamsha 4, and 0.5 for each
/// of the other nine.
pub const SHODASHAVARGA: [(VargaType, f64); 16] = [
    (VargaType::Rashi, 3.5),
    (VargaType::Hora, 1.0),
    (VargaType::Drekkana, 1.0),
    (VargaType::Chaturthamsha, 0.5),
    (VargaType::Saptamsha, 0.5),
    (VargaType::Navamsha, 3.0),
    (VargaType::Dashamsha, 0.5),
    (VargaType::Dwadashamsha, 0.5),
    (VargaType::Shodashamsha, 2.0),
    (VargaType::Vimshamsha, 0.5),
    (VargaType::ChaturVimshamsha, 0.5),
    (VargaType::Saptavimshamsha, 0.5),
    (VargaType::Trimshamsha, 1.0),
    (VargaType::Khavedamsha, 0.5),
    (VargaType::Akshavedamsha, 0.5),
    (VargaType::Shashtiamsha, 4.0),
];

/// The varga viswa of BPHS 7.24-25: the value, out of 20, of the relation a
/// graha holds to the rashi it stands in, in a varga.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum VargaRelation {
    /// In its own varga rashi: 20.
    Own,
    /// Extreme friend (natural and temporary friend): 18.
    Adhimitra,
    /// Friend: 15.
    Mitra,
    /// Neutral: 10.
    Sama,
    /// Enemy: 7.
    Shatru,
    /// Extreme enemy (natural and temporary enemy): 5.
    Adhishatru,
}

impl VargaRelation {
    /// The viswa out of 20.
    #[must_use]
    pub const fn viswa(self) -> f64 {
        match self {
            Self::Own => 20.0,
            Self::Adhimitra => 18.0,
            Self::Mitra => 15.0,
            Self::Sama => 10.0,
            Self::Shatru => 7.0,
            Self::Adhishatru => 5.0,
        }
    }
}

/// One varga's part of a graha's vimshopaka.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct VimshopakaVarga {
    /// Which varga.
    pub varga: VargaType,
    /// The varga rashi the graha stands in, zero-based (0 = Aries).
    pub sign: u8,
    /// The scheme's full weight for this varga (swaviswa).
    pub swaviswa: f64,
    /// The relation's value, out of 20 (varga viswa).
    pub viswa: f64,
    /// `swaviswa * viswa / 20`.
    pub anka: f64,
}

/// A graha's vimshopaka across one scheme.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Vimshopaka {
    /// The graha.
    pub graha: Graha,
    /// Each varga of the scheme, in the scheme's order.
    pub vargas: Vec<VimshopakaVarga>,
    /// The sum of the vargas' `anka`, 0 to 20.
    pub anka: f64,
}

/// A graha's vimshopaka over `scheme`, with the default varga tradition.
///
/// `scheme` is `(varga, swaviswa)` pairs summing to 20; the text's schemes are
/// [`SHADVARGA`], [`SAPTAVARGA`], [`DASHAVARGA`] and [`SHODASHAVARGA`].
/// `viswa(varga, sign)` returns the relation's value out of 20 for the graha in
/// that varga rashi (see [`VargaRelation::viswa`]); it is called once per
/// varga, in the scheme's order.
///
/// # Panics
///
/// Panics when the weights of `scheme` do not sum to 20: the result would not
/// be a score out of 20.
#[must_use]
pub fn compute_vimshopaka(
    graha: Graha,
    longitude_deg: f64,
    scheme: &[(VargaType, f64)],
    viswa: &dyn Fn(VargaType, u8) -> f64,
) -> Vimshopaka {
    compute_vimshopaka_with_tradition(
        graha,
        longitude_deg,
        scheme,
        viswa,
        VargaTradition::Modality,
    )
}

/// [`compute_vimshopaka`] with the varga tradition chosen for the vargas on
/// which they diverge (D16, D20, D30, D45).
///
/// # Panics
///
/// Panics when the weights of `scheme` do not sum to 20.
#[must_use]
pub fn compute_vimshopaka_with_tradition(
    graha: Graha,
    longitude_deg: f64,
    scheme: &[(VargaType, f64)],
    viswa: &dyn Fn(VargaType, u8) -> f64,
    tradition: VargaTradition,
) -> Vimshopaka {
    let total: f64 = scheme.iter().map(|&(_, w)| w).sum();
    assert!(
        (total - 20.0).abs() < 1e-9,
        "a vimshopaka scheme's weights must sum to 20, got {total}"
    );
    let vargas: Vec<VimshopakaVarga> = scheme
        .iter()
        .map(|&(varga, swaviswa)| {
            let sign = varga_sign_with_tradition(longitude_deg, varga, tradition);
            let v = viswa(varga, sign);
            VimshopakaVarga {
                varga,
                sign,
                swaviswa,
                viswa: v,
                anka: swaviswa * v / 20.0,
            }
        })
        .collect();
    let anka = vargas.iter().map(|v| v.anka).sum();
    Vimshopaka {
        graha,
        vargas,
        anka,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::varga::varga_sign;
    use std::cell::RefCell;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    fn weight(scheme: &[(VargaType, f64)], v: VargaType) -> f64 {
        scheme
            .iter()
            .find(|&&(x, _)| x == v)
            .unwrap_or_else(|| panic!("{v:?} missing"))
            .1
    }

    const ALL_SCHEMES: [(&str, &[(VargaType, f64)]); 5] = [
        ("shadvarga", &SHADVARGA),
        ("saptavarga", &SAPTAVARGA),
        ("saptavarga sequence", &SAPTAVARGA_SEQUENCE),
        ("dashavarga", &DASHAVARGA),
        ("shodashavarga", &SHODASHAVARGA),
    ];

    /// Each scheme's weights sum to 20, with no
    /// varga listed twice.
    #[test]
    fn every_scheme_sums_to_twenty_over_distinct_vargas() {
        for (name, scheme) in ALL_SCHEMES {
            let total: f64 = scheme.iter().map(|&(_, w)| w).sum();
            assert!(close(total, 20.0), "{name}: {total}");
            for (i, &(a, _)) in scheme.iter().enumerate() {
                assert!(
                    scheme[i + 1..].iter().all(|&(b, _)| a != b),
                    "{name}: {a:?} twice"
                );
            }
        }
    }

    /// 7.17: the six, by the verse's own naming, and their weights 6 2 4 5 2 1.
    #[test]
    fn shadvarga_is_the_six_of_verse_17() {
        assert_eq!(SHADVARGA.len(), 6);
        let want = [
            (VargaType::Rashi, 6.0),
            (VargaType::Hora, 2.0),
            (VargaType::Drekkana, 4.0),
            (VargaType::Navamsha, 5.0),
            (VargaType::Dwadashamsha, 2.0),
            (VargaType::Trimshamsha, 1.0),
        ];
        for (v, w) in want {
            assert!(close(weight(&SHADVARGA, v), w), "{v:?}");
        }
    }

    /// 7.18-19: the seven numbers 5 2 3 2.5 4.5 2 1 in each reading, and the
    /// two readings differ in exactly the vargas whose place in the order
    /// moves (D7, D9, D12, D30).
    #[test]
    fn the_two_saptavarga_readings_use_the_same_seven_numbers() {
        let mut a: Vec<f64> = SAPTAVARGA.iter().map(|&(_, w)| w).collect();
        let mut b: Vec<f64> = SAPTAVARGA_SEQUENCE.iter().map(|&(_, w)| w).collect();
        a.sort_by(f64::total_cmp);
        b.sort_by(f64::total_cmp);
        assert_eq!(a, b);
        assert_eq!(a, vec![1.0, 2.0, 2.0, 2.5, 3.0, 4.5, 5.0]);
        // The same seven vargas in both.
        for &(v, _) in &SAPTAVARGA {
            assert!(SAPTAVARGA_SEQUENCE.iter().any(|&(x, _)| x == v), "{v:?}");
        }
        // Agree on rashi 5, hora 2, drekkana 3.
        for (v, w) in [
            (VargaType::Rashi, 5.0),
            (VargaType::Hora, 2.0),
            (VargaType::Drekkana, 3.0),
        ] {
            assert!(close(weight(&SAPTAVARGA, v), w));
            assert!(close(weight(&SAPTAVARGA_SEQUENCE, v), w));
        }
        // Division-number order: saptamsha 2.5, navamsha 4.5, dvadashamsha 2,
        // trimshamsha 1.
        for (v, w) in [
            (VargaType::Saptamsha, 2.5),
            (VargaType::Navamsha, 4.5),
            (VargaType::Dwadashamsha, 2.0),
            (VargaType::Trimshamsha, 1.0),
        ] {
            assert!(close(weight(&SAPTAVARGA, v), w), "{v:?}");
        }
        // Sequence order: the six of 7.17 then the saptamsha.
        for (v, w) in [
            (VargaType::Navamsha, 2.5),
            (VargaType::Dwadashamsha, 4.5),
            (VargaType::Trimshamsha, 2.0),
            (VargaType::Saptamsha, 1.0),
        ] {
            assert!(close(weight(&SAPTAVARGA_SEQUENCE, v), w), "{v:?}");
        }
    }

    /// 7.20: the dashavarga is the saptavarga's vargas plus D10, D16, D60;
    /// rashi 3, shashtiamsha 5, the other eight 1.5.
    #[test]
    fn dashavarga_is_the_saptavarga_plus_three() {
        for &(v, _) in &SAPTAVARGA {
            assert!(DASHAVARGA.iter().any(|&(x, _)| x == v), "{v:?}");
        }
        for v in [
            VargaType::Dashamsha,
            VargaType::Shodashamsha,
            VargaType::Shashtiamsha,
        ] {
            assert!(DASHAVARGA.iter().any(|&(x, _)| x == v), "{v:?}");
        }
        assert_eq!(DASHAVARGA.len(), 10);
        assert!(close(weight(&DASHAVARGA, VargaType::Rashi), 3.0));
        assert!(close(weight(&DASHAVARGA, VargaType::Shashtiamsha), 5.0));
        for &(v, w) in &DASHAVARGA {
            if !matches!(v, VargaType::Rashi | VargaType::Shashtiamsha) {
                assert!(close(w, 1.5), "{v:?}");
            }
        }
    }

    /// 7.21-24: all sixteen vargas once, the named weights, and half for the
    /// nine unnamed.
    #[test]
    fn shodashavarga_covers_all_sixteen_with_the_named_weights() {
        assert_eq!(SHODASHAVARGA.len(), 16);
        let named = [
            (VargaType::Rashi, 3.5),
            (VargaType::Hora, 1.0),
            (VargaType::Drekkana, 1.0),
            (VargaType::Navamsha, 3.0),
            (VargaType::Shodashamsha, 2.0),
            (VargaType::Trimshamsha, 1.0),
            (VargaType::Shashtiamsha, 4.0),
        ];
        for (v, w) in named {
            assert!(close(weight(&SHODASHAVARGA, v), w), "{v:?}");
        }
        let halves = [
            VargaType::Chaturthamsha,
            VargaType::Saptamsha,
            VargaType::Dashamsha,
            VargaType::Dwadashamsha,
            VargaType::Vimshamsha,
            VargaType::ChaturVimshamsha,
            VargaType::Saptavimshamsha,
            VargaType::Khavedamsha,
            VargaType::Akshavedamsha,
        ];
        assert_eq!(halves.len(), 9);
        for v in halves {
            assert!(close(weight(&SHODASHAVARGA, v), 0.5), "{v:?}");
        }
        // Division numbers 1,2,3,4,7,9,10,12,16,20,24,27,30,40,45,60 once each.
        let mut divs: Vec<u32> = SHODASHAVARGA.iter().map(|&(v, _)| v.division()).collect();
        divs.sort_unstable();
        assert_eq!(
            divs,
            vec![1, 2, 3, 4, 7, 9, 10, 12, 16, 20, 24, 27, 30, 40, 45, 60]
        );
    }

    /// 7.24-25: the six values of the scale.
    #[test]
    fn relation_scale_is_the_six_values_of_verse_25() {
        assert!(close(VargaRelation::Own.viswa(), 20.0));
        assert!(close(VargaRelation::Adhimitra.viswa(), 18.0));
        assert!(close(VargaRelation::Mitra.viswa(), 15.0));
        assert!(close(VargaRelation::Sama.viswa(), 10.0));
        assert!(close(VargaRelation::Shatru.viswa(), 7.0));
        assert!(close(VargaRelation::Adhishatru.viswa(), 5.0));
    }

    /// In its own varga rashi everywhere, a
    /// graha scores the whole 20, in every scheme.
    #[test]
    fn own_everywhere_scores_twenty() {
        let own = |_: VargaType, _: u8| VargaRelation::Own.viswa();
        for (name, scheme) in ALL_SCHEMES {
            let v = compute_vimshopaka(Graha::Mars, 100.0, scheme, &own);
            assert!(close(v.anka, 20.0), "{name}: {}", v.anka);
        }
    }

    /// Every relation neutral scores 10.
    #[test]
    fn neutral_everywhere_scores_ten() {
        let neutral = |_: VargaType, _: u8| VargaRelation::Sama.viswa();
        for (name, scheme) in ALL_SCHEMES {
            let v = compute_vimshopaka(Graha::Venus, 211.7, scheme, &neutral);
            assert!(close(v.anka, 10.0), "{name}: {}", v.anka);
        }
    }

    /// One varga's part is its weight times
    /// the relation over 20. Rashi 6 (shadvarga) at extreme friend 18 gives
    /// 5.4.
    #[test]
    fn one_varga_contribution_is_weight_times_relation_over_twenty() {
        let rashi_adhimitra_rest_neutral = |v: VargaType, _: u8| {
            if v == VargaType::Rashi {
                VargaRelation::Adhimitra.viswa()
            } else {
                0.0
            }
        };
        let r = compute_vimshopaka(
            Graha::Jupiter,
            33.0,
            &SHADVARGA,
            &rashi_adhimitra_rest_neutral,
        );
        let rashi = r.vargas[0];
        assert_eq!(rashi.varga, VargaType::Rashi);
        assert!(close(rashi.swaviswa, 6.0));
        assert!(close(rashi.viswa, 18.0));
        assert!(close(rashi.anka, 5.4));
        // Everything else was given 0, so the total is the one part.
        assert!(close(r.anka, 5.4));
    }

    /// A full hand sum over the shadvarga, relations chosen per varga:
    /// rashi 18 -> 6*18/20 = 5.4; hora 10 -> 2*10/20 = 1.0; drekkana 18 ->
    /// 4*18/20 = 3.6; navamsha 15 -> 5*15/20 = 3.75; dvadashamsha 7 ->
    /// 2*7/20 = 0.7; trimshamsha 20 -> 1*20/20 = 1.0. Total 15.45.
    #[test]
    fn shadvarga_hand_sum() {
        let table = |v: VargaType, _: u8| match v {
            VargaType::Rashi | VargaType::Drekkana => 18.0,
            VargaType::Hora => 10.0,
            VargaType::Navamsha => 15.0,
            VargaType::Dwadashamsha => 7.0,
            _ => 20.0,
        };
        let r = compute_vimshopaka(Graha::Venus, 275.0, &SHADVARGA, &table);
        let parts: Vec<f64> = r.vargas.iter().map(|v| v.anka).collect();
        let want = [5.4, 1.0, 3.6, 3.75, 0.7, 1.0];
        for (p, w) in parts.iter().zip(want) {
            assert!(close(*p, w), "{parts:?}");
        }
        assert!(close(r.anka, 15.45));
    }

    /// The viswa callback is asked for exactly the (varga, sign) the varga
    /// module places the longitude in, once per varga, in the scheme's order;
    /// and the result carries the same signs.
    #[test]
    fn viswa_is_asked_for_the_varga_placement() {
        let lon = 100.0;
        let asked = RefCell::new(Vec::new());
        let record = |v: VargaType, s: u8| {
            asked.borrow_mut().push((v, s));
            10.0
        };
        let r = compute_vimshopaka(Graha::Moon, lon, &DASHAVARGA, &record);
        let expected: Vec<(VargaType, u8)> = DASHAVARGA
            .iter()
            .map(|&(v, _)| (v, varga_sign(lon, v)))
            .collect();
        assert_eq!(*asked.borrow(), expected);
        let got: Vec<(VargaType, u8)> = r.vargas.iter().map(|v| (v.varga, v.sign)).collect();
        assert_eq!(got, expected);
        assert_eq!(r.graha, Graha::Moon);
    }

    /// The tradition reaches the placement: at Cancer 10 degrees the movable
    /// (default) D16 starts from Aries and the element tradition from Cancer,
    /// 5 parts in: Aries + 5 = Virgo (5), Cancer + 5 = Sagittarius (8).
    #[test]
    fn tradition_reaches_the_placement() {
        let flat = |_: VargaType, _: u8| 10.0;
        let d16 = |t: VargaTradition| {
            let r = compute_vimshopaka_with_tradition(Graha::Moon, 100.0, &SHODASHAVARGA, &flat, t);
            r.vargas
                .iter()
                .find(|v| v.varga == VargaType::Shodashamsha)
                .unwrap()
                .sign
        };
        assert_eq!(d16(VargaTradition::Modality), 5);
        assert_eq!(d16(VargaTradition::Element), 8);
        // And the default entry point is the modality tradition.
        let r = compute_vimshopaka(Graha::Moon, 100.0, &SHODASHAVARGA, &flat);
        let default_d16 = r
            .vargas
            .iter()
            .find(|v| v.varga == VargaType::Shodashamsha)
            .unwrap()
            .sign;
        assert_eq!(default_d16, 5);
    }

    #[test]
    #[should_panic(expected = "must sum to 20")]
    fn a_scheme_not_summing_to_twenty_panics() {
        let flat = |_: VargaType, _: u8| 10.0;
        let short = [(VargaType::Rashi, 6.0), (VargaType::Navamsha, 5.0)];
        let _ = compute_vimshopaka(Graha::Sun, 10.0, &short, &flat);
    }
}
