// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1
// Contact: info@arthiq.net | https://vedaksha.net

//! Shadbala for a computed chart: the complete form, fed from a chart and a
//! place.
//!
//! [`crate::shadbala::compute_shadbala_complete`] wants a birth moment (the
//! sunrise that begins the Hindu day, the sunset, the next sunrise, a day
//! label, the local apparent time), the four angular cusps and the grahas.
//! This module derives them from a [`ComputedChart`] and the observer, so the
//! MCP and WASM surfaces serve one routine's answer and cannot drift apart.
//! Every rule of the strengths themselves is in [`crate::shadbala`]; what is
//! decided here is only where each input comes from.
//!
//! * **Sunrise, sunset and the next sunrise** are the engine's own, for the
//!   observer's place and elevation (`vedaksha_astro::riseset`): the sunrise
//!   is the last one at or before the birth, the sunset the first after it,
//!   and the next sunrise the first after the birth. Where the Sun does not
//!   both rise and set (the polar day and night) there is no Hindu day to
//!   divide into thirds and horas, and the result is
//!   [`ChartShadbalaError::NoSunEvent`] rather than a guess.
//! * **The Hindu day label** is the local calendar date of that sunrise, at
//!   the observer's clock offset ([`hindu_day_jdn`]).
//! * **The local apparent time** is the hour angle of the true Sun plus 12
//!   hours: the local apparent sidereal time less the Sun's right ascension,
//!   from the same Sun position the sunrise search reads.
//! * **The cusps** are the chart's own: the first, fourth, seventh and tenth
//!   of its house system, in the chart's zodiac.
//! * **The nodes.** Rahu is the chart's `MeanNode` and Ketu opposite it; they
//!   receive no strength and enter only as possible malefic company for
//!   Mercury (BPHS 3.11). A chart without a `MeanNode` simply has none.
//! * **Motion.** Each graha also carries its motion state of BPHS 27.21 to
//!   27.23 ([`crate::cheshta`]), judged against the mean geocentric motion at
//!   the instant ([`mean_geocentric_motion`]). It is reported beside the
//!   strengths and is not part of any total: the cheshta bala of the complete
//!   form is the kendra rule of 27.24 and 27.25, which reads no mean speed.

use serde::Serialize;
use vedaksha_astro::chart::ComputedChart;
use vedaksha_astro::riseset::{next_rise, previous_rise, sun_equatorial_deg, sun_rise_set};
use vedaksha_ephem_core::analytical::AnalyticalProvider;
use vedaksha_ephem_core::{delta_t, nutation, obliquity, sidereal_time};

use crate::cheshta::{Cheshta, cheshta_at};
use crate::graha::{Graha, GrahaPosition};
use crate::mean_motion::mean_geocentric_motion;
use crate::shadbala::{
    AngularCusps, CompletePlanetData, CompleteShadbala, KalaMoment, ShadbalaContext, ShadbalaError,
    ShadbalaOptions, apparent_hours_from_sun_hour_angle, compute_shadbala_complete, hindu_day_jdn,
};

/// The chart body standing for Rahu.
pub const RAHU_BODY_NAME: &str = "MeanNode";

/// The seven grahas of shadbala as a chart names them.
const CHART_NAMES: [(&str, Graha); 7] = [
    ("Sun", Graha::Sun),
    ("Moon", Graha::Moon),
    ("Mars", Graha::Mars),
    ("Mercury", Graha::Mercury),
    ("Jupiter", Graha::Jupiter),
    ("Venus", Graha::Venus),
    ("Saturn", Graha::Saturn),
];

/// A chart and the place and instant it was cast for.
#[derive(Debug, Clone, Copy)]
pub struct ChartShadbalaRequest<'a> {
    /// The chart. Its longitudes and cusps are in the sidereal zodiac the
    /// ayanamsha below defines.
    pub chart: &'a ComputedChart,
    /// The instant, Julian Day (UT1).
    pub jd_ut: f64,
    /// Geographic latitude, degrees, north positive.
    pub latitude_deg: f64,
    /// Geographic longitude, degrees, east positive.
    pub longitude_deg: f64,
    /// Elevation above sea level, metres: it lowers the horizon and so moves
    /// sunrise and sunset.
    pub elevation_m: f64,
    /// The observer's clock offset from UT, minutes: it names the calendar
    /// date that labels the Hindu day.
    pub tz_offset_minutes: i32,
    /// The ayanamsha in degrees that the chart's longitudes had taken off
    /// (the true ayanamsha for the instant; 0 for a tropical chart).
    pub ayanamsha_deg: f64,
}

/// Why a chart's shadbala could not be computed.
#[derive(Debug, Clone, PartialEq)]
pub enum ChartShadbalaError {
    /// The chart lacks one of the bodies shadbala reads.
    MissingBody(&'static str),
    /// The Sun does not have the named event, so there is no Hindu day to
    /// divide: the polar day or night, or an ephemeris that gave no position.
    NoSunEvent(&'static str),
    /// The complete form refused its inputs.
    Shadbala(ShadbalaError),
}

impl core::fmt::Display for ChartShadbalaError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::MissingBody(name) => write!(f, "the chart has no {name}"),
            Self::NoSunEvent(what) => write!(
                f,
                "no {what} at this place and time (polar day or night): the Hindu day cannot \
                 be divided, and tribhaga, hora and nathonnata bala need it"
            ),
            Self::Shadbala(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for ChartShadbalaError {}

impl From<ShadbalaError> for ChartShadbalaError {
    fn from(e: ShadbalaError) -> Self {
        Self::Shadbala(e)
    }
}

/// A graha's motion state at the birth (BPHS 27.21 to 27.23).
#[derive(Debug, Clone, Copy, Serialize)]
pub struct MotionState {
    /// The state, by [`cheshta_at`].
    pub state: Cheshta,
    /// The cheshta bala that state carries in the verse, virupas
    /// ([`Cheshta::virupas`]). Not part of the shadbala total.
    pub virupas: f64,
    /// The graha's speed in longitude, degrees per day, negative when
    /// retrograde.
    pub speed_deg_per_day: f64,
    /// The mean geocentric motion it was compared with, degrees per day.
    pub mean_speed_deg_per_day: f64,
}

/// One graha's complete shadbala and its motion state.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct GrahaShadbala {
    /// Every part of the complete form.
    #[serde(flatten)]
    pub strength: CompleteShadbala,
    /// The graha's motion state.
    pub motion: MotionState,
}

/// The shadbala of a chart.
#[derive(Debug, Clone, Serialize)]
pub struct ChartShadbala {
    /// The birth moment the kala bala was reckoned from.
    pub moment: KalaMoment,
    /// The cusps dig bala was measured from.
    pub cusps: AngularCusps,
    /// The seven grahas, Sun to Saturn.
    pub planets: Vec<GrahaShadbala>,
}

/// The birth moment for an observer: sunrise, sunset and the next sunrise from
/// the engine's own rise and set, the Hindu day label and the local apparent
/// time. See the module documentation for each.
///
/// # Errors
/// [`ChartShadbalaError::NoSunEvent`] where the Sun has no sunrise, sunset or
/// next sunrise for the observer.
pub fn kala_moment(
    jd_ut: f64,
    latitude_deg: f64,
    longitude_deg: f64,
    elevation_m: f64,
    tz_offset_minutes: i32,
) -> Result<KalaMoment, ChartShadbalaError> {
    let provider = AnalyticalProvider;
    let equatorial = |jd: f64| sun_equatorial_deg(&provider, jd);

    let jd_sunrise = previous_rise(jd_ut, latitude_deg, longitude_deg, elevation_m, &equatorial)
        .ok_or(ChartShadbalaError::NoSunEvent("sunrise"))?;
    // The sunset that follows that sunrise: the Sun is up at `jd_sunrise` and
    // stays up until it sets, so the first set in the next day is the one.
    let jd_sunset = sun_rise_set(
        jd_sunrise,
        latitude_deg,
        longitude_deg,
        elevation_m,
        &equatorial,
    )
    .set
    .ok_or(ChartShadbalaError::NoSunEvent("sunset"))?;
    let jd_next_sunrise = next_rise(jd_ut, latitude_deg, longitude_deg, elevation_m, &equatorial)
        .ok_or(ChartShadbalaError::NoSunEvent("next sunrise"))?;

    // Hour angle of the true Sun: local apparent sidereal time less its right
    // ascension. Rotation takes UT1, nutation and obliquity take TT.
    let (right_ascension_deg, _) =
        equatorial(jd_ut).ok_or(ChartShadbalaError::NoSunEvent("position of the Sun"))?;
    let jd_tt = delta_t::ut1_to_tt(jd_ut);
    let (dpsi, deps) = nutation::nutation(jd_tt);
    let true_obliquity = obliquity::true_obliquity(jd_tt, deps);
    let last_deg =
        sidereal_time::local_sidereal_time(jd_ut, longitude_deg.to_radians(), dpsi, true_obliquity)
            .to_degrees();
    let hour_angle_deg = (last_deg - right_ascension_deg).rem_euclid(360.0);

    Ok(KalaMoment {
        jd: jd_ut,
        jd_sunrise,
        jd_sunset,
        jd_next_sunrise,
        hindu_day_jdn: hindu_day_jdn(jd_sunrise, f64::from(tz_offset_minutes) / 60.0),
        apparent_hours_from_midnight: apparent_hours_from_sun_hour_angle(hour_angle_deg),
    })
}

/// The complete shadbala of a chart (see the module documentation for where
/// each input comes from, and [`crate::shadbala`] for every rule).
///
/// # Errors
/// [`ChartShadbalaError::MissingBody`] if the chart lacks one of the seven
/// grahas, [`ChartShadbalaError::NoSunEvent`] in the polar day or night,
/// [`ChartShadbalaError::Shadbala`] if the complete form refuses its inputs.
pub fn shadbala_for_chart(
    request: &ChartShadbalaRequest<'_>,
    options: &ShadbalaOptions<'_>,
) -> Result<ChartShadbala, ChartShadbalaError> {
    let chart = request.chart;
    let body = |name: &'static str| {
        chart
            .planets
            .iter()
            .find(|p| p.name == name)
            .ok_or(ChartShadbalaError::MissingBody(name))
    };

    let mut planets = Vec::with_capacity(9);
    for (name, graha) in CHART_NAMES {
        let p = body(name)?;
        planets.push(CompletePlanetData {
            position: GrahaPosition {
                planet: graha,
                sign: p.sign_index,
                longitude: p.longitude,
                bhava: p.house,
            },
            latitude_deg: p.latitude,
            declination_deg: None,
        });
    }
    // The nodes: company for Mercury only.
    if let Some(rahu) = chart.planets.iter().find(|p| p.name == RAHU_BODY_NAME) {
        let ketu_longitude = (rahu.longitude + 180.0).rem_euclid(360.0);
        for (graha, longitude, sign, bhava) in [
            (Graha::Rahu, rahu.longitude, rahu.sign_index, rahu.house),
            (
                Graha::Ketu,
                ketu_longitude,
                (rahu.sign_index + 6) % 12,
                (rahu.house + 5) % 12 + 1,
            ),
        ] {
            planets.push(CompletePlanetData {
                position: GrahaPosition {
                    planet: graha,
                    sign,
                    longitude,
                    bhava,
                },
                latitude_deg: 0.0,
                declination_deg: None,
            });
        }
    }

    let moment = kala_moment(
        request.jd_ut,
        request.latitude_deg,
        request.longitude_deg,
        request.elevation_m,
        request.tz_offset_minutes,
    )?;
    let jd_tt = delta_t::ut1_to_tt(request.jd_ut);
    let cusps = AngularCusps::from_twelve(&chart.houses.cusps);
    let context = ShadbalaContext {
        jd_tt,
        ayanamsha_deg: request.ayanamsha_deg,
        moment,
        cusps,
    };
    let strengths = compute_shadbala_complete(&planets, &context, options)?;

    let mut out = Vec::with_capacity(strengths.len());
    for (strength, (name, graha)) in strengths.into_iter().zip(CHART_NAMES) {
        let p = body(name)?;
        let mean = mean_geocentric_motion(graha, jd_tt);
        let state = cheshta_at(graha, p.longitude, p.speed, mean);
        out.push(GrahaShadbala {
            strength,
            motion: MotionState {
                state,
                virupas: state.virupas(),
                speed_deg_per_day: p.speed,
                mean_speed_deg_per_day: mean,
            },
        });
    }
    Ok(ChartShadbala {
        moment,
        cusps,
        planets: out,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use vedaksha_astro::chart::ChartPlanet;
    use vedaksha_astro::houses::{HouseCusps, HouseSystem};

    // Chennai, 13.08 N 80.27 E, clock UTC+5:30. JD 2461338.0 is noon UT on the
    // civil day whose Julian Day Number is 2461338.
    const LAT: f64 = 13.08;
    const LON: f64 = 80.27;
    const NOON_UT: f64 = 2_461_338.0;
    const IST: i32 = 330;

    #[test]
    fn the_moment_brackets_the_birth_and_labels_the_civil_day_of_the_sunrise() {
        // Noon UT is 17:30 on the clock, the same civil day as its sunrise.
        let m = kala_moment(NOON_UT, LAT, LON, 0.0, IST).expect("a mid-latitude day");
        assert!(m.jd_sunrise <= m.jd && m.jd < m.jd_next_sunrise);
        assert!(m.jd_sunrise < m.jd_sunset && m.jd_sunset < m.jd_next_sunrise);
        assert_eq!(m.hindu_day_jdn, 2_461_338);
        // A day in the tropics runs about 24 hours from sunrise to sunrise and
        // about 12 from sunrise to sunset.
        assert!((m.jd_next_sunrise - m.jd_sunrise - 1.0).abs() < 0.01);
        assert!((m.jd_sunset - m.jd_sunrise - 0.5).abs() < 0.03);

        // 03:00 on the clock is 21:30 UT the evening before: before that
        // day's sunrise, so the Hindu day is the previous one.
        let before_sunrise = 2_461_336.5 + 21.5 / 24.0;
        let m = kala_moment(before_sunrise, LAT, LON, 0.0, IST).expect("a mid-latitude day");
        assert!(m.jd_sunrise <= before_sunrise && before_sunrise < m.jd_next_sunrise);
        assert_eq!(m.hindu_day_jdn, 2_461_337);
        // ... and that sunrise is itself the previous one, not a day early.
        assert!(before_sunrise - m.jd_sunrise < 1.0);
    }

    #[test]
    fn the_apparent_time_is_twelve_at_the_suns_own_transit_and_follows_longitude() {
        let eq = |jd: f64| sun_equatorial_deg(&AnalyticalProvider, jd);
        let transit = sun_rise_set(NOON_UT - 0.5, LAT, LON, 0.0, &eq)
            .transit
            .expect("the Sun transits");
        let at = |jd: f64, lon: f64| {
            kala_moment(jd, LAT, lon, 0.0, IST)
                .expect("a mid-latitude day")
                .apparent_hours_from_midnight
        };
        // Within 20 seconds of noon at the engine's own meridian transit.
        assert!(
            (at(transit, LON) - 12.0).abs() < 20.0 / 3600.0,
            "{}",
            at(transit, LON)
        );
        // Fifteen degrees east is an hour later, whatever the date.
        let dh = (at(transit, LON + 15.0) - at(transit, LON)).rem_euclid(24.0);
        assert!((dh - 1.0).abs() < 1e-9, "{dh}");
        // A quarter of a day on, the Sun has gone round by about six hours.
        let later = (at(transit + 0.25, LON) - at(transit, LON)).rem_euclid(24.0);
        assert!((later - 6.0).abs() < 0.02, "{later}");
    }

    #[test]
    fn the_polar_day_has_no_hindu_day_to_divide() {
        // Ny-Alesund under the midnight sun, 2020-06-15 12:00 UT.
        let err = kala_moment(2_459_016.0, 78.22, 15.65, 0.0, 60).unwrap_err();
        assert!(matches!(err, ChartShadbalaError::NoSunEvent(_)), "{err}");
        assert!(err.to_string().contains("polar"));
    }

    fn body(name: &str, longitude: f64, speed: f64, house: u8) -> ChartPlanet {
        ChartPlanet {
            name: name.to_string(),
            longitude,
            latitude: 0.0,
            distance: 1.0,
            speed,
            retrograde: speed < 0.0,
            sign: String::new(),
            sign_index: (longitude / 30.0) as u8,
            house,
            dignity: String::new(),
        }
    }

    /// Sun 126, Moon 32, Mars 5.5, Mercury 62, Jupiter 245, Venus 100, Saturn
    /// 190 (the chart the Drik Bala vectors of [`crate::shadbala`] are worked
    /// on), and four cusps no two a half-circle apart.
    fn chart(with_rahu_at: Option<f64>) -> ComputedChart {
        let mut planets = vec![
            body("Sun", 126.0, 0.95, 10),
            body("Moon", 32.0, 13.2, 2),
            body("Mars", 5.5, 0.5, 1),
            body("Mercury", 62.0, 1.2, 12),
            body("Jupiter", 245.0, 0.1, 9),
            body("Venus", 100.0, 1.1, 4),
            body("Saturn", 190.0, -0.05, 7),
        ];
        if let Some(lon) = with_rahu_at {
            planets.push(body(RAHU_BODY_NAME, lon, -0.05, 12));
        }
        let mut cusps = [0.0; 12];
        for (i, c) in cusps.iter_mut().enumerate() {
            *c = (15.0 + 30.0 * i as f64) % 360.0;
        }
        cusps[3] = 108.0;
        cusps[6] = 195.0;
        cusps[9] = 287.0;
        ComputedChart {
            planets,
            houses: HouseCusps {
                cusps,
                asc: cusps[0],
                mc: cusps[9],
                system: HouseSystem::Placidus,
                polar_fallback: false,
            },
            aspects: Vec::new(),
            config_summary: String::new(),
        }
    }

    fn request(chart: &ComputedChart) -> ChartShadbalaRequest<'_> {
        ChartShadbalaRequest {
            chart,
            jd_ut: NOON_UT,
            latitude_deg: LAT,
            longitude_deg: LON,
            elevation_m: 0.0,
            tz_offset_minutes: IST,
            ayanamsha_deg: 24.2,
        }
    }

    fn of(result: &ChartShadbala, graha: Graha) -> GrahaShadbala {
        *result
            .planets
            .iter()
            .find(|p| p.strength.planet == graha)
            .expect("graha")
    }

    #[test]
    fn the_chart_feeds_the_house_the_cusps_and_the_motion() {
        let c = chart(None);
        let r = shadbala_for_chart(&request(&c), &ShadbalaOptions::default()).expect("complete");
        let order: Vec<Graha> = r.planets.iter().map(|p| p.strength.planet).collect();
        assert_eq!(
            order,
            [
                Graha::Sun,
                Graha::Moon,
                Graha::Mars,
                Graha::Mercury,
                Graha::Jupiter,
                Graha::Venus,
                Graha::Saturn
            ]
        );
        // Kendradi bala follows the chart's own house: Sun in the 10th (60),
        // Moon in the 2nd (30), Mercury in the 12th (15).
        assert!((of(&r, Graha::Sun).strength.sthana.kendradi - 60.0).abs() < 1e-12);
        assert!((of(&r, Graha::Moon).strength.sthana.kendradi - 30.0).abs() < 1e-12);
        assert!((of(&r, Graha::Mercury).strength.sthana.kendradi - 15.0).abs() < 1e-12);
        // Dig bala is measured from the chart's cusps: the Sun (126) from the
        // fourth cusp (108) is 18 degrees, 6 virupas; Saturn (190) from the
        // first (15) is 175, 58.33; the Moon (32) from the tenth (287) is 105,
        // 35; Mercury (62) from the seventh (195) is 133, 44.33.
        assert!((of(&r, Graha::Sun).strength.dig_bala - 6.0).abs() < 1e-9);
        assert!((of(&r, Graha::Saturn).strength.dig_bala - 175.0 / 3.0).abs() < 1e-9);
        assert!((of(&r, Graha::Moon).strength.dig_bala - 35.0).abs() < 1e-9);
        assert!((of(&r, Graha::Mercury).strength.dig_bala - 133.0 / 3.0).abs() < 1e-9);
        assert_eq!(r.cusps, AngularCusps::from_twelve(&c.houses.cusps));

        // Saturn at -0.05 degrees a day, ten degrees into Libra: clearly
        // retrograde (more than a minute of arc a day), not at a boundary, so
        // vakra at 60 virupas. The Moon at 13.2 against the mean 13.176 is at
        // the mean pace, sama at 30.
        let saturn = of(&r, Graha::Saturn).motion;
        assert_eq!(saturn.state, Cheshta::Vakra);
        assert!((saturn.virupas - 60.0).abs() < 1e-12);
        assert!((saturn.speed_deg_per_day + 0.05).abs() < 1e-12);
        let moon = of(&r, Graha::Moon).motion;
        assert_eq!(moon.state, Cheshta::Sama);
        assert!((moon.mean_speed_deg_per_day - 13.1764).abs() < 1e-3);
        // Mercury's mean pace is the mean Sun's, not his own orbit's.
        let sun_mean = of(&r, Graha::Sun).motion.mean_speed_deg_per_day;
        assert!((of(&r, Graha::Mercury).motion.mean_speed_deg_per_day - sun_mean).abs() < 1e-12);

        // The motion is reported beside the total and does not enter it.
        for p in &r.planets {
            let s = &p.strength;
            let sum = s.sthana.total
                + s.dig_bala
                + s.kala.total
                + s.cheshta_bala
                + s.naisargika_bala
                + s.drik_bala;
            assert!((s.total - sum).abs() < 1e-9, "{:?}", s.planet);
        }
    }

    #[test]
    fn rahu_in_mercurys_rashi_makes_him_malefic_company_and_gets_no_strength() {
        let without = shadbala_for_chart(&request(&chart(None)), &ShadbalaOptions::default())
            .expect("complete");
        // Rahu at 65 degrees shares Gemini with Mercury (62). On Venus (100)
        // Mercury's aspect is x = 38, c = 1, d = 8: generic 4. Benefic: quarter
        // +1 and the whole +4 = 5. Malefic: -1 + 4 = 3. A difference of -2.
        let with = shadbala_for_chart(&request(&chart(Some(65.0))), &ShadbalaOptions::default())
            .expect("complete");
        assert_eq!(with.planets.len(), 7, "no strength for the nodes");
        let delta = of(&with, Graha::Venus).strength.drik_bala
            - of(&without, Graha::Venus).strength.drik_bala;
        assert!((delta + 2.0).abs() < 1e-9, "{delta}");
        // Nothing else of Venus moves.
        assert!(
            (of(&with, Graha::Venus).strength.sthana.total
                - of(&without, Graha::Venus).strength.sthana.total)
                .abs()
                < 1e-12
        );
        // Rahu in another rashi (Gemini is 60 to 90; this is 200) changes
        // nothing at all.
        let far = shadbala_for_chart(&request(&chart(Some(200.0))), &ShadbalaOptions::default())
            .expect("complete");
        assert!(
            (of(&far, Graha::Venus).strength.drik_bala
                - of(&without, Graha::Venus).strength.drik_bala)
                .abs()
                < 1e-12
        );
    }

    #[test]
    fn a_chart_without_a_graha_or_in_the_polar_day_is_refused() {
        let mut c = chart(None);
        c.planets.retain(|p| p.name != "Venus");
        assert_eq!(
            shadbala_for_chart(&request(&c), &ShadbalaOptions::default()).unwrap_err(),
            ChartShadbalaError::MissingBody("Venus")
        );
        assert!(
            ChartShadbalaError::MissingBody("Venus")
                .to_string()
                .contains("Venus")
        );

        let c = chart(None);
        let mut polar = request(&c);
        polar.latitude_deg = 78.22;
        polar.longitude_deg = 15.65;
        polar.jd_ut = 2_459_016.0;
        assert!(matches!(
            shadbala_for_chart(&polar, &ShadbalaOptions::default()).unwrap_err(),
            ChartShadbalaError::NoSunEvent(_)
        ));
    }
}
