// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1

//! Sine and cosine of integer combinations of a few base angles, by angle
//! addition.
//!
//! # The problem
//!
//! Every ELP/MPP02 term is `amplitude · sin(φ)` or `cos(φ)` with
//! `φ = Σⱼ mⱼ·Aⱼ`: an integer combination of at most 13 slowly-varying
//! arguments `Aⱼ` (the Delaunay arguments, the planetary mean longitudes and
//! ζ). The straightforward evaluation forms `φ` for each of the 35,758 terms
//! and calls `sincos` on it, which is the single largest cost in the engine.
//!
//! Almost all of that work repeats. Measured over the loaded tables (and
//! asserted by `tests::elp_tables_have_the_sharing_the_design_relies_on` in
//! `elp_mpp02.rs`):
//!
//! - the 35,758 terms use only **21,325 distinct** multiplier vectors, because
//!   the longitude, latitude and distance series (and the four powers of `t`
//!   within each) share many phases;
//! - the whole set of 13-argument multiplier vectors is a prefix tree with
//!   **27,721 nodes**, because most vectors extend a shorter one by a single
//!   `(argument, multiplier)` pair;
//! - only **394 distinct `(argument, multiplier)` pairs** occur.
//!
//! # The method
//!
//! `sin` and `cos` of a sum follow from those of the parts:
//!
//! ```text
//! sin(α + β) = sin α · cos β + cos α · sin β
//! cos(α + β) = cos α · cos β − sin α · sin β
//! ```
//!
//! So per evaluation:
//!
//! 1. For each argument, build `(sin kAⱼ, cos kAⱼ)` for `k = 1 …` its largest
//!    multiplier by angle addition from a single libm `sincos(Aⱼ)`, splitting
//!    `k` in halves so the rounding error grows with `log k` and not with `k`.
//!    Negative multiples are the same pair with the sine negated.
//! 2. Each prefix-tree node is its parent's pair rotated by one table entry:
//!    four multiplies and two adds. Nodes are stored level by level (all nodes
//!    with two non-zero multipliers, then three, …), so a level depends only on
//!    earlier levels and its members are independent of one another.
//! 3. A term reads the pair at its own node.
//!
//! That is about 27,600 four-multiply rotations where there were 35,758
//! `sincos` calls of some 30 flops each.
//!
//! # Accuracy
//!
//! Each rotation adds about one unit in the last place, and the deepest node
//! is seven rotations from its table entries, so a pair is good to a few times
//! `1e-16`. That is the same order as the `sincos_f64x4` kernel it replaces
//! (1–2 ulp). The pair is **more** accurate than the old route in one respect:
//! the old route formed `φ` in floating point first, rounding it to the
//! magnitude of the largest partial sum (`ulp` of 3×10⁵ rad is 6×10⁻¹¹ rad at
//! the edge of the supported range), then took its sine. Here each `Aⱼ` is
//! reduced exactly by libm and the multiples are formed on the unit circle, so
//! no phase is ever rounded at the magnitude of `φ`.
//!
//! The two therefore differ by the *old* route's phase rounding, which grows
//! with `|t|`; `elp_mpp02.rs` measures it end to end.

use super::fused::mul_add;

/// Number of base arguments a multiplier vector can reference.
pub(crate) const N_ARGS: usize = 13;

/// One multiplier vector: the integer coefficient of each base argument.
pub(crate) type Multipliers = [i16; N_ARGS];

/// Largest `|multiplier|` the table ladders support. The loaded ELP tables top
/// out at 75; [`PhaseGraph::build`] refuses anything larger rather than
/// reading past a ladder.
const LADDER: usize = 128;

/// A `(sin, cos)` pair.
pub(crate) type Pair = [f64; 2];

/// Multiply two unit-circle pairs: the pair of the summed angle.
#[inline(always)]
fn rotate(a: Pair, b: Pair) -> Pair {
    [
        mul_add(a[0], b[1], a[1] * b[0]),
        mul_add(a[1], b[1], -(a[0] * b[0])),
    ]
}

/// Prefix tree over multiplier vectors, evaluated into a flat pool of
/// `(sin, cos)` pairs.
///
/// Pool layout, by id:
///
/// | ids | contents |
/// |---|---|
/// | `0` | the empty combination, phase 0: `(0, 1)` |
/// | `1 ..= entries.len()` | one table entry per distinct `(argument, multiplier)` |
/// | after that | derived nodes, level by level |
pub(crate) struct PhaseGraph {
    /// `(argument, multiplier)` of table entry `e`, whose pool id is `e + 1`.
    /// Sorted, so an argument's entries are contiguous.
    entries: Vec<(u8, i16)>,
    /// Largest `|multiplier|` used per argument; 0 for an unused argument.
    k_max: [usize; N_ARGS],
    /// `[parent pool id, entry pool id]` of each derived node, in level order.
    /// A parent's id is always lower than its child's.
    nodes: Vec<[u32; 2]>,
    /// Exclusive end offset into [`Self::nodes`] of each derived level
    /// (the level of nodes with two non-zero multipliers first).
    level_ends: Vec<usize>,
}

impl PhaseGraph {
    /// Build the tree for `vectors` and return the pool id of each one.
    ///
    /// Duplicate vectors share one id. A vector's non-zero multipliers are
    /// taken in argument order, so a node's value is a function of its own
    /// `(argument, multiplier)` sequence and of nothing else in the set:
    /// two graphs built from different sets give bit-identical pairs for a
    /// vector they both contain. `elp_geocentric_distance` relies on that.
    ///
    /// # How it is built
    ///
    /// This runs once, on the first evaluation, and an early version that kept
    /// the tree in `HashMap`s took 11 ms to do it, several times what the
    /// coefficient tables cost to load. So it sorts instead. A multiplier
    /// vector packs into 104 bits (one byte per argument, biased so that
    /// byte order is numeric order), with the vector's index in the 24 bits
    /// that are left over; sorting those puts every set of vectors that share
    /// a leading run of multipliers next to each other. One pass over the
    /// sorted list then walks the tree depth-first, creating a node only where
    /// a vector leaves the path of the one before it. Nodes are finally
    /// renumbered level by level (all with two non-zero multipliers, then
    /// three, …) with a counting sort on depth; that keeps each level's parents
    /// ascending, which makes the evaluation read the pool sequentially.
    pub(crate) fn build(vectors: &[Multipliers]) -> (Self, Vec<u32>) {
        assert!(
            vectors.len() < (1 << 24),
            "too many multiplier vectors to index in 24 bits"
        );

        // Table entries: every distinct (argument, multiplier), sorted.
        let mut present = [[false; 256]; N_ARGS];
        for v in vectors {
            for (j, &m) in v.iter().enumerate() {
                assert!(
                    usize::from(m.unsigned_abs()) < LADDER,
                    "multiplier {m} exceeds the {LADDER}-entry ladder"
                );
                if m != 0 {
                    present[j][(i32::from(m) + 128) as usize] = true;
                }
            }
        }
        let mut entries: Vec<(u8, i16)> = Vec::new();
        let mut entry_id = [[0_u32; 256]; N_ARGS];
        let mut k_max = [0_usize; N_ARGS];
        for (j, row) in present.iter().enumerate() {
            for (biased, &used) in row.iter().enumerate() {
                if used {
                    let m = biased as i16 - 128;
                    entries.push((j as u8, m));
                    entry_id[j][biased] = entries.len() as u32;
                    k_max[j] = k_max[j].max(usize::from(m.unsigned_abs()));
                }
            }
        }

        // Sort by (packed vector, index).
        let mut keyed: Vec<u128> = vectors
            .iter()
            .enumerate()
            .map(|(i, v)| {
                let mut packed = 0_u128;
                for &m in v {
                    packed = (packed << 8) | (i32::from(m) + 128) as u128;
                }
                // 13 bytes = 104 bits, then room for the index.
                (packed << 24) | i as u128
            })
            .collect();
        keyed.sort_unstable();

        // Walk the sorted vectors depth-first. `stack[d]` is the pool reference
        // (an entry id, or `1 + entries + provisional` for a derived node) of
        // the current path's prefix of d + 1 pairs.
        let derived_base = 1 + entries.len() as u32;
        let mut stack = [0_u32; N_ARGS];
        let mut prev_path = [0_u32; N_ARGS];
        let mut prev_len = 0_usize;
        let mut prev_ref = 0_u32;
        let mut prev_vec: Option<&Multipliers> = None;
        // Provisional derived nodes in depth-first order.
        let mut prov: Vec<(u32, u32, u8)> = Vec::new(); // (parent ref, entry id, depth)
        let mut vector_ref = vec![0_u32; vectors.len()];
        for key in keyed {
            let index = (key & 0xFF_FFFF) as usize;
            let v = &vectors[index];
            if prev_vec == Some(v) {
                vector_ref[index] = prev_ref;
                continue;
            }
            let mut path = [0_u32; N_ARGS];
            let mut len = 0;
            for (j, &m) in v.iter().enumerate() {
                if m != 0 {
                    path[len] = entry_id[j][(i32::from(m) + 128) as usize];
                    len += 1;
                }
            }
            let mut lcp = 0;
            while lcp < len && lcp < prev_len && path[lcp] == prev_path[lcp] {
                lcp += 1;
            }
            for d in lcp..len {
                stack[d] = if d == 0 {
                    path[0]
                } else {
                    prov.push((stack[d - 1], path[d], (d + 1) as u8));
                    derived_base + (prov.len() - 1) as u32
                };
            }
            prev_ref = if len == 0 { 0 } else { stack[len - 1] };
            vector_ref[index] = prev_ref;
            prev_path = path;
            prev_len = len;
            prev_vec = Some(v);
        }

        // Renumber derived nodes level by level, keeping depth-first order
        // within a level.
        let max_depth = prov.iter().map(|p| usize::from(p.2)).max().unwrap_or(1);
        let mut level_count = vec![0_usize; max_depth + 1];
        for p in &prov {
            level_count[usize::from(p.2)] += 1;
        }
        let mut level_start = vec![0_usize; max_depth + 2];
        for d in 2..=max_depth {
            level_start[d + 1] = level_start[d] + level_count[d];
        }
        let mut next_in_level = level_start.clone();
        let mut final_of = vec![0_u32; prov.len()];
        for (k, p) in prov.iter().enumerate() {
            let depth = usize::from(p.2);
            final_of[k] = derived_base + next_in_level[depth] as u32;
            next_in_level[depth] += 1;
        }
        let remap = |r: u32| -> u32 {
            if r >= derived_base {
                final_of[(r - derived_base) as usize]
            } else {
                r
            }
        };
        let mut nodes = vec![[0_u32; 2]; prov.len()];
        for (k, p) in prov.iter().enumerate() {
            nodes[(final_of[k] - derived_base) as usize] = [remap(p.0), p.1];
        }
        let level_ends: Vec<usize> = (2..=max_depth).map(|d| level_start[d + 1]).collect();
        let ids = vector_ref.iter().map(|&r| remap(r)).collect();
        (
            Self {
                entries,
                k_max,
                nodes,
                level_ends,
            },
            ids,
        )
    }

    /// A copy of this graph that keeps only the derived nodes `roots` depend
    /// on, with the new pool id of each root.
    ///
    /// Node order, and so level contiguity, is preserved; every kept node has
    /// the same parent chain, so its pair is bit-identical to the original's.
    /// Table entries are all kept (they are cheap and their ids do not move),
    /// and `roots` that are entries or the identity come back unchanged.
    pub(crate) fn pruned(&self, roots: &[u32]) -> (Self, Vec<u32>) {
        let base = 1 + self.entries.len();
        let mut needed = vec![false; self.nodes.len()];
        for &r in roots {
            if r as usize >= base {
                needed[r as usize - base] = true;
            }
        }
        for k in (0..self.nodes.len()).rev() {
            if needed[k] {
                for p in self.nodes[k] {
                    if p as usize >= base {
                        needed[p as usize - base] = true;
                    }
                }
            }
        }
        let mut new_index = vec![0_u32; self.nodes.len()];
        let mut kept = 0_u32;
        for (slot, &keep) in new_index.iter_mut().zip(&needed) {
            if keep {
                *slot = kept;
                kept += 1;
            }
        }
        let remap = |r: u32| -> u32 {
            if r as usize >= base {
                base as u32 + new_index[r as usize - base]
            } else {
                r
            }
        };
        let nodes: Vec<[u32; 2]> = (0..self.nodes.len())
            .filter(|&k| needed[k])
            .map(|k| [remap(self.nodes[k][0]), self.nodes[k][1]])
            .collect();
        let mut level_ends = Vec::with_capacity(self.level_ends.len());
        let mut done = 0;
        let mut kept_so_far = 0;
        for &end in &self.level_ends {
            kept_so_far += (done..end).filter(|&k| needed[k]).count();
            level_ends.push(kept_so_far);
            done = end;
        }
        let ids = roots.iter().map(|&r| remap(r)).collect();
        (
            Self {
                entries: self.entries.clone(),
                k_max: self.k_max,
                nodes,
                level_ends,
            },
            ids,
        )
    }

    /// Number of pairs the pool holds.
    pub(crate) fn pool_len(&self) -> usize {
        1 + self.entries.len() + self.nodes.len()
    }

    /// Number of distinct `(argument, multiplier)` table entries.
    #[cfg(test)]
    pub(crate) fn entry_count(&self) -> usize {
        self.entries.len()
    }

    /// Number of derived (two or more non-zero multipliers) nodes.
    #[cfg(test)]
    pub(crate) fn derived_count(&self) -> usize {
        self.nodes.len()
    }

    /// Fill `pool[..self.pool_len()]` with the `(sin φ, cos φ)` of every id,
    /// given the value `a[j]` of each base argument.
    ///
    /// `pool` must be at least [`Self::pool_len`] long; extra entries are left
    /// alone.
    pub(crate) fn fill_phases(&self, a: &[f64; N_ARGS], pool: &mut [Pair]) {
        // Entries are sorted by argument, so one ladder at a time suffices:
        // `ladder[k] = (sin kAⱼ, cos kAⱼ)`, built by halving `k`.
        pool[0] = [0.0, 1.0];
        let mut ladder = [[0.0_f64; 2]; LADDER];
        let mut built_for = usize::MAX;
        for (e, &(j, m)) in self.entries.iter().enumerate() {
            let j = usize::from(j);
            if j != built_for {
                let (s, c) = libm::sincos(a[j]);
                ladder[1] = [s, c];
                for k in 2..=self.k_max[j] {
                    let half = k / 2;
                    ladder[k] = rotate(ladder[half], ladder[k - half]);
                }
                built_for = j;
            }
            let [s, c] = ladder[usize::from(m.unsigned_abs())];
            pool[e + 1] = if m < 0 { [-s, c] } else { [s, c] };
        }

        let mut first = 1 + self.entries.len();
        let mut done = 0;
        for &end in &self.level_ends {
            let (earlier, level) = pool.split_at_mut(first);
            for (out, node) in level.iter_mut().zip(&self.nodes[done..end]) {
                *out = rotate(earlier[node[0] as usize], earlier[node[1] as usize]);
            }
            first += end - done;
            done = end;
        }
    }

    /// Fill `omega[..self.pool_len()]` with `ω = Σ mⱼ·Ȧⱼ`, the time derivative
    /// of every id's phase, given the derivative `adot[j]` of each argument.
    ///
    /// Uses the same tree as [`Self::fill_phases`]: a node's `ω` is its
    /// parent's plus its table entry's.
    pub(crate) fn fill_omega(&self, adot: &[f64; N_ARGS], omega: &mut [f64]) {
        omega[0] = 0.0;
        for (e, &(j, m)) in self.entries.iter().enumerate() {
            omega[e + 1] = f64::from(m) * adot[usize::from(j)];
        }
        let mut first = 1 + self.entries.len();
        let mut done = 0;
        for &end in &self.level_ends {
            let (earlier, level) = omega.split_at_mut(first);
            for (out, node) in level.iter_mut().zip(&self.nodes[done..end]) {
                *out = earlier[node[0] as usize] + earlier[node[1] as usize];
            }
            first += end - done;
            done = end;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A small deterministic set of vectors, with duplicates, an empty one,
    /// shared prefixes, negative multipliers and a large one.
    fn sample_vectors() -> Vec<Multipliers> {
        let mut v = vec![[0_i16; N_ARGS]; 8];
        v[1][0] = 1;
        v[2][0] = 1;
        v[2][2] = -1;
        v[3][0] = 2;
        v[3][2] = -1;
        v[3][3] = 1;
        v[4][0] = 1; // duplicate of v[1]
        v[5][6] = 75;
        v[5][12] = -2;
        v[6][2] = -1; // single negative pair
        v[7][0] = 1;
        v[7][2] = -1;
        v[7][5] = 69;
        v[7][9] = 16;
        v[7][12] = 2;
        // v[0] stays the all-zero vector.
        v
    }

    /// Base arguments of the given magnitude scale, mixed sign.
    fn test_args(scale: f64) -> [f64; N_ARGS] {
        let mut a = [0.0; N_ARGS];
        for (j, x) in a.iter_mut().enumerate() {
            *x = (j as f64 + 1.0) * scale * if j % 2 == 0 { 1.0 } else { -1.0 };
        }
        a
    }

    /// Reference: the old route, one `sincos` of the dense phase in order.
    fn reference(m: &Multipliers, a: &[f64; N_ARGS]) -> (f64, f64) {
        let phase: f64 = m.iter().zip(a).map(|(&m, &a)| f64::from(m) * a).sum();
        libm::sincos(phase)
    }

    /// Every id's pair equals `sincos` of the dense phase.
    ///
    /// `scale` sets the size of the arguments and so how much the reference
    /// itself rounds: with small arguments the phase is exact to ~1e-12 and the
    /// tolerance can be tight; with the large ones the engine really sees the
    /// reference is the noisy side, so `tol` is its rounding, not the graph's.
    fn check_against_dense_phase(scale: f64, tol: f64) {
        let vectors = sample_vectors();
        let (graph, ids) = PhaseGraph::build(&vectors);
        let a = test_args(scale);
        let mut pool = vec![[0.0; 2]; graph.pool_len()];
        graph.fill_phases(&a, &mut pool);
        for (m, &id) in vectors.iter().zip(&ids) {
            let (s, c) = reference(m, &a);
            let got = pool[id as usize];
            assert!((got[0] - s).abs() < tol, "{m:?}: sin {} vs {s}", got[0]);
            assert!((got[1] - c).abs() < tol, "{m:?}: cos {} vs {c}", got[1]);
        }
    }

    #[test]
    fn ids_reproduce_every_vector_with_small_arguments() {
        // Phases up to ~75 * 13 * 0.7 = 700 rad: reference rounds to ~1e-13.
        check_against_dense_phase(0.7, 2e-12);
    }

    #[test]
    fn ids_reproduce_every_vector_with_engine_sized_arguments() {
        // Phases up to ~1e6 rad: the reference rounds to ~1e-10.
        check_against_dense_phase(1234.567_891, 5e-9);
    }

    #[test]
    fn duplicates_share_an_id_and_the_empty_vector_is_the_identity() {
        let vectors = sample_vectors();
        let (graph, ids) = PhaseGraph::build(&vectors);
        assert_eq!(ids[1], ids[4], "equal vectors must share a node");
        assert_ne!(ids[1], ids[2]);
        assert_eq!(ids[0], 0, "the empty combination is pool id 0");
        let mut pool = vec![[9.0; 2]; graph.pool_len()];
        graph.fill_phases(&test_args(1.0), &mut pool);
        assert_eq!(pool[0], [0.0, 1.0]);
    }

    #[test]
    fn sharing_collapses_the_tree() {
        let vectors = sample_vectors();
        let (graph, _) = PhaseGraph::build(&vectors);
        // Distinct (arg, m) pairs: (0,1) (0,2) (2,-1) (3,1) (5,69) (6,75)
        // (9,16) (12,2) (12,-2) = 9.
        assert_eq!(graph.entry_count(), 9);
        // Derived nodes: {0:1,2:-1}, {0:2,2:-1}, {0:2,2:-1,3:1}, {6:75,12:-2},
        // {0:1,2:-1,5:69}, {..,9:16}, {..,12:2}, and the one-pair ones are
        // entries, so exactly these seven.
        assert_eq!(graph.derived_count(), 7);
    }

    #[test]
    fn parents_precede_children_and_levels_are_contiguous() {
        let (graph, _) = PhaseGraph::build(&sample_vectors());
        let first = 1 + graph.entries.len();
        for (k, node) in graph.nodes.iter().enumerate() {
            let id = (first + k) as u32;
            assert!(node[0] < id && node[1] < id, "node {id} reads a later id");
        }
        let mut level_start = 0;
        for &end in &graph.level_ends {
            let level_first = (first + level_start) as u32;
            for node in &graph.nodes[level_start..end] {
                assert!(
                    node[0] < level_first,
                    "a node reads its own level, so a level is not independent"
                );
            }
            level_start = end;
        }
        assert_eq!(level_start, graph.nodes.len());
    }

    #[test]
    fn omega_is_the_dot_product() {
        let vectors = sample_vectors();
        let (graph, ids) = PhaseGraph::build(&vectors);
        let mut adot = [0.0; N_ARGS];
        for (j, x) in adot.iter_mut().enumerate() {
            *x = 1e3 * (j as f64 - 5.5);
        }
        let mut omega = vec![0.0; graph.pool_len()];
        graph.fill_omega(&adot, &mut omega);
        for (m, &id) in vectors.iter().zip(&ids) {
            let want: f64 = m.iter().zip(&adot).map(|(&m, &d)| f64::from(m) * d).sum();
            let got = omega[id as usize];
            assert!(
                (got - want).abs() <= 1e-9 * want.abs().max(1.0),
                "{m:?}: omega {got} vs {want}"
            );
        }
    }

    /// The same vector gets the same bits from two graphs built from
    /// different sets: `elp_geocentric_distance` is bit-identical to the
    /// distance of the full evaluation only because of this.
    #[test]
    fn a_node_does_not_depend_on_the_other_vectors_in_the_set() {
        let all = sample_vectors();
        let (g_all, ids_all) = PhaseGraph::build(&all);
        let subset = vec![all[7], all[2]];
        let (g_sub, ids_sub) = PhaseGraph::build(&subset);
        let a = test_args(1234.567_891);
        let mut p_all = vec![[0.0; 2]; g_all.pool_len()];
        let mut p_sub = vec![[0.0; 2]; g_sub.pool_len()];
        g_all.fill_phases(&a, &mut p_all);
        g_sub.fill_phases(&a, &mut p_sub);
        for (sub_i, all_i) in [(0, 7), (1, 2)] {
            let x = p_all[ids_all[all_i] as usize];
            let y = p_sub[ids_sub[sub_i] as usize];
            assert_eq!(x[0].to_bits(), y[0].to_bits(), "sin of vector {all_i}");
            assert_eq!(x[1].to_bits(), y[1].to_bits(), "cos of vector {all_i}");
        }
    }

    /// Pruning keeps exactly the nodes the roots reach, and the survivors'
    /// pairs are bit-identical to the original graph's.
    #[test]
    fn pruning_keeps_only_what_the_roots_need_and_changes_no_bits() {
        let vectors = sample_vectors();
        let (graph, ids) = PhaseGraph::build(&vectors);
        // Roots: the vector with the long chain and one entry-only vector.
        let roots = [ids[7], ids[6]];
        let (small, new_roots) = graph.pruned(&roots);
        // v[7] needs {0:1,2:-1}, +5:69, +9:16, +12:2: four derived nodes.
        assert_eq!(small.derived_count(), 4);
        assert_eq!(new_roots[1], roots[1], "an entry keeps its id");
        let a = test_args(1234.567_891);
        let mut full = vec![[0.0; 2]; graph.pool_len()];
        let mut part = vec![[0.0; 2]; small.pool_len()];
        graph.fill_phases(&a, &mut full);
        small.fill_phases(&a, &mut part);
        for (old, new) in roots.iter().zip(&new_roots) {
            let (x, y) = (full[*old as usize], part[*new as usize]);
            assert_eq!(x[0].to_bits(), y[0].to_bits());
            assert_eq!(x[1].to_bits(), y[1].to_bits());
        }
        let mut omega_full = vec![0.0; graph.pool_len()];
        let mut omega_part = vec![0.0; small.pool_len()];
        let adot = [3.0; N_ARGS];
        graph.fill_omega(&adot, &mut omega_full);
        small.fill_omega(&adot, &mut omega_part);
        assert_eq!(
            omega_full[roots[0] as usize].to_bits(),
            omega_part[new_roots[0] as usize].to_bits()
        );
    }

    #[test]
    #[should_panic(expected = "ladder")]
    fn a_multiplier_beyond_the_ladder_is_refused() {
        let mut v = [0_i16; N_ARGS];
        v[0] = 200;
        let _ = PhaseGraph::build(&[v]);
    }
}
