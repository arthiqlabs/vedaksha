// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1
// Contact: info@arthiq.net | https://vedaksha.net

//! `khagola::grahana`: the lunar shadow radii checked against NASA/GSFC's
//! published catalogue, and the solar horizon test.
//!
//! # Where the lunar numbers come from
//!
//! F. Espenak, NASA/GSFC Eclipse Web Site, *Catalog of Lunar Eclipses: 2001 to
//! 2100* (the Five Millennium Catalog of Lunar Eclipses),
//! <https://eclipse.gsfc.nasa.gov/LEcat5/LE2001-2100.html>, read 2026-10-24.
//! Data of NASA, public domain. Only the rows this file checks are kept: the
//! calendar date, the time of greatest eclipse in TD, the type, the penumbral
//! magnitude and the umbral magnitude. The catalogue computes the shadow with
//! Danjon's enlargement (<https://eclipse.gsfc.nasa.gov/LEcat5/shadow.html>,
//! equations 1-5 and 1-6), which is why [`ShadowEnlargement::Danjon`] is the
//! module's default.
//!
//! Over all 228 eclipses of that catalogue (2001 to 2100) the module, run once
//! offline, finds every one and nothing else, with the same type, penumbral and
//! umbral magnitudes within 0.0009, and the time of greatest eclipse within
//! 2.3 s once both sides are in TD. This file keeps the decade 2021 to 2030 and
//! the three eclipses of the century that the module's earlier shadow radii
//! missed or misclassified.
//!
//! # The solar numbers
//!
//! Positions are the engine's apparent positions; the Sun's altitude and
//! sunrise/sunset are cross-checked against [`vedaksha_astro::riseset`], whose
//! sunrise code shares no line with this module. The 2024-04-08 point is from
//! NASA's *Catalog of Solar Eclipses: 2001 to 2100*
//! (<https://eclipse.gsfc.nasa.gov/SEcat5/SE2001-2100.html>): greatest eclipse
//! 18:18:29 TD, total, 25 N 104 W, Sun altitude 70 degrees.

mod common;

use common::{delta_t_s, jd, lunar_geometry, solar_geometry};
use vedaksha_astro::riseset::{SUN_STANDARD_ALTITUDE_DEG, next_rise, sun_rise_set};
use vedaksha_ephem_core::analytical::AnalyticalProvider;
use vedaksha_ephem_core::sidereal_time::gmst;
use vedaksha_vedic::khagola::{
    Grahana, GrahanaKind, LunarGeometry, ShadowEnlargement, SolarGeometry, grahana, grahana_with,
};

struct NasaLunar {
    y: i32,
    m: u32,
    d: u32,
    h: u32,
    mi: u32,
    s: u32,
    class: &'static str,
    pen: f64,
    um: f64,
}

impl NasaLunar {
    /// Greatest eclipse, JD in TD.
    fn jd_td(&self) -> f64 {
        jd(
            self.y,
            self.m,
            self.d,
            f64::from(self.h) + f64::from(self.mi) / 60.0 + f64::from(self.s) / 3600.0,
        )
    }
    fn kind(&self) -> GrahanaKind {
        match self.class.as_bytes()[0] {
            b'T' => GrahanaKind::LunarTotal,
            b'P' => GrahanaKind::LunarPartial,
            _ => GrahanaKind::LunarPenumbral,
        }
    }
}

macro_rules! row {
    ($y:expr, $m:expr, $d:expr, $h:expr, $mi:expr, $s:expr, $class:expr, $pen:expr, $um:expr) => {
        NasaLunar {
            y: $y,
            m: $m,
            d: $d,
            h: $h,
            mi: $mi,
            s: $s,
            class: $class,
            pen: $pen,
            um: $um,
        }
    };
}

/// The decade 2021 to 2030, then 2013-05-25, 2042-09-29 and 2096-06-06.
const NASA: [NasaLunar; 25] = [
    row!(2021, 5, 26, 11, 19, 53, "T", 1.9540, 1.0095),
    row!(2021, 11, 19, 9, 4, 6, "P", 2.0720, 0.9742),
    row!(2022, 5, 16, 4, 12, 42, "T-", 2.3726, 1.4137),
    row!(2022, 11, 8, 11, 0, 22, "T+", 2.4143, 1.3589),
    row!(2023, 5, 5, 17, 24, 5, "N", 0.9636, -0.0457),
    row!(2023, 10, 28, 20, 15, 18, "P", 1.1181, 0.1220),
    row!(2024, 3, 25, 7, 13, 59, "N", 0.9557, -0.1325),
    row!(2024, 9, 18, 2, 45, 25, "P", 1.0372, 0.0848),
    row!(2025, 3, 14, 6, 59, 56, "T", 2.2595, 1.1784),
    row!(2025, 9, 7, 18, 12, 58, "T", 2.3440, 1.3619),
    row!(2026, 3, 3, 11, 34, 52, "T", 2.1838, 1.1507),
    row!(2026, 8, 28, 4, 14, 4, "P", 1.9645, 0.9299),
    row!(2027, 2, 20, 23, 14, 6, "N", 0.9266, -0.0569),
    row!(2027, 7, 18, 16, 4, 9, "Ne", 0.0014, -1.0680),
    row!(2027, 8, 17, 7, 14, 59, "N", 0.5456, -0.5254),
    row!(2028, 1, 12, 4, 14, 13, "P", 1.0468, 0.0662),
    row!(2028, 7, 6, 18, 20, 57, "P", 1.4266, 0.3892),
    row!(2028, 12, 31, 16, 53, 15, "T", 2.2742, 1.2463),
    row!(2029, 6, 26, 3, 23, 22, "T+", 2.8266, 1.8436),
    row!(2029, 12, 20, 22, 43, 12, "T", 2.2008, 1.1174),
    row!(2030, 6, 15, 18, 34, 34, "P", 1.4480, 0.5025),
    row!(2030, 12, 9, 22, 28, 51, "N", 0.9416, -0.1628),
    row!(2013, 5, 25, 4, 11, 6, "Nb", 0.0157, -0.9335),
    row!(2042, 9, 29, 10, 45, 47, "N", 0.9528, -0.0031),
    row!(2096, 6, 6, 2, 43, 41, "Nb", 0.0047, -1.0584),
];

/// How far a magnitude may sit from the catalogue's. Over the whole 2001 to
/// 2100 catalogue the largest difference is 0.0009; Chauvenet's radii would
/// put every penumbral magnitude 0.026 high, the module's earlier ones 0.018
/// low.
const MAG_TOL: f64 = 0.0015;

/// The lunar eclipse nearest NASA's greatest eclipse, searching from 12 days
/// before it.
fn lunar_near(
    p: &AnalyticalProvider,
    row: &NasaLunar,
    shadow: ShadowEnlargement,
) -> Option<Grahana> {
    let lunar = |t: f64| lunar_geometry(p, t);
    let no_solar: fn(f64) -> Option<SolarGeometry> = |_| None;
    let td = row.jd_td();
    let g = grahana_with(td - 12.0, 0.0, 0.0, shadow, &lunar, &no_solar)?;
    let g_td = g.madhya_jd + delta_t_s(g.madhya_jd) / 86_400.0;
    ((g_td - td).abs() < 0.5).then_some(g)
}

#[test]
fn lunar_eclipses_2021_to_2030_are_the_catalogues_and_only_those() {
    let p = AnalyticalProvider::new();
    let lunar = |t: f64| lunar_geometry(&p, t);
    let no_solar: fn(f64) -> Option<SolarGeometry> = |_| None;
    let decade: Vec<&NasaLunar> = NASA
        .iter()
        .filter(|r| (2021..=2030).contains(&r.y))
        .collect();
    assert_eq!(
        decade.len(),
        22,
        "the catalogue lists 22 lunar eclipses in 2021 to 2030"
    );

    let end = jd(2031, 1, 1, 0.0);
    let mut t = jd(2021, 1, 1, 0.0);
    let mut ours = Vec::new();
    while t < end {
        let g = grahana_with(t, 0.0, 0.0, ShadowEnlargement::Danjon, &lunar, &no_solar)
            .expect("a lunar eclipse within 8 full moons");
        t = g.moksha_jd + 1.0;
        if g.madhya_jd < end {
            ours.push(g);
        }
    }
    assert_eq!(
        ours.len(),
        decade.len(),
        "found {} lunar eclipses, the catalogue lists {}",
        ours.len(),
        decade.len()
    );
    for (g, row) in ours.iter().zip(&decade) {
        let td = g.madhya_jd + delta_t_s(g.madhya_jd) / 86_400.0;
        let label = format!("{}-{:02}-{:02}", row.y, row.m, row.d);
        assert!(
            (td - row.jd_td()).abs() * 86_400.0 < 4.0,
            "{label}: greatest eclipse {:.1} s from the catalogue's",
            (td - row.jd_td()) * 86_400.0
        );
        assert_eq!(g.kind, row.kind(), "{label}: type");
        let pen = g
            .penumbral_magnitude
            .expect("lunar eclipse has a penumbral magnitude");
        assert!(
            pen > 0.0,
            "{label}: every reported eclipse touches the penumbra"
        );
        assert!(
            (pen - row.pen).abs() < MAG_TOL,
            "{label}: penumbral magnitude {pen} vs {}",
            row.pen
        );
        assert!(
            (g.magnitude - row.um).abs() < MAG_TOL,
            "{label}: umbral magnitude {} vs {}",
            g.magnitude,
            row.um
        );
        assert!(
            g.sparsha_jd < g.madhya_jd && g.madhya_jd < g.moksha_jd,
            "{label}: order"
        );
        assert!(
            g.horizon.is_none(),
            "{label}: a lunar eclipse carries no horizon state"
        );
    }
}

/// The three eclipses of 2001 to 2100 whose penumbral magnitude is under 0.02,
/// which the module's earlier radii (penumbra not enlarged at all) did not see,
/// and the one whose umbral magnitude they pushed from -0.0031 to +0.0054.
#[test]
fn the_shallowest_penumbral_eclipses_are_found_and_not_promoted() {
    let p = AnalyticalProvider::new();
    for row in NASA
        .iter()
        .filter(|r| (r.y, r.m) == (2013, 5) || (r.y, r.m) == (2027, 7) || (r.y, r.m) == (2096, 6))
    {
        let g = lunar_near(&p, row, ShadowEnlargement::Danjon)
            .unwrap_or_else(|| panic!("{}-{:02}-{:02} is not found", row.y, row.m, row.d));
        let pen = g.penumbral_magnitude.unwrap();
        assert_eq!(g.kind, GrahanaKind::LunarPenumbral);
        assert!(
            pen > 0.0 && (pen - row.pen).abs() < MAG_TOL,
            "pen {pen} vs {}",
            row.pen
        );
        // A magnitude this small is under 4 arcseconds of shadow edge: say so.
        assert!(pen < 0.02, "pen {pen}");
    }
    // 2042-09-29: umbral magnitude -0.0031, so penumbral, not partial.
    let row = NASA.iter().find(|r| (r.y, r.m) == (2042, 9)).unwrap();
    let g = lunar_near(&p, row, ShadowEnlargement::Danjon).unwrap();
    assert_eq!(g.kind, GrahanaKind::LunarPenumbral);
    assert!(g.magnitude < 0.0, "umbral magnitude {}", g.magnitude);
}

/// Chauvenet's radii are Danjon's plus a published margin: the catalogue's
/// page puts the difference at about 0.026 in penumbral magnitude and 0.006 in
/// umbral.
#[test]
fn the_two_shadow_conventions_differ_by_the_published_margin() {
    let p = AnalyticalProvider::new();
    for row in NASA
        .iter()
        .filter(|r| matches!((r.y, r.m), (2023, 5) | (2023, 10) | (2022, 11)))
    {
        let d = lunar_near(&p, row, ShadowEnlargement::Danjon).unwrap();
        let c = lunar_near(&p, row, ShadowEnlargement::Chauvenet).unwrap();
        let dpen = c.penumbral_magnitude.unwrap() - d.penumbral_magnitude.unwrap();
        let dum = c.magnitude - d.magnitude;
        assert!(
            (0.024..0.028).contains(&dpen),
            "{}-{}: penumbral difference {dpen}",
            row.y,
            row.m
        );
        assert!(
            (0.004..0.007).contains(&dum),
            "{}-{}: umbral difference {dum}",
            row.y,
            row.m
        );
        // The default is Danjon.
        let default = lunar_near(&p, row, ShadowEnlargement::default()).unwrap();
        assert_eq!(default.magnitude, d.magnitude);
    }
}

// ── solar ───────────────────────────────────────────────────────

/// The Sun's geometric altitude at `jd_ut` for the observer, degrees, from the
/// engine's apparent geocentric right ascension and declination (Meeus Ch. 13;
/// the parallax of the Sun, 8.8 arcseconds, is ignored).
fn sun_altitude(p: &AnalyticalProvider, jd_ut: f64, lat: f64, lon: f64) -> f64 {
    let g = solar_geometry(p, jd_ut).unwrap();
    let h = (gmst(jd_ut).to_degrees() + lon - g.sun_ra_deg).to_radians();
    let (phi, dec) = (lat.to_radians(), g.sun_dec_deg.to_radians());
    (phi.sin() * dec.sin() + phi.cos() * dec.cos() * h.cos())
        .asin()
        .to_degrees()
}

/// The fraction of the Sun's diameter the Moon covers at `jd_ut` for the
/// observer, `(s_sun + s_moon - separation) / (2 s_sun)`, from Meeus Ch. 40
/// (the parallax in right ascension and declination from the observer's
/// `rho sin phi'` and `rho cos phi'`, Ch. 11), written out here from the book's
/// equations and not from the module's vector form.
fn magnitude_at(p: &AnalyticalProvider, jd_ut: f64, lat: f64, lon: f64) -> f64 {
    let g = solar_geometry(p, jd_ut).unwrap();
    let b_over_a = 1.0 - 1.0 / 298.257_223;
    let phi = lat.to_radians();
    let u = (b_over_a * phi.tan()).atan();
    let (rho_sin, rho_cos) = (b_over_a * u.sin(), u.cos());
    let topo = |ra: f64, dec: f64, dist_au: f64| {
        let sin_pi = 6378.137 / (dist_au * 149_597_870.7);
        let h = (gmst(jd_ut).to_degrees() + lon - ra).to_radians();
        let dec = dec.to_radians();
        let d_ra = (-rho_cos * sin_pi * h.sin()).atan2(dec.cos() - rho_cos * sin_pi * h.cos());
        let dec2 = ((dec.sin() - rho_sin * sin_pi) * d_ra.cos())
            .atan2(dec.cos() - rho_cos * sin_pi * h.cos());
        (ra.to_radians() + d_ra, dec2)
    };
    let (sa, sd) = topo(g.sun_ra_deg, g.sun_dec_deg, g.sun_dist_au);
    let (ma, md) = topo(g.moon_ra_deg, g.moon_dec_deg, g.moon_dist_au);
    let sep = (sd.sin() * md.sin() + sd.cos() * md.cos() * (sa - ma).cos())
        .acos()
        .to_degrees();
    let semi =
        |radius_km: f64, dist_au: f64| (radius_km / (dist_au * 149_597_870.7)).asin().to_degrees();
    let (s_s, s_m) = (semi(695_700.0, g.sun_dist_au), semi(1737.4, g.moon_dist_au));
    (s_s + s_m - sep) / (2.0 * s_s)
}

/// A solar-only search that looks at one new moon: the callback goes dark after
/// `cutoff`, so a place without a visible eclipse at that syzygy returns `None`
/// at once rather than walking on to the next.
fn solar_at(p: &AnalyticalProvider, from: f64, cutoff: f64, lat: f64, lon: f64) -> Option<Grahana> {
    let solar = |t: f64| {
        if t > cutoff {
            None
        } else {
            solar_geometry(p, t)
        }
    };
    let no_lunar: fn(f64) -> Option<LunarGeometry> = |_| None;
    grahana(from, lat, lon, &no_lunar, &solar)
}

const PUNE: (f64, f64) = (18.5204, 73.8567);
const DALLAS: (f64, f64) = (32.7767, -96.797);

/// Three total eclipses at places where the Sun is down (Pune for 2024-04-08
/// and 2026-08-12, Dallas for 2027-08-02): the local time of each madhya is
/// night, so none may be reported.
#[test]
fn eclipses_at_night_are_not_reported() {
    let p = AnalyticalProvider::new();
    let solar = |t: f64| solar_geometry(&p, t);
    let no_lunar: fn(f64) -> Option<LunarGeometry> = |_| None;
    // (place, search from, eclipse that falls in the night there)
    let cases = [
        (
            "Pune 2024-04-08",
            PUNE,
            jd(2024, 4, 1, 0.0),
            jd(2024, 4, 8, 18.77),
        ),
        (
            "Pune 2026-08-12",
            PUNE,
            jd(2026, 8, 1, 0.0),
            jd(2026, 8, 12, 17.85),
        ),
        (
            "Dallas 2027-08-02",
            DALLAS,
            jd(2027, 7, 25, 0.0),
            jd(2027, 8, 2, 8.95),
        ),
    ];
    for (label, (lat, lon), from, night_madhya) in cases {
        // Independent of the module: the Sun is well below the horizon then.
        let alt = sun_altitude(&p, night_madhya, lat, lon);
        assert!(
            alt < -10.0,
            "{label}: the Sun is at {alt} deg, this is not a night case"
        );
        if let Some(g) = grahana(from, lat, lon, &no_lunar, &solar) {
            assert!(
                (g.madhya_jd - night_madhya).abs() > 5.0,
                "{label}: the night-side eclipse is reported (madhya {})",
                g.madhya_jd
            );
        }
    }
}

/// Dallas, 2027-07-25 onward: the next eclipse it can see is the partial one at
/// sunrise on 2028-01-26. Its first contact is below the horizon, so sparsha is
/// sunrise, and that sunrise is the one [`vedaksha_astro::riseset`] finds.
#[test]
fn an_eclipse_in_progress_at_sunrise_starts_at_sunrise() {
    let p = AnalyticalProvider::new();
    let solar = |t: f64| solar_geometry(&p, t);
    let no_lunar: fn(f64) -> Option<LunarGeometry> = |_| None;
    let g = grahana(jd(2027, 7, 25, 0.0), DALLAS.0, DALLAS.1, &no_lunar, &solar).unwrap();
    assert!(
        (jd(2028, 1, 26, 0.0)..jd(2028, 1, 27, 0.0)).contains(&g.madhya_jd),
        "madhya {}",
        g.madhya_jd
    );
    let h = g
        .horizon
        .expect("a solar eclipse carries its horizon state");
    assert!(h.sparsha_at_sunrise && !h.moksha_at_sunset);
    assert!(
        h.geometric_sparsha_jd < g.sparsha_jd,
        "first contact is before sunrise"
    );
    assert_eq!(h.geometric_moksha_jd, g.moksha_jd);
    // The clipped sparsha is sunrise, by the independent rise/set code.
    let eq = |t: f64| solar_geometry(&p, t).map(|s| (s.sun_ra_deg, s.sun_dec_deg));
    let rise = next_rise(h.geometric_sparsha_jd, DALLAS.0, DALLAS.1, 0.0, &eq).unwrap();
    assert!(
        (rise - g.sparsha_jd).abs() * 86_400.0 < 30.0,
        "sparsha {} vs sunrise {rise}: {} s",
        g.sparsha_jd,
        (rise - g.sparsha_jd) * 86_400.0
    );
    let alt = sun_altitude(&p, g.sparsha_jd, DALLAS.0, DALLAS.1);
    assert!(
        (alt - SUN_STANDARD_ALTITUDE_DEG).abs() < 0.01,
        "altitude at sparsha {alt}"
    );
    // Greatest eclipse is in the daylight: the visible magnitude is the whole.
    assert!(h.madhya_visible);
    assert_eq!(h.visible_magnitude, g.magnitude);
}

/// Pune, 2022-10-25: the partial eclipse is under way at sunset, so moksha is
/// sunset, which [`vedaksha_astro::riseset::sun_rise_set`] gives independently.
#[test]
fn an_eclipse_in_progress_at_sunset_ends_at_sunset() {
    let p = AnalyticalProvider::new();
    let g = solar_at(
        &p,
        jd(2022, 10, 20, 0.0),
        jd(2022, 10, 28, 0.0),
        PUNE.0,
        PUNE.1,
    )
    .unwrap();
    assert_eq!(g.kind, GrahanaKind::SolarPartial);
    let h = g.horizon.unwrap();
    assert!(h.moksha_at_sunset && !h.sparsha_at_sunrise, "{h:?}");
    assert!(
        g.moksha_jd < h.geometric_moksha_jd,
        "last contact is after sunset"
    );
    let eq = |t: f64| solar_geometry(&p, t).map(|s| (s.sun_ra_deg, s.sun_dec_deg));
    let day = sun_rise_set(jd(2022, 10, 25, 0.0), PUNE.0, PUNE.1, 0.0, &eq);
    let set = day.set.unwrap();
    assert!(
        (set - g.moksha_jd).abs() * 86_400.0 < 30.0,
        "moksha {} vs sunset {set}: {} s",
        g.moksha_jd,
        (set - g.moksha_jd) * 86_400.0
    );
    let alt = sun_altitude(&p, g.moksha_jd, PUNE.0, PUNE.1);
    assert!(
        (alt - SUN_STANDARD_ALTITUDE_DEG).abs() < 0.01,
        "altitude at moksha {alt}"
    );
    // Greatest eclipse came while the Sun was still up.
    assert!(h.madhya_visible && g.madhya_jd < g.moksha_jd);
}

/// Dallas sees the 2024-04-08 total eclipse whole, in the afternoon.
#[test]
fn dallas_sees_the_2024_total_eclipse_whole() {
    let p = AnalyticalProvider::new();
    let g = solar_at(
        &p,
        jd(2024, 4, 1, 0.0),
        jd(2024, 4, 11, 0.0),
        DALLAS.0,
        DALLAS.1,
    )
    .unwrap();
    assert_eq!(g.kind, GrahanaKind::SolarTotal);
    let h = g.horizon.unwrap();
    assert!(!h.sparsha_at_sunrise && !h.moksha_at_sunset && h.madhya_visible);
    assert_eq!(h.geometric_sparsha_jd, g.sparsha_jd);
    assert_eq!(h.geometric_moksha_jd, g.moksha_jd);
    // Totality in the early afternoon of Dallas, the Sun high.
    let alt = sun_altitude(&p, g.madhya_jd, DALLAS.0, DALLAS.1);
    assert!(alt > 60.0, "altitude at madhya {alt}");
    // About three hours from first to last contact.
    let hours = (g.moksha_jd - g.sparsha_jd) * 24.0;
    assert!((2.5..3.5).contains(&hours), "{hours} h");
}

/// NASA's point of greatest eclipse on 2024-04-08 (25 N 104 W, 18:18:29 TD,
/// Sun altitude 70 degrees): total there, within minutes of NASA's instant.
#[test]
fn the_2024_greatest_eclipse_point_is_total_with_the_catalogues_sun() {
    let p = AnalyticalProvider::new();
    let g = solar_at(&p, jd(2024, 4, 1, 0.0), jd(2024, 4, 11, 0.0), 25.0, -104.0).unwrap();
    assert_eq!(g.kind, GrahanaKind::SolarTotal);
    let nasa_ut =
        jd(2024, 4, 8, 18.0 + 18.0 / 60.0 + 29.0 / 3600.0) - delta_t_s(g.madhya_jd) / 86_400.0;
    assert!(
        (g.madhya_jd - nasa_ut).abs() * 1440.0 < 3.0,
        "madhya {} min from NASA's greatest eclipse",
        (g.madhya_jd - nasa_ut) * 1440.0
    );
    let alt = sun_altitude(&p, g.madhya_jd, 25.0, -104.0);
    assert!(
        (alt - 70.0).abs() < 1.5,
        "Sun altitude {alt}, the catalogue says 70"
    );
}

/// Places across the 2024-04-08 eclipse, among them ones where it begins at
/// sunrise, ends at sunset, or is greatest with the Sun down (every place
/// where it is seen at all, from a 15 degree grid of longitudes, and a few more
/// for company): whatever the module reports must satisfy the horizon rule,
/// tested against the Sun's altitude on a one-minute grid it does not share.
#[test]
fn horizon_rule_holds_at_places_across_the_eclipse() {
    let p = AnalyticalProvider::new();
    let from = jd(2024, 4, 1, 0.0);
    let cutoff = jd(2024, 4, 11, 0.0);
    let places = [
        (-30.0, -150.0),
        (0.0, -165.0),
        (0.0, -150.0),
        (20.0, -165.0),
        (20.0, -30.0),
        (30.0, -180.0),
        (30.0, -30.0),
        (30.0, -15.0),
        (40.0, -30.0),
        (40.0, -15.0),
        (50.0, -30.0),
        (50.0, -15.0),
        (60.0, -15.0),
        (60.0, 0.0),
        (70.0, 0.0),
        (25.0, -104.0),
        (32.7767, -96.797),
        (0.0, 100.0),
        (-60.0, 30.0),
    ];
    let (mut reported, mut sunrise, mut sunset, mut hidden_max) = (0, 0, 0, 0);
    for (lat, lon) in places {
        let Some(g) = solar_at(&p, from, cutoff, lat, lon) else {
            continue;
        };
        reported += 1;
        let h = g.horizon.unwrap();
        // Geometric contacts bracket the visible ones.
        assert!(h.geometric_sparsha_jd <= g.sparsha_jd + 1e-9);
        assert!(g.moksha_jd <= h.geometric_moksha_jd + 1e-9);
        assert!(g.sparsha_jd <= g.moksha_jd);
        // The Sun is up at both visible ends, and at the clipped ones it is on
        // the standard horizon, with the Sun down just outside.
        let up = |t: f64| sun_altitude(&p, t, lat, lon) - SUN_STANDARD_ALTITUDE_DEG;
        assert!(
            up(g.sparsha_jd) > -0.02 && up(g.moksha_jd) > -0.02,
            "({lat}, {lon})"
        );
        if h.sparsha_at_sunrise {
            sunrise += 1;
            assert!(
                up(g.sparsha_jd).abs() < 0.02,
                "({lat}, {lon}): sparsha not on the horizon"
            );
            assert!(up(g.sparsha_jd - 0.002) < 0.0, "({lat}, {lon})");
        } else {
            assert_eq!(g.sparsha_jd, h.geometric_sparsha_jd);
        }
        if h.moksha_at_sunset {
            sunset += 1;
            assert!(
                up(g.moksha_jd).abs() < 0.02,
                "({lat}, {lon}): moksha not on the horizon"
            );
            assert!(up(g.moksha_jd + 0.002) < 0.0, "({lat}, {lon})");
        } else {
            assert_eq!(g.moksha_jd, h.geometric_moksha_jd);
        }
        // Madhya visibility is the Sun's altitude there. When it is down, the
        // largest visible magnitude is the part reached at the clipped end, less
        // than the whole and still positive; when up it is the whole.
        assert_eq!(h.madhya_visible, up(g.madhya_jd) >= 0.0, "({lat}, {lon})");
        if h.madhya_visible {
            assert_eq!(h.visible_magnitude, g.magnitude);
        } else {
            hidden_max += 1;
            assert!(
                h.visible_magnitude > 0.0 && h.visible_magnitude < g.magnitude,
                "({lat}, {lon}): visible magnitude {} of {}",
                h.visible_magnitude,
                g.magnitude
            );
        }
        // Nowhere reported has the Sun below the horizon for all of it: on a
        // one-minute grid some minute is up.
        let (a, b) = (h.geometric_sparsha_jd, h.geometric_moksha_jd);
        let steps = ((b - a) * 1440.0).ceil() as usize;
        let at = |i: usize| a + (b - a) * i as f64 / steps as f64;
        let any_up = (0..=steps).any(|i| up(at(i)) >= 0.0);
        assert!(
            any_up,
            "({lat}, {lon}): reported with the Sun down throughout"
        );
        // The magnitudes against Meeus Ch. 40 on that grid: the whole is the
        // largest over the eclipse, the visible one the largest while the Sun is
        // up, and the magnitude vanishes at the geometric contacts.
        let whole = (0..=steps)
            .map(|i| magnitude_at(&p, at(i), lat, lon))
            .fold(f64::MIN, f64::max);
        // (The visible ends are the sunrise and sunset themselves, which the
        // one-minute grid straddles, and the magnitude there changes by 0.008 a
        // minute: they are sampled too.)
        let seen = (0..=steps)
            .map(at)
            .filter(|&t| up(t) >= 0.0)
            .chain([g.sparsha_jd, g.moksha_jd])
            .map(|t| magnitude_at(&p, t, lat, lon))
            .fold(f64::MIN, f64::max);
        assert!(
            (g.magnitude - whole).abs() < 2.0e-3,
            "({lat}, {lon}): magnitude {} against {whole}",
            g.magnitude
        );
        assert!(
            (h.visible_magnitude - seen).abs() < 2.0e-3,
            "({lat}, {lon}): visible magnitude {} against {seen}",
            h.visible_magnitude
        );
        assert!(
            magnitude_at(&p, a, lat, lon).abs() < 2.0e-3
                && magnitude_at(&p, b, lat, lon).abs() < 2.0e-3,
            "({lat}, {lon}): not zero at the geometric contacts"
        );
    }
    println!(
        "{reported} places, {sunrise} cut by sunrise, {sunset} by sunset, {hidden_max} with the Sun down at the greatest"
    );
    assert!(reported >= 14, "{reported} places saw it");
    assert!(
        sunrise >= 3 && sunset >= 6 && hidden_max >= 4,
        "{sunrise} {sunset} {hidden_max}"
    );
}

/// Places on the far side of the Earth from NASA's greatest eclipse (the
/// antipode of 25 N 104 W is 25 S 76 E): the Sun is deep below the horizon at
/// every moment of the eclipse, yet the Sun and Moon both lie within a degree of
/// the nadir, so a separation test that ignores the horizon calls it an
/// eclipse. None may be reported.
#[test]
fn places_opposite_the_greatest_eclipse_see_nothing() {
    let p = AnalyticalProvider::new();
    let nasa_greatest = jd(2024, 4, 8, 18.3);
    let from = jd(2024, 4, 1, 0.0);
    let cutoff = jd(2024, 4, 11, 0.0);
    let places = [
        (-25.0, 76.0),
        (-20.0, 80.0),
        (-30.0, 70.0),
        (-15.0, 72.0),
        (-5.0, 78.0),
        (-35.0, 85.0),
    ];
    for (lat, lon) in places {
        // Night throughout: the eclipse is a few hours either side of this.
        for dh in [-2.5, 0.0, 2.5] {
            let alt = sun_altitude(&p, nasa_greatest + dh / 24.0, lat, lon);
            assert!(
                alt < -10.0,
                "({lat}, {lon}) at {dh:+} h: the Sun is at {alt} deg"
            );
        }
        assert!(
            solar_at(&p, from, cutoff, lat, lon).is_none(),
            "({lat}, {lon}): an eclipse is reported with the Sun below the horizon"
        );
    }
}
