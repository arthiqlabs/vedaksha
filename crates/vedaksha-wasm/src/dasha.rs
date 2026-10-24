// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1
// Contact: info@arthiq.net | https://vedaksha.net

//! The wasm `compute_dasha` export: the arguments of the MCP `compute_dasha`
//! tool, key for key, and its payload.
//!
//! This crate cannot depend on `vedaksha-mcp` (it is a dev-dependency only, for
//! the parity tests), so the tool's validation and dispatch are written out
//! here over the same `vedaksha_vedic::dasha` functions, with the same defaults
//! and the same words for every refusal.
//! `tests/mcp_surface_parity.rs` holds the two to one payload for every system,
//! and the unit tests below hold them to one message for every refusal; change
//! the tool in `vedaksha-mcp/src/tools/compute_dasha.rs` and
//! `vedaksha-mcp/src/server.rs` and this file together.

use serde::Deserialize;

/// The Julian Day range `vedaksha-mcp` accepts (about 1800 to 2400 CE).
const JD_MIN: f64 = 2_378_496.5;
const JD_MAX: f64 = 2_597_641.5;

/// The most nested levels a Moon-based system may ask for.
const MAX_LEVELS: u8 = 5;

/// The most Yogini periods one response may carry. One round has
/// 8 + 8^2 + ... + 8^levels periods (584 at the default three levels, 37,448
/// at five), so this admits ten rounds at four levels and one at five.
const MAX_YOGINI_PERIODS: u64 = 50_000;

/// Natal signs of the seven classical grahas and Rahu, 0 = Aries. `i32` so a
/// negative reaches the range check and is named, as on the MCP tool.
#[derive(Deserialize, Clone, Copy)]
#[serde(deny_unknown_fields)]
struct GrahaSigns {
    sun: Option<i32>,
    moon: Option<i32>,
    mars: Option<i32>,
    mercury: Option<i32>,
    jupiter: Option<i32>,
    venus: Option<i32>,
    saturn: Option<i32>,
    rahu: Option<i32>,
}

/// Sidereal longitudes of the seven classical grahas and Rahu.
#[derive(Deserialize, Clone, Copy)]
#[serde(deny_unknown_fields)]
struct GrahaLongitudes {
    sun: Option<f64>,
    moon: Option<f64>,
    mars: Option<f64>,
    mercury: Option<f64>,
    jupiter: Option<f64>,
    venus: Option<f64>,
    saturn: Option<f64>,
    rahu: Option<f64>,
}

/// The arguments of the MCP `compute_dasha` tool. Unknown top-level keys are
/// ignored, as the tool ignores them, so one input is accepted or refused alike
/// on both surfaces.
#[derive(Deserialize)]
struct DashaInput {
    system: Option<String>,
    birth_jd: f64,
    moon_longitude: Option<f64>,
    lagna_sign: Option<u8>,
    graha_signs: Option<GrahaSigns>,
    levels: Option<u8>,
    graha_longitudes: Option<GrahaLongitudes>,
    karaka_scheme: Option<String>,
    rounds: Option<u8>,
    until: Option<f64>,
    antardashas: Option<bool>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum System {
    Vimshottari,
    Ashtottari,
    Yogini,
    Chara,
    Narayana,
    Sthira,
    NiryanaShoola,
    Maheshwara,
}

impl System {
    fn is_longitude_based(self) -> bool {
        matches!(self, Self::Sthira | Self::NiryanaShoola | Self::Maheshwara)
    }

    fn is_moon_based(self) -> bool {
        matches!(self, Self::Vimshottari | Self::Ashtottari | Self::Yogini)
    }
}

/// A refusal in the words of the MCP tool's `INVALID_PARAMETER`.
fn invalid(param: &str, detail: &str) -> String {
    format!("Invalid parameter '{param}': {detail}")
}

/// Ignoring case and any `_`, `-` or space, as the tool does.
fn parse_system(s: &str) -> Result<System, String> {
    let normalized: String = s
        .chars()
        .filter(|c| !matches!(c, '_' | '-' | ' '))
        .map(|c| c.to_ascii_lowercase())
        .collect();
    match normalized.as_str() {
        "vimshottari" => Ok(System::Vimshottari),
        "ashtottari" => Ok(System::Ashtottari),
        "yogini" => Ok(System::Yogini),
        "chara" => Ok(System::Chara),
        "narayana" => Ok(System::Narayana),
        "sthira" => Ok(System::Sthira),
        "niryanashoola" => Ok(System::NiryanaShoola),
        "maheshwara" => Ok(System::Maheshwara),
        _ => Err(invalid(
            "system",
            "must be one of: Vimshottari, Ashtottari, Yogini, Chara, Narayana, Sthira, \
             NiryanaShoola, Maheshwara",
        )),
    }
}

/// The options that change which answer a system gives, checked against the
/// system they belong to (see the MCP tool's `validate_extensions`).
fn validate_extensions(input: &DashaInput, system: System) -> Result<(), String> {
    let only = |param: &str, systems: &str| invalid(param, &format!("applies to {systems} only"));
    if (input.rounds.is_some() || input.until.is_some()) && system != System::Yogini {
        let param = if input.rounds.is_some() {
            "rounds"
        } else {
            "until"
        };
        return Err(only(param, "Yogini"));
    }
    if input.antardashas == Some(true) && system != System::Chara {
        return Err(invalid(
            "antardashas",
            "applies to Chara only; Sthira, NiryanaShoola and Maheshwara always carry \
             theirs, and the other systems have levels",
        ));
    }
    if input.karaka_scheme.is_some() && !matches!(system, System::Sthira | System::Maheshwara) {
        return Err(only("karaka_scheme", "Sthira and Maheshwara"));
    }
    if let Some(scheme) = &input.karaka_scheme
        && !matches!(scheme.as_str(), "7" | "8")
    {
        return Err(invalid("karaka_scheme", "must be \"7\" or \"8\""));
    }
    if input.rounds.is_some() && input.until.is_some() {
        return Err(invalid(
            "rounds",
            "exclusive with until: give a number of rounds or a horizon, not both",
        ));
    }
    if let Some(rounds) = input.rounds {
        let max = vedaksha_vedic::dasha::yogini::MAX_YOGINI_ROUNDS;
        if rounds == 0 || rounds > max {
            return Err(invalid("rounds", &format!("must be between 1 and {max}")));
        }
    }
    if let Some(until) = input.until
        && !until.is_finite()
    {
        return Err(invalid("until", "must be a finite Julian Day"));
    }
    if system == System::Yogini {
        let levels = u32::from(input.levels.unwrap_or(3).clamp(1, MAX_LEVELS));
        let per_round: u64 = (1..=levels).map(|l| 8_u64.pow(l)).sum();
        let max = u64::from(vedaksha_vedic::dasha::yogini::MAX_YOGINI_ROUNDS);
        // `until` stops at the first round ending at or past it; a round is
        // 36 years, and the first round is the shortest, so this counts high.
        let rounds = match (input.rounds, input.until) {
            (Some(r), _) => u64::from(r),
            (None, Some(until)) => {
                let years = (until - input.birth_jd) / (36.0 * 365.25);
                if years > 0.0 {
                    (years.ceil() as u64 + 1).min(max)
                } else {
                    1
                }
            }
            (None, None) => 1,
        };
        if per_round * rounds > MAX_YOGINI_PERIODS {
            return Err(invalid(
                if input.rounds.is_some() {
                    "rounds"
                } else {
                    "until"
                },
                &format!(
                    "{rounds} rounds at {levels} levels is {} periods, over the \
                     {MAX_YOGINI_PERIODS} one response may carry: ask for fewer levels or \
                     rounds",
                    per_round * rounds
                ),
            ));
        }
    }
    Ok(())
}

/// Resolve the system and check every field (see the MCP tool's `validate`).
fn validate(input: &DashaInput) -> Result<System, String> {
    let system = match &input.system {
        Some(s) => parse_system(s)?,
        None => System::Vimshottari,
    };

    if !input.birth_jd.is_finite() || !(JD_MIN..=JD_MAX).contains(&input.birth_jd) {
        return Err(format!(
            "Julian Day {} is outside valid range [{JD_MIN}, {JD_MAX}]",
            input.birth_jd
        ));
    }
    validate_extensions(input, system)?;

    if system.is_longitude_based() {
        let sign = input.lagna_sign.ok_or_else(|| {
            invalid(
                "lagna_sign",
                "required for Sthira, NiryanaShoola and Maheshwara",
            )
        })?;
        if !(0..=11).contains(&sign) {
            return Err(invalid("lagna_sign", "must be an integer in [0, 11]"));
        }
        let l = input.graha_longitudes.ok_or_else(|| {
            invalid(
                "graha_longitudes",
                "required for Sthira, NiryanaShoola and Maheshwara",
            )
        })?;
        for (name, value) in [
            ("graha_longitudes.sun", l.sun),
            ("graha_longitudes.moon", l.moon),
            ("graha_longitudes.mars", l.mars),
            ("graha_longitudes.mercury", l.mercury),
            ("graha_longitudes.jupiter", l.jupiter),
            ("graha_longitudes.venus", l.venus),
            ("graha_longitudes.saturn", l.saturn),
            ("graha_longitudes.rahu", l.rahu),
        ] {
            let value = value.ok_or_else(|| {
                invalid(
                    name,
                    "required when system is Sthira, NiryanaShoola or Maheshwara",
                )
            })?;
            if !value.is_finite() || !(0.0..360.0).contains(&value) {
                return Err(invalid(name, "must be a finite number in [0, 360)"));
            }
        }
        // Signs and longitudes could disagree, and one would be ignored.
        if input.graha_signs.is_some() {
            return Err(invalid(
                "graha_signs",
                "not used by Sthira, NiryanaShoola or Maheshwara; pass graha_longitudes",
            ));
        }
    } else if system.is_moon_based() {
        let moon = input.moon_longitude.ok_or_else(|| {
            invalid(
                "moon_longitude",
                "required for Vimshottari, Ashtottari, and Yogini",
            )
        })?;
        if !moon.is_finite() || !(0.0..360.0).contains(&moon) {
            return Err(invalid(
                "moon_longitude",
                "must be a finite number in [0, 360)",
            ));
        }
    } else {
        let sign = input
            .lagna_sign
            .ok_or_else(|| invalid("lagna_sign", "required for Chara and Narayana"))?;
        if !(0..=11).contains(&sign) {
            return Err(invalid("lagna_sign", "must be an integer in [0, 11]"));
        }
        let g = input
            .graha_signs
            .ok_or_else(|| invalid("graha_signs", "required for Chara and Narayana"))?;
        for (name, value) in [
            ("graha_signs.sun", g.sun),
            ("graha_signs.moon", g.moon),
            ("graha_signs.mars", g.mars),
            ("graha_signs.mercury", g.mercury),
            ("graha_signs.jupiter", g.jupiter),
            ("graha_signs.venus", g.venus),
            ("graha_signs.saturn", g.saturn),
            ("graha_signs.rahu", g.rahu),
        ] {
            let value =
                value.ok_or_else(|| invalid(name, "required when system is Chara or Narayana"))?;
            if !(0..=11).contains(&value) {
                return Err(invalid(name, "must be an integer in [0, 11]"));
            }
        }
    }

    if let Some(levels) = input.levels
        && (levels == 0 || levels > MAX_LEVELS)
    {
        return Err(invalid(
            "levels",
            &format!("must be between 1 and {MAX_LEVELS}"),
        ));
    }

    Ok(system)
}

/// Why there is no dasha, in words a caller can act on. The same words as the
/// MCP tool's.
const NO_BRAHMA: &str = "No Brahma graha in this chart. Jaimini 2.1.52 to 2.1.57 \
    (BPHS 46.170 to 173) name Brahma as the strong lord of the sixth, eighth or twelfth \
    rashi, in an odd rashi, behind the stronger of the lagna and its seventh, and the \
    sthira dasha starts from his rashi (2.3.4); in this chart the marks leave no graha. \
    The text names no other start. This is a property of the chart, not an error: it \
    is common, and a chart without a Brahma has no sthira dasha by the text.";

/// Sthira, Niryana Shoola and Maheshwara, with the defaults of the vedic
/// crate's dasha functions. They read the lagna and the grahas' longitudes.
fn jaimini_rashi_dasha(
    input: &DashaInput,
    system: System,
) -> Result<serde_json::Value, serde_json::Error> {
    use vedaksha_vedic::dasha::maheshwara::{
        MaheshwaraOptions, compute_maheshwara, maheshwara_graha,
    };
    use vedaksha_vedic::dasha::rashi_strength::{JaiminiStrength, NatalPlacements};
    use vedaksha_vedic::dasha::shoola::{ShoolaOptions, compute_niryana_shoola};
    use vedaksha_vedic::dasha::sthira::{SthiraOptions, compute_sthira};
    use vedaksha_vedic::karaka::{KarakaInput, KarakaScheme, compute_karakas};

    let l = input.graha_longitudes.expect("validated above");
    let get = |v: Option<f64>| v.expect("validated above");
    let longitudes = [
        get(l.sun),
        get(l.moon),
        get(l.mars),
        get(l.mercury),
        get(l.jupiter),
        get(l.venus),
        get(l.saturn),
        get(l.rahu),
    ];
    // Only the lagna's sign is read; the middle of the sign stands in for a
    // longitude the tool is not given.
    let lagna_sign = input.lagna_sign.expect("validated above");
    let placements =
        NatalPlacements::from_longitudes(f64::from(lagna_sign) * 30.0 + 15.0, longitudes);
    let strength = JaiminiStrength::new(&placements);

    let scheme = match input.karaka_scheme.as_deref() {
        Some("8") => KarakaScheme::Eight,
        _ => KarakaScheme::Seven,
    };
    let atmakaraka = compute_karakas(&KarakaInput {
        sun: longitudes[0],
        moon: longitudes[1],
        mars: longitudes[2],
        mercury: longitudes[3],
        jupiter: longitudes[4],
        venus: longitudes[5],
        saturn: longitudes[6],
        rahu: Some(longitudes[7]),
        scheme,
    })[0]
        .planet;

    match system {
        System::NiryanaShoola => serde_json::to_value(compute_niryana_shoola(
            &placements,
            input.birth_jd,
            &ShoolaOptions::default(),
            &strength,
        )),
        System::Sthira => {
            match compute_sthira(
                &placements,
                atmakaraka,
                input.birth_jd,
                &SthiraOptions::default(),
                &strength,
            ) {
                Some(dasha) => {
                    let mut v = serde_json::to_value(dasha)?;
                    v["no_brahma"] = serde_json::Value::Bool(false);
                    Ok(v)
                }
                None => Ok(serde_json::json!({
                    "no_brahma": true,
                    "message": NO_BRAHMA,
                    "anchor_sign": null,
                    "brahma": null,
                    "direction": null,
                    "periods": [],
                })),
            }
        }
        System::Maheshwara => {
            let options = MaheshwaraOptions::default();
            match compute_maheshwara(&placements, atmakaraka, input.birth_jd, &options, &strength) {
                Some(dasha) => {
                    let mut v = serde_json::to_value(dasha)?;
                    v["no_brahma"] = serde_json::Value::Bool(false);
                    Ok(v)
                }
                // The Maheshwara graha does not depend on Brahma; only the
                // sthira dasha its life span is read from does.
                None => {
                    let graha = maheshwara_graha(&placements, atmakaraka, &strength, options.rules);
                    Ok(serde_json::json!({
                        "no_brahma": true,
                        "message": NO_BRAHMA,
                        "maheshwara": graha,
                        "rashi": placements.sign_of(graha),
                    }))
                }
            }
        }
        _ => unreachable!("compute_dasha_inner sends only the Jaimini rashi dashas here"),
    }
}

fn graha_signs_of(input: &DashaInput) -> vedaksha_vedic::dasha::chara::GrahaSigns {
    let g = input.graha_signs.expect("validated above");
    // `validate` bounds every field to [0, 11], so each cast is exact.
    let sign = |v: Option<i32>| -> u8 {
        u8::try_from(v.expect("validated above")).expect("validated above")
    };
    vedaksha_vedic::dasha::chara::GrahaSigns {
        sun: sign(g.sun),
        moon: sign(g.moon),
        mars: sign(g.mars),
        mercury: sign(g.mercury),
        jupiter: sign(g.jupiter),
        venus: sign(g.venus),
        saturn: sign(g.saturn),
        rahu: sign(g.rahu),
    }
}

/// The dasha payload for the MCP tool's arguments, as a JSON string; the error
/// is the tool's message for the same input.
pub(crate) fn compute_dasha_inner(input_json: &str) -> Result<String, String> {
    use vedaksha_vedic::dasha;

    let args: serde_json::Value =
        serde_json::from_str(input_json).map_err(|e| format!("invalid input: {e}"))?;
    let input: DashaInput =
        serde_json::from_value(args).map_err(|e| invalid("arguments", &e.to_string()))?;
    let system = validate(&input)?;

    let levels = input.levels.unwrap_or(3).clamp(1, 5);
    let result = match system {
        System::Vimshottari => serde_json::to_value(dasha::vimshottari::compute_vimshottari(
            input.moon_longitude.expect("validated above"),
            input.birth_jd,
            levels,
        )),
        System::Ashtottari => serde_json::to_value(dasha::ashtottari::compute_ashtottari(
            input.moon_longitude.expect("validated above"),
            input.birth_jd,
            levels,
        )),
        System::Yogini => {
            let moon = input.moon_longitude.expect("validated above");
            // `rounds` and `until` are exclusive (validated above); with
            // neither, the first round only.
            let tree = match (input.rounds, input.until) {
                (Some(rounds), _) => {
                    dasha::yogini::compute_yogini_rounds(moon, input.birth_jd, levels, rounds)
                }
                (None, Some(until)) => {
                    dasha::yogini::compute_yogini_until(moon, input.birth_jd, levels, until)
                }
                (None, None) => dasha::yogini::compute_yogini(moon, input.birth_jd, levels),
            };
            serde_json::to_value(tree)
        }
        System::Chara => {
            let lagna = input.lagna_sign.expect("validated above");
            let positions = graha_signs_of(&input);
            // With `antardashas` each period carries its twelve (BPHS 51.5 to
            // 51.9, the library's defaults); the twelve mahadashas are bit for
            // bit the same either way.
            let periods = if input.antardashas == Some(true) {
                serde_json::to_value(dasha::chara::compute_chara_with_antars(
                    lagna,
                    input.birth_jd,
                    positions,
                ))
            } else {
                serde_json::to_value(dasha::chara::compute_chara(
                    lagna,
                    input.birth_jd,
                    positions,
                ))
            }
            .map_err(|e| format!("Computation failed: {e}"))?;
            Ok(serde_json::json!({ "lagna_sign": lagna, "periods": periods }))
        }
        System::Narayana => serde_json::to_value(dasha::narayana::compute_narayana(
            input.lagna_sign.expect("validated above"),
            input.birth_jd,
            graha_signs_of(&input),
        )),
        System::Sthira | System::NiryanaShoola | System::Maheshwara => {
            jaimini_rashi_dasha(&input, system)
        }
    };

    let value = result.map_err(|e| format!("Computation failed: {e}"))?;
    serde_json::to_string(&value).map_err(|e| format!("Computation failed: {e}"))
}

#[cfg(test)]
mod tests {
    use super::compute_dasha_inner;
    use serde_json::{Value, json};

    const BIRTH: f64 = 2_451_545.0;

    fn signs() -> Value {
        json!({ "sun": 9, "moon": 2, "mars": 1, "mercury": 10,
                "jupiter": 10, "venus": 10, "saturn": 4, "rahu": 2 })
    }

    fn longitudes() -> Value {
        json!({ "sun": 29.0, "moon": 95.0, "mars": 100.0, "mercury": 38.0,
                "jupiter": 252.0, "venus": 63.0, "saturn": 124.0, "rahu": 350.0 })
    }

    /// The MCP tool's answer to the same arguments: its payload, or its
    /// refusal message.
    fn mcp(args: &Value) -> Result<Value, String> {
        let server = vedaksha_mcp::server::McpServer::new();
        let request = json!({
            "jsonrpc": "2.0", "id": 1, "method": "tools/call",
            "params": { "name": "compute_dasha", "arguments": args }
        });
        let response: Value =
            serde_json::from_str(&server.handle_request(&request.to_string())).unwrap();
        match response.get("error") {
            Some(e) => Err(e["message"].as_str().unwrap().to_string()),
            None => Ok(serde_json::from_str(
                response["result"]["content"][0]["text"].as_str().unwrap(),
            )
            .unwrap()),
        }
    }

    fn with(base: Value, edits: &[(&str, Value)]) -> Value {
        let mut v = base;
        for (k, x) in edits {
            if x.is_null() {
                v.as_object_mut().unwrap().remove(*k);
            } else {
                v[*k] = x.clone();
            }
        }
        v
    }

    /// Every way the tool refuses, and a few inputs it accepts, put to both
    /// surfaces: the same verdict and, for a refusal, the same message. A
    /// check that a refusal exists is not enough, since the point is that a
    /// caller moving between the surfaces is told the same thing.
    #[test]
    fn refuses_exactly_what_the_mcp_tool_refuses_in_its_words() {
        let moon = json!({ "moon_longitude": 123.45, "birth_jd": BIRTH });
        let chara = json!({ "system": "Chara", "lagna_sign": 10, "birth_jd": BIRTH,
                            "graha_signs": signs() });
        let jaimini = json!({ "system": "Sthira", "lagna_sign": 0, "birth_jd": BIRTH,
                              "graha_longitudes": longitudes() });
        let yogini = with(moon.clone(), &[("system", "Yogini".into())]);
        let mut cases: Vec<(&str, Value)> = vec![
            ("default accepted", moon.clone()),
            (
                "unknown system",
                with(moon.clone(), &[("system", "Nope".into())]),
            ),
            (
                "jd below range",
                with(moon.clone(), &[("birth_jd", 1.0.into())]),
            ),
            (
                "jd above range",
                with(moon.clone(), &[("birth_jd", 3.0e6.into())]),
            ),
            (
                "moon missing",
                with(moon.clone(), &[("moon_longitude", Value::Null)]),
            ),
            (
                "moon 360",
                with(moon.clone(), &[("moon_longitude", 360.0.into())]),
            ),
            (
                "moon negative",
                with(moon.clone(), &[("moon_longitude", (-1.0).into())]),
            ),
            ("levels 0", with(moon.clone(), &[("levels", 0.into())])),
            ("levels 6", with(moon.clone(), &[("levels", 6.into())])),
            (
                "levels 5 accepted",
                with(moon.clone(), &[("levels", 5.into())]),
            ),
            (
                "rounds on vimshottari",
                with(moon.clone(), &[("rounds", 2.into())]),
            ),
            (
                "until on vimshottari",
                with(moon.clone(), &[("until", 2.46e6.into())]),
            ),
            (
                "antardashas on vimshottari",
                with(moon.clone(), &[("antardashas", true.into())]),
            ),
            (
                "antardashas false accepted",
                with(moon.clone(), &[("antardashas", false.into())]),
            ),
            (
                "karaka_scheme on vimshottari",
                with(moon.clone(), &[("karaka_scheme", "7".into())]),
            ),
            (
                "yogini rounds 2",
                with(yogini.clone(), &[("rounds", 2.into())]),
            ),
            (
                "yogini rounds 0",
                with(yogini.clone(), &[("rounds", 0.into())]),
            ),
            (
                "yogini rounds 11",
                with(yogini.clone(), &[("rounds", 11.into())]),
            ),
            (
                "yogini rounds and until",
                with(
                    yogini.clone(),
                    &[("rounds", 2.into()), ("until", 2.46e6.into())],
                ),
            ),
            (
                "yogini rounds over the period cap",
                with(
                    yogini.clone(),
                    &[("rounds", 2.into()), ("levels", 5.into())],
                ),
            ),
            (
                "yogini rounds at the cap accepted",
                with(
                    yogini.clone(),
                    &[("rounds", 10.into()), ("levels", 4.into())],
                ),
            ),
            (
                "yogini until over the period cap",
                with(
                    yogini.clone(),
                    &[
                        ("until", (BIRTH + 40.0 * 365.25).into()),
                        ("levels", 5.into()),
                    ],
                ),
            ),
            ("chara accepted", chara.clone()),
            (
                "chara lagna 12",
                with(chara.clone(), &[("lagna_sign", 12.into())]),
            ),
            (
                "chara lagna missing",
                with(chara.clone(), &[("lagna_sign", Value::Null)]),
            ),
            (
                "chara signs missing",
                with(chara.clone(), &[("graha_signs", Value::Null)]),
            ),
            (
                "chara rahu missing",
                with(
                    chara.clone(),
                    &[("graha_signs", with(signs(), &[("rahu", Value::Null)]))],
                ),
            ),
            (
                "chara mars 12",
                with(
                    chara.clone(),
                    &[("graha_signs", with(signs(), &[("mars", 12.into())]))],
                ),
            ),
            (
                "chara mars negative",
                with(
                    chara.clone(),
                    &[("graha_signs", with(signs(), &[("mars", (-1).into())]))],
                ),
            ),
            (
                "chara ketu supplied",
                with(
                    chara.clone(),
                    &[("graha_signs", with(signs(), &[("ketu", 8.into())]))],
                ),
            ),
            (
                "chara antardashas accepted",
                with(chara.clone(), &[("antardashas", true.into())]),
            ),
            (
                "narayana antardashas",
                with(
                    chara.clone(),
                    &[("system", "Narayana".into()), ("antardashas", true.into())],
                ),
            ),
            ("sthira accepted", jaimini.clone()),
            (
                "sthira karaka 8 accepted",
                with(jaimini.clone(), &[("karaka_scheme", "8".into())]),
            ),
            (
                "sthira karaka 9",
                with(jaimini.clone(), &[("karaka_scheme", "9".into())]),
            ),
            (
                "sthira lagna missing",
                with(jaimini.clone(), &[("lagna_sign", Value::Null)]),
            ),
            (
                "sthira longitudes missing",
                with(jaimini.clone(), &[("graha_longitudes", Value::Null)]),
            ),
            (
                "sthira venus 360",
                with(
                    jaimini.clone(),
                    &[(
                        "graha_longitudes",
                        with(longitudes(), &[("venus", 360.0.into())]),
                    )],
                ),
            ),
            (
                "sthira venus missing",
                with(
                    jaimini.clone(),
                    &[(
                        "graha_longitudes",
                        with(longitudes(), &[("venus", Value::Null)]),
                    )],
                ),
            ),
            (
                "sthira signs and longitudes both",
                with(jaimini.clone(), &[("graha_signs", signs())]),
            ),
            (
                "moon-based with a stray graha_signs",
                with(moon.clone(), &[("graha_signs", signs())]),
            ),
            ("birth_jd missing", json!({ "moon_longitude": 1.0 })),
            (
                "wrong type",
                with(moon.clone(), &[("levels", "three".into())]),
            ),
            (
                "unknown top-level key ignored",
                with(moon.clone(), &[("extra", 1.into())]),
            ),
        ];
        for system in ["Ashtottari", "Maheshwara", "NiryanaShoola"] {
            cases.push((
                "other system accepted",
                if system == "Ashtottari" {
                    with(moon.clone(), &[("system", system.into())])
                } else {
                    with(jaimini.clone(), &[("system", system.into())])
                },
            ));
        }

        let mut refused = 0;
        for (label, args) in &cases {
            let wasm = compute_dasha_inner(&args.to_string());
            match (mcp(args), wasm) {
                (Ok(_), Ok(_)) => {}
                (Err(m), Err(w)) => {
                    refused += 1;
                    assert_eq!(m, w, "[{label}] the two surfaces refuse in different words");
                }
                (m, w) => panic!(
                    "[{label}] verdicts differ: mcp {:?}, wasm {:?}",
                    m.map(|_| "ok"),
                    w.map(|_| "ok")
                ),
            }
        }
        // The matrix must have teeth: most of its cases are refusals.
        assert!(refused >= 30, "only {refused} refusals exercised");
    }

    #[test]
    fn the_earlier_positional_arguments_are_not_a_valid_input() {
        // What the export took before: it was three numbers, not an object.
        assert!(compute_dasha_inner("100.0").is_err());
        assert!(
            compute_dasha_inner("not json")
                .unwrap_err()
                .contains("invalid input")
        );
        // `levels` was clamped to 1..=5; it is now refused outside it.
        let e = compute_dasha_inner(
            &json!({ "moon_longitude": 100.0, "birth_jd": BIRTH, "levels": 9 }).to_string(),
        )
        .unwrap_err();
        assert!(e.contains("levels"), "{e}");
    }
}
