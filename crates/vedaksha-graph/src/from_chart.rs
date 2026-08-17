// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Build a [`ChartGraph`] from a computed chart.
//!
//! **Why this module exists.** Until v5.0.1 this crate could emit a graph in
//! five formats and had no way to *produce* one. `ChartGraph::new` returns an
//! empty graph and the only other constructions in the repository were test
//! fixtures, so every emitter was verified against hand-assembled input that no
//! caller could obtain from a computation. The README promised "every chart is
//! a property graph"; the MCP `emit_graph` schema promised "`ChartGraph` JSON as
//! returned by `compute_natal_chart`". Feeding it exactly that returned
//! `missing field 'nodes'`. This module is the missing half.
//!
//! **What it maps.** A [`ComputedChart`] carries planets, house cusps and
//! aspects, so this produces four of the ontology's nine node types and seven
//! of its twelve edge types:
//!
//! | Produced | Not produced |
//! |---|---|
//! | `Chart`, `Planet`, `Sign`, `House` | `Nakshatra`, `Pada`, `Pattern`, `DashaPeriod`, `FixedStar` |
//! | `BelongsTo`, `PlacedIn`, `Occupies`, `Aspects`, `Rules`, `CuspOf`, `Disposits` | `InNakshatra`, `PartOfPattern`, `DashaLord`, `ContainsPeriod`, `ConjunctStar` |
//!
//! The absent ones need data a `ComputedChart` does not carry — nakshatra
//! divisions, dasha periods, star catalogues. They are listed here rather than
//! left for a reader to discover by finding an empty node set.

use crate::chart_graph::ChartGraph;
use crate::classification::DataClassification;
use crate::ids::NodeId;
use crate::ontology::{Edge, EdgeProperties, EdgeType, Node, NodeProperties, NodeType};
use vedaksha_astro::chart::ComputedChart;
use vedaksha_astro::dignity::{DignityPlanet, RulershipScheme, Sign, domicile_ruler};

/// The four classical elements, indexed by `sign_index % 4`.
///
/// Aries is Fire, Taurus Earth, Gemini Air, Cancer Water, and the cycle repeats
/// — this is the definition of the elemental assignment, not a source-dependent
/// claim.
const ELEMENTS: [&str; 4] = ["Fire", "Earth", "Air", "Water"];

/// The three modalities, indexed by `sign_index % 3`: Aries cardinal, Taurus
/// fixed, Gemini mutable, repeating.
const MODALITIES: [&str; 3] = ["Cardinal", "Fixed", "Mutable"];

/// The display name a `DignityPlanet` carries in a chart, so a ruler can be
/// matched back to a planet node.
fn dignity_planet_name(planet: DignityPlanet) -> &'static str {
    match planet {
        DignityPlanet::Sun => "Sun",
        DignityPlanet::Moon => "Moon",
        DignityPlanet::Mercury => "Mercury",
        DignityPlanet::Venus => "Venus",
        DignityPlanet::Mars => "Mars",
        DignityPlanet::Jupiter => "Jupiter",
        DignityPlanet::Saturn => "Saturn",
        DignityPlanet::Uranus => "Uranus",
        DignityPlanet::Neptune => "Neptune",
        DignityPlanet::Pluto => "Pluto",
    }
}

/// Stable, lower-cased node key for a planet name (`"True Node"` → `"true_node"`).
fn planet_key(name: &str) -> String {
    name.to_lowercase().replace(' ', "_")
}

/// The global node ID for a sign. Global, not chart-scoped: a sign is the same
/// entity across every chart, which is what makes cross-chart queries possible.
fn sign_id(index: u8) -> NodeId {
    NodeId::global("sign", &Sign::from_index(index).name().to_lowercase())
}

/// Add all twelve signs.
///
/// All twelve, always — a sign with nothing in it is still a legitimate query
/// target ("which houses fall in Leo?"), and omitting empty signs would make the
/// graph's shape depend on the chart, which breaks comparison queries.
fn add_sign_nodes(graph: &mut ChartGraph) {
    for index in 0..12u8 {
        let sign = Sign::from_index(index);
        graph.add_node(Node {
            id: sign_id(index),
            node_type: NodeType::Sign,
            properties: NodeProperties::Sign {
                name: sign.name().to_string(),
                index,
                element: ELEMENTS[(index % 4) as usize].to_string(),
                modality: MODALITIES[(index % 3) as usize].to_string(),
            },
        });
    }
}

/// Add `Rules` (planet → sign it rules) and `Disposits` (ruler → occupant).
///
/// Emitted only for bodies actually present in this chart. A `Rules` edge from a
/// planet node that does not exist would leave a dangling reference that every
/// emitter would faithfully serialise, and that only fails at import time.
fn add_rulership_edges(
    graph: &mut ChartGraph,
    chart: &ComputedChart,
    chart_key: &str,
    scheme: RulershipScheme,
) {
    let planet_id = |name: &str| NodeId::chart_scoped(chart_key, "planet", &planet_key(name));
    // A Vec rather than a set: a chart holds ~10 bodies.
    let present: Vec<String> = chart.planets.iter().map(|p| planet_key(&p.name)).collect();
    let has = |name: &str| {
        let key = planet_key(name);
        present.contains(&key)
    };

    for index in 0..12u8 {
        let ruler = dignity_planet_name(domicile_ruler(Sign::from_index(index), scheme));
        if has(ruler) {
            graph.add_edge(Edge {
                edge_type: EdgeType::Rules,
                from: planet_id(ruler),
                to: sign_id(index),
                properties: EdgeProperties::None,
            });
        }
    }

    for planet in &chart.planets {
        let ruler =
            dignity_planet_name(domicile_ruler(Sign::from_index(planet.sign_index), scheme));
        // Skip the self-loop a planet in its own domicile would otherwise create.
        if has(ruler) && planet_key(ruler) != planet_key(&planet.name) {
            graph.add_edge(Edge {
                edge_type: EdgeType::Disposits,
                from: planet_id(ruler),
                to: planet_id(&planet.name),
                properties: EdgeProperties::None,
            });
        }
    }
}

/// Convert a computed chart into a queryable property graph.
///
/// `julian_day`, `latitude` and `longitude` are taken as arguments rather than
/// read off the chart because [`ComputedChart`] does not carry them — it is the
/// result of a computation whose inputs the caller still holds. They identify
/// the chart node and seed its ID hash.
///
/// `scheme` selects traditional or modern rulerships, which decides who rules
/// Scorpio, Aquarius and Pisces and therefore which `Rules` and `Disposits`
/// edges appear.
///
/// # Example
///
/// ```
/// # use vedaksha_graph::from_chart::chart_to_graph;
/// # use vedaksha_graph::classification::DataClassification;
/// # use vedaksha_astro::dignity::RulershipScheme;
/// # fn demo(chart: &vedaksha_astro::chart::ComputedChart) {
/// let graph = chart_to_graph(
///     chart,
///     2_451_545.0,
///     28.6,
///     77.2,
///     RulershipScheme::Traditional,
///     DataClassification::Anonymous,
/// );
/// assert!(graph.node_count() > 0);
/// # }
/// ```
#[must_use]
pub fn chart_to_graph(
    chart: &ComputedChart,
    julian_day: f64,
    latitude: f64,
    longitude: f64,
    scheme: RulershipScheme,
    classification: DataClassification,
) -> ChartGraph {
    // The config summary distinguishes two charts that share an instant and a
    // place but differ in house system or zodiac, so it is what varies the hash.
    let config_hash = NodeId::config_hash(&chart.config_summary);
    let chart_key = NodeId::chart_hash(julian_day, latitude, longitude, config_hash);
    let chart_id = NodeId(chart_key.clone());

    let mut graph = ChartGraph::new(chart_id.clone(), classification);

    graph.add_node(Node {
        id: chart_id.clone(),
        node_type: NodeType::Chart,
        properties: NodeProperties::Chart {
            julian_day,
            latitude,
            longitude,
            classification,
        },
    });

    add_sign_nodes(&mut graph);

    // ── Houses ───────────────────────────────────────────────────────────────
    let system = format!("{:?}", chart.houses.system);
    let house_id = |number: u8| NodeId::chart_scoped(&chart_key, "house", &number.to_string());
    for (i, cusp) in chart.houses.cusps.iter().enumerate() {
        // `cusps` is [f64; 12], so i < 12 and this cannot truncate. Written as
        // a saturating cast rather than an expect() so the function carries no
        // panic path at all.
        #[allow(clippy::cast_possible_truncation)]
        let number = (i + 1) as u8;
        graph.add_node(Node {
            id: house_id(number),
            node_type: NodeType::House,
            properties: NodeProperties::House {
                number,
                cusp_longitude: *cusp,
                system: system.clone(),
            },
        });
        graph.add_edge(Edge {
            edge_type: EdgeType::BelongsTo,
            from: house_id(number),
            to: chart_id.clone(),
            properties: EdgeProperties::None,
        });
        // Which sign the cusp itself falls in — the relationship that makes
        // "is the 7th cusp in a fixed sign?" answerable.
        graph.add_edge(Edge {
            edge_type: EdgeType::CuspOf,
            from: house_id(number),
            to: sign_id(vedaksha_astro::dignity::sign_index(*cusp)),
            properties: EdgeProperties::None,
        });
    }

    // ── Planets ──────────────────────────────────────────────────────────────
    let planet_id = |name: &str| NodeId::chart_scoped(&chart_key, "planet", &planet_key(name));
    for planet in &chart.planets {
        graph.add_node(Node {
            id: planet_id(&planet.name),
            node_type: NodeType::Planet,
            properties: NodeProperties::Planet {
                name: planet.name.clone(),
                longitude: planet.longitude,
                latitude: planet.latitude,
                distance: planet.distance,
                speed: planet.speed,
                retrograde: planet.retrograde,
                sign_index: planet.sign_index,
                house: planet.house,
            },
        });
        graph.add_edge(Edge {
            edge_type: EdgeType::BelongsTo,
            from: planet_id(&planet.name),
            to: chart_id.clone(),
            properties: EdgeProperties::None,
        });
        graph.add_edge(Edge {
            edge_type: EdgeType::PlacedIn,
            from: planet_id(&planet.name),
            to: sign_id(planet.sign_index),
            properties: EdgeProperties::None,
        });
        // `house` is 1-12 from the chart computation; a 0 would mean "not
        // determined" and must not become an edge to a house that does not exist.
        if (1..=12).contains(&planet.house) {
            graph.add_edge(Edge {
                edge_type: EdgeType::Occupies,
                from: planet_id(&planet.name),
                to: house_id(planet.house),
                properties: EdgeProperties::None,
            });
        }
    }

    add_rulership_edges(&mut graph, chart, &chart_key, scheme);

    // ── Aspects ──────────────────────────────────────────────────────────────
    // `body1_index` / `body2_index` index into `chart.planets`; a stale or
    // out-of-range index is dropped rather than panicking, because this
    // function is reachable from a deserialised chart supplied over MCP.
    for aspect in &chart.aspects {
        let (Some(a), Some(b)) = (
            chart.planets.get(aspect.body1_index),
            chart.planets.get(aspect.body2_index),
        ) else {
            continue;
        };
        graph.add_edge(Edge {
            edge_type: EdgeType::Aspects,
            from: planet_id(&a.name),
            to: planet_id(&b.name),
            properties: EdgeProperties::Aspect {
                aspect_type: format!("{:?}", aspect.aspect_type),
                orb: aspect.orb,
                applying: aspect.motion == vedaksha_astro::aspects::AspectMotion::Applying,
                strength: aspect.strength,
            },
        });
    }

    graph
}

#[cfg(test)]
mod tests {
    use super::*;
    use vedaksha_astro::chart::{ChartConfig, compute_chart};
    use vedaksha_astro::houses::HouseSystem;

    /// A chart with known placements, so the assertions below are about the
    /// mapping and not about the ephemeris. Sun at 15° Aries, Moon at 15°
    /// Cancer, Mars at 15° Capricorn — the Moon in its own domicile, which is
    /// the self-loop the builder must not emit.
    fn fixture() -> ComputedChart {
        let planets = vec![
            ("Sun".to_string(), 15.0, 0.0, 1.0, 0.98),
            ("Moon".to_string(), 105.0, 2.0, 0.002, 13.2),
            ("Mars".to_string(), 285.0, -1.0, 1.5, -0.2),
        ];
        compute_chart(
            &planets,
            0.0,
            28.6,
            23.44,
            2_451_545.0,
            &ChartConfig {
                house_system: HouseSystem::WholeSign,
                ..ChartConfig::default()
            },
        )
    }

    fn graph_of(chart: &ComputedChart) -> ChartGraph {
        chart_to_graph(
            chart,
            2_451_545.0,
            28.6,
            77.2,
            RulershipScheme::Traditional,
            DataClassification::Anonymous,
        )
    }

    /// The failure this module exists to fix: a computed chart must yield a
    /// populated graph, with every node type a chart can supply.
    #[test]
    fn a_computed_chart_becomes_a_populated_graph() {
        let graph = graph_of(&fixture());
        assert_eq!(graph.nodes_of_type(NodeType::Chart).len(), 1);
        assert_eq!(
            graph.nodes_of_type(NodeType::Sign).len(),
            12,
            "all twelve signs"
        );
        assert_eq!(graph.nodes_of_type(NodeType::House).len(), 12);
        assert_eq!(graph.nodes_of_type(NodeType::Planet).len(), 3);
    }

    /// Every edge must point at a node that exists. A dangling reference would
    /// be serialised faithfully by all five emitters and fail only at import.
    #[test]
    fn no_edge_dangles() {
        let graph = graph_of(&fixture());
        assert!(!graph.edges.is_empty(), "a chart with 3 planets has edges");
        for edge in &graph.edges {
            assert!(
                graph.find_node(&edge.from).is_some(),
                "edge {:?} starts at a node that does not exist: {:?}",
                edge.edge_type,
                edge.from
            );
            assert!(
                graph.find_node(&edge.to).is_some(),
                "edge {:?} points at a node that does not exist: {:?}",
                edge.edge_type,
                edge.to
            );
        }
    }

    /// Placement edges must agree with what the chart computed, not merely
    /// exist — this catches an off-by-one in the sign or house index.
    #[test]
    fn placements_match_the_computed_chart() {
        let chart = fixture();
        let graph = graph_of(&chart);
        for planet in &chart.planets {
            let pid = NodeId::chart_scoped(
                &graph.chart_id.0,
                "planet",
                &planet.name.to_lowercase().replace(' ', "_"),
            );
            let placed: Vec<_> = graph
                .edges_from(&pid)
                .into_iter()
                .filter(|e| e.edge_type == EdgeType::PlacedIn)
                .collect();
            assert_eq!(placed.len(), 1, "{} is in exactly one sign", planet.name);
            let expected = NodeId::global(
                "sign",
                &Sign::from_index(planet.sign_index).name().to_lowercase(),
            );
            assert_eq!(placed[0].to, expected, "{} sign edge", planet.name);
        }
    }

    /// The graph's vocabulary must be the chart's vocabulary. `ChartPlanet`
    /// carries a `sign` string ("Aries"); the sign nodes are built from
    /// `sign_index`. If those ever disagree, the graph would name a sign the
    /// chart never mentioned and a consumer would need a second vocabulary to
    /// join them.
    #[test]
    fn sign_names_in_the_graph_match_the_names_the_chart_emits() {
        let chart = fixture();
        let graph = graph_of(&chart);
        for planet in &chart.planets {
            let node = graph
                .find_node(&NodeId::global(
                    "sign",
                    &Sign::from_index(planet.sign_index).name().to_lowercase(),
                ))
                .expect("the sign node a planet is placed in must exist");
            let NodeProperties::Sign { name, index, .. } = &node.properties else {
                panic!("sign node carries sign properties");
            };
            assert_eq!(
                name, &planet.sign,
                "graph calls it {name}, the chart calls it {}",
                planet.sign
            );
            assert_eq!(*index, planet.sign_index);
        }
    }

    /// Planet node properties must carry the chart's own values, unrounded and
    /// unrenamed — the graph is a view of the chart, not a restatement of it.
    #[test]
    fn planet_properties_are_the_charts_own_values() {
        let chart = fixture();
        let graph = graph_of(&chart);
        for planet in &chart.planets {
            let node = graph
                .find_node(&NodeId::chart_scoped(
                    &graph.chart_id.0,
                    "planet",
                    &planet.name.to_lowercase(),
                ))
                .expect("planet node");
            let NodeProperties::Planet {
                name,
                longitude,
                house,
                retrograde,
                ..
            } = &node.properties
            else {
                panic!("planet node carries planet properties");
            };
            assert_eq!(name, &planet.name);
            assert!((longitude - planet.longitude).abs() < f64::EPSILON);
            assert_eq!(*house, planet.house);
            assert_eq!(*retrograde, planet.retrograde);
        }
    }

    /// A planet in its own domicile disposits nothing — the Moon in Cancer must
    /// not produce a self-loop.
    #[test]
    fn a_planet_in_its_own_sign_produces_no_self_loop() {
        let graph = graph_of(&fixture());
        for edge in &graph.edges {
            assert!(
                !(edge.edge_type == EdgeType::Disposits && edge.from == edge.to),
                "self-loop on {:?}",
                edge.from
            );
        }
    }

    /// Rulership edges are emitted only for planets present in the chart. The
    /// fixture has no Venus, so Taurus must have no incoming `Rules`.
    #[test]
    fn absent_rulers_produce_no_edges() {
        let graph = graph_of(&fixture());
        let taurus = NodeId::global("sign", "taurus");
        let rules: Vec<_> = graph
            .edges_to(&taurus)
            .into_iter()
            .filter(|e| e.edge_type == EdgeType::Rules)
            .collect();
        assert!(
            rules.is_empty(),
            "Venus is not in this chart, so it rules nothing here"
        );
    }

    /// Node IDs must be stable across runs: they end up in emitted Cypher that
    /// a caller may diff or re-import.
    #[test]
    fn the_same_chart_yields_the_same_ids() {
        let chart = fixture();
        let a = graph_of(&chart);
        let b = graph_of(&chart);
        assert_eq!(a.chart_id, b.chart_id);
        let ids_a: Vec<_> = a.nodes.iter().map(|n| n.id.clone()).collect();
        let ids_b: Vec<_> = b.nodes.iter().map(|n| n.id.clone()).collect();
        assert_eq!(ids_a, ids_b);
    }

    /// Two charts differing only in configuration must not collide, which is
    /// what the config hash in the chart ID is for.
    #[test]
    fn a_different_configuration_yields_a_different_chart_id() {
        let mut other = fixture();
        other.config_summary = "Houses: Placidus, Zodiac: Lahiri, Rulership: Traditional".into();
        assert_ne!(graph_of(&fixture()).chart_id, graph_of(&other).chart_id);
    }

    /// The end-to-end claim: a computed chart survives into every shipped
    /// emitter. This is what "every chart is a property graph you can query in
    /// Cypher, `SurrealQL`, or JSON-LD" actually asserts.
    #[cfg(feature = "emitters")]
    #[test]
    fn the_graph_emits_in_every_published_format() {
        use crate::emitters::GraphEmitter;
        let graph = graph_of(&fixture());
        let outputs = [
            (
                "cypher",
                crate::emitters::cypher::CypherEmitter.emit(&graph),
            ),
            (
                "surreal",
                crate::emitters::surreal::SurrealEmitter.emit(&graph),
            ),
            (
                "jsonld",
                crate::emitters::jsonld::JsonLdEmitter.emit(&graph),
            ),
            (
                "json",
                crate::emitters::json_graph::JsonGraphEmitter.emit(&graph),
            ),
            (
                "embedding",
                crate::emitters::embedding_text::EmbeddingTextEmitter.emit(&graph),
            ),
        ];
        for (name, out) in outputs {
            let text = out.unwrap_or_else(|e| panic!("{name} emitter failed: {e}"));
            assert!(!text.is_empty(), "{name} emitted nothing");
            assert!(
                text.contains("Sun"),
                "{name} output does not mention a planet in the chart"
            );
        }
    }
}
