use crate::types::*;
use std::f64::consts::PI;

const NICK_WEIGHT: f64 = 1.38;
const HALF_PAIR: f64 = 0.5;
const STEM_RADIUS: f64 = 0.6;
const TWO_PI: f64 = 2.0 * PI;

// External loop fixed values (NR solution for n_pairs=1, eff=1.38)
const EXT_RADIUS: f64 = 0.6765120519;
const EXT_PAIR_ANGLE: f64 = 1.663422387158712;

/// Calculate loop geometry and base coordinates.
pub fn calculate(loop_infos: &[LoopInfo], pt: &PairTable) -> (Vec<Loop>, Vec<Base>) {
    let n = pt.n_bases;
    if n == 0 || loop_infos.is_empty() {
        return (vec![], vec![]);
    }

    let n_loops = loop_infos.len();
    let mut loops: Vec<Loop> = Vec::with_capacity(n_loops);

    // Step 1: Calculate radius/height/pair_angle/arc_angle per loop
    for info in loop_infos {
        let n_pairs = info.child_pairs.len() + if info.parent_pair.is_some() { 1 } else { 0 };
        let n_unpaired = info.unpaired_bases.len();
        let n_nicks = info.nicks_in_loop.len();

        let (radius, pair_angle, arc_angle) =
            if info.parent_pair.is_none() && n_unpaired == 0 && n_pairs <= 1 {
                // External loop with no unpaired bases, single pair: fixed radius
                // (EXT_RADIUS is the NR solution for n_pairs=1, eff=1.38)
                let eff = effective_arcs(n_pairs, n_unpaired, n_nicks);
                let aa = if eff > 0.0 {
                    (TWO_PI - (n_pairs as f64) * EXT_PAIR_ANGLE) / eff
                } else {
                    TWO_PI - (n_pairs as f64) * EXT_PAIR_ANGLE
                };
                (EXT_RADIUS, EXT_PAIR_ANGLE, aa)
            } else if n_unpaired == 0 && n_pairs == 2 && !info.child_pairs.is_empty() {
                // Stem: exactly 2 pairs, no unpaired bases.
                // Covers: internal stems (parent+child, with or without nicks),
                // 2-pair external loops. Nicks don't affect stem geometry.
                let pa = 2.0 * (HALF_PAIR / STEM_RADIUS).asin();
                let eff = effective_arcs(n_pairs, n_unpaired, n_nicks);
                let aa = (TWO_PI - (n_pairs as f64) * pa) / eff;
                (STEM_RADIUS, pa, aa)
            } else if n_unpaired == 0 && info.child_pairs.is_empty() {
                // Empty hairpin (with or without nicks): parent pair only, no children, no unpaired
                // Uses external loop radius regardless of nicks
                let eff = effective_arcs(n_pairs, n_unpaired, n_nicks);
                let aa = (TWO_PI - (n_pairs as f64) * EXT_PAIR_ANGLE) / eff.max(1.0);
                (EXT_RADIUS, EXT_PAIR_ANGLE, aa)
            } else {
                // Complex loop: Newton-Raphson
                let eff = effective_arcs(n_pairs, n_unpaired, n_nicks);
                let r = newton_raphson_radius(n_pairs as f64, eff);
                let pa = 2.0 * (HALF_PAIR / r).asin();
                let aa = if eff > 0.0 {
                    (TWO_PI - (n_pairs as f64) * pa) / eff
                } else {
                    TWO_PI - (n_pairs as f64) * pa
                };
                (r, pa, aa)
            };

        let height = (radius * radius - HALF_PAIR * HALF_PAIR).sqrt();

        loops.push(Loop {
            arc_angle,
            height,
            pair_angle,
            pairs: vec![],
            radius,
            x: 0.0,
            y: 0.0,
        });
    }

    // Step 2: BFS — build loop pairs (with correct orientation) and place loops
    // Returns center angle per loop (needed for base coordinate computation)
    let centers = bfs_build_and_place(&mut loops, loop_infos, pt);

    // Step 3: Calculate base coordinates
    let mut bases = compute_bases(&loops, loop_infos, pt, &centers);

    // Step 4: Center all coordinates via bounding box
    // Compute bbox of loop centers + base (x,y) + base (xt,yt),
    // then shifts everything so bbox center = origin.
    center_coordinates(&mut loops, &mut bases);

    (loops, bases)
}

/// effective_arcs = n_pairs + n_unpaired + n_nicks * 0.38
/// (each nick replaces one regular arc with a 1.38× arc)
fn effective_arcs(n_pairs: usize, n_unpaired: usize, n_nicks: usize) -> f64 {
    (n_pairs + n_unpaired) as f64 + (n_nicks as f64) * (NICK_WEIGHT - 1.0)
}

/// Newton-Raphson: solve n_p * 2*asin(0.5/r) + eff/r = 2π
fn newton_raphson_radius(np: f64, eff: f64) -> f64 {
    // Initial guess
    let mut r = (np * 1.0 + eff) / TWO_PI;
    if r < HALF_PAIR + 0.01 {
        r = HALF_PAIR + 0.01;
    }

    for _ in 0..30 {
        let s = HALF_PAIR / r;
        if s.abs() >= 1.0 {
            r *= 1.5;
            continue;
        }
        let asin_s = s.asin();
        let f = np * 2.0 * asin_s + eff / r - TWO_PI;
        let cos_asin = (1.0 - s * s).sqrt();
        let df = np * 2.0 * (-HALF_PAIR / (r * r * cos_asin)) - eff / (r * r);
        if df.abs() < 1e-30 {
            break;
        }
        r -= f / df;
        if r < HALF_PAIR + 1e-10 {
            r = HALF_PAIR + 1e-10;
        }
    }
    r
}

// ── Loop pair construction ──────────────────────────────────────────

#[derive(Debug, Clone)]
enum Elem {
    PairFirst(usize, usize, bool), // (base, partner, is_parent)
    PairLast(usize, usize, bool),
    Unpaired(usize),
    Nick,
}

/// Collect elements on a loop's circle in CW traversal order.
/// Nick markers are inserted between elements based on sequence adjacency.
fn collect_elements(info: &LoopInfo, pt: &PairTable) -> Vec<Elem> {
    let mut items: Vec<(usize, Elem)> = Vec::new();

    if let Some((pi, pj)) = info.parent_pair {
        items.push((pj, Elem::PairFirst(pj, pi, true)));
        items.push((pi, Elem::PairLast(pi, pj, true)));
    }

    for &(ci, cj) in &info.child_pairs {
        items.push((ci, Elem::PairFirst(ci, cj, false)));
        items.push((cj, Elem::PairLast(cj, ci, false)));
    }

    for &b in &info.unpaired_bases {
        items.push((b, Elem::Unpaired(b)));
    }

    if items.is_empty() {
        return vec![];
    }

    items.sort_by_key(|&(idx, _)| idx);

    // Build ordered traversal
    let ordered: Vec<(usize, Elem)> = if let Some((_pi, pj)) = info.parent_pair {
        // Internal: start at pj, go decreasing (CW inside pair)
        let pj_pos = items.iter().position(|(idx, _)| *idx == pj).unwrap_or(0);
        let n = items.len();
        (0..n)
            .map(|i| items[(pj_pos + n - i) % n].clone())
            .collect()
    } else {
        // External: start at PairLast of first child pair (cj), go in sequence order
        // Traverse CW starting from PairLast, so PairFirst ends up at the
        // bottom of the angle range (wrap-around = pair bond).
        let start = if let Some(&(_ci, cj)) = info.child_pairs.first() {
            items.iter().position(|(idx, _)| *idx == cj).unwrap_or(0)
        } else {
            0
        };
        let n = items.len();
        (0..n).map(|i| items[(start + i) % n].clone()).collect()
    };

    // Insert nick markers. A nick sits between two sequence-adjacent bases.
    // Each nick is inserted exactly once (consumed after first match).
    let mut result: Vec<Elem> = Vec::new();
    let n = ordered.len();

    // Nick edges: (base_before, base_after) for each nick
    let mut nick_edges: Vec<(usize, usize)> = Vec::new();
    for &nick in &info.nicks_in_loop {
        if nick == 0 {
            nick_edges.push((pt.n_bases - 1, 0));
        } else {
            nick_edges.push((nick - 1, nick));
        }
    }

    let mut consumed: Vec<bool> = vec![false; nick_edges.len()];

    for i in 0..n {
        result.push(ordered[i].1.clone());

        if i + 1 < n {
            let a = ordered[i].0;
            let b = ordered[i + 1].0;
            // Check if any unconsumed nick edge matches (bidirectional)
            if let Some(ei) = nick_edges.iter().enumerate().find_map(|(ei, &(lo, hi))| {
                if !consumed[ei] && ((a == lo && b == hi) || (a == hi && b == lo)) {
                    Some(ei)
                } else {
                    None
                }
            }) {
                consumed[ei] = true;
                result.push(Elem::Nick);
            }
        }
    }

    // Wrap-around: check unconsumed nicks between last and first element
    if n >= 2 {
        let a = ordered[n - 1].0;
        let b = ordered[0].0;
        if let Some(ei) = nick_edges.iter().enumerate().find_map(|(ei, &(lo, hi))| {
            if !consumed[ei] && ((a == lo && b == hi) || (a == hi && b == lo)) {
                Some(ei)
            } else {
                None
            }
        }) {
            consumed[ei] = true;
            result.push(Elem::Nick);
        }
    }

    result
}

/// BFS: build loop pairs with correct orientation, then place loop centers.
///
/// For the root loop (external), center = π/2.
/// For each child loop visited via BFS, center = incoming_angle + π,
/// where incoming_angle is the pair angle in the parent loop.
/// This ensures multiloop branches are correctly rotated.
fn bfs_build_and_place(loops: &mut [Loop], infos: &[LoopInfo], pt: &PairTable) -> Vec<f64> {
    let n = loops.len();
    if n == 0 {
        return vec![];
    }

    let mut centers = vec![0.0f64; n];

    // Build pairs for loop 0 (external) with center = π/2
    centers[0] = PI / 2.0;
    build_loop_pairs(loops, infos, pt, 0, centers[0]);
    loops[0].x = 0.0;
    loops[0].y = 0.0;

    let mut visited = vec![false; n];
    visited[0] = true;
    let mut queue = std::collections::VecDeque::new();
    queue.push_back(0usize);

    while let Some(li) = queue.pop_front() {
        let pairs = loops[li].pairs.clone();
        for lp in &pairs {
            let ni = lp.neighbor;
            if ni >= n || visited[ni] {
                continue;
            }
            visited[ni] = true;

            // The child loop's parent pair should point back towards us.
            let child_center = lp.angle + PI;
            centers[ni] = child_center;

            // Build pairs for child loop with correct orientation
            build_loop_pairs(loops, infos, pt, ni, child_center);

            // Place child: pair midpoint in parent loop
            let mx = loops[li].x + loops[li].height * lp.angle.cos();
            let my = loops[li].y + loops[li].height * lp.angle.sin();

            // Find angle of parent pair in child loop
            let ni_angle = loops[ni]
                .pairs
                .iter()
                .find(|p| p.first == lp.last && p.last == lp.first)
                .map(|p| p.angle)
                .unwrap_or(child_center);

            // Child center = midpoint - child.height * direction
            loops[ni].x = mx - loops[ni].height * ni_angle.cos();
            loops[ni].y = my - loops[ni].height * ni_angle.sin();

            queue.push_back(ni);
        }
    }

    centers
}

/// Build LoopPair entries for a single loop with the given center angle.
fn build_loop_pairs(
    loops: &mut [Loop],
    infos: &[LoopInfo],
    pt: &PairTable,
    li: usize,
    center: f64,
) {
    loops[li].pairs.clear();
    let info = &infos[li];
    let elements = collect_elements(info, pt);
    if elements.is_empty() {
        return;
    }

    let r = loops[li].radius;
    let half_pa = (HALF_PAIR / r).asin();
    let pair_a = 2.0 * half_pa;
    let arc_a = loops[li].arc_angle;
    let nick_a = arc_a * NICK_WEIGHT;
    let is_external = info.parent_pair.is_none();

    let angles = assign_angles(
        &elements,
        half_pa,
        pair_a,
        arc_a,
        nick_a,
        is_external,
        center,
    );

    // Extract pairs: pair center = PairLast_angle + half_pa
    for (i, elem) in elements.iter().enumerate() {
        if let Elem::PairFirst(first, last, _) = elem {
            let last_idx = elements
                .iter()
                .position(|e| matches!(e, Elem::PairLast(b, _, _) if *b == *last))
                .unwrap_or(i);
            let pair_center = angles[last_idx] + half_pa;
            let neighbor = find_neighbor_loop(infos, li, *first, *last);
            loops[li].pairs.push(LoopPair {
                angle: pair_center,
                first: *first,
                last: *last,
                neighbor,
            });
        }
    }
}

/// Assign angles to loop elements.
///
/// External loops: CW (decreasing) from center + half_pa.
/// Internal loops: CCW (increasing) from center + half_pa - 2π.
///
/// `center` is the angle of the parent pair (for internal) or the first
/// child pair direction (for external). For internal loops in multiloop
/// branches, this is rotated to match the actual incoming direction.
fn assign_angles(
    elements: &[Elem],
    half_pa: f64,
    pair_a: f64,
    arc_a: f64,
    nick_a: f64,
    is_external: bool,
    center: f64,
) -> Vec<f64> {
    let n = elements.len();
    let mut angles = vec![0.0f64; n];

    let first_angle = if is_external {
        // External CW: start at center - half_pa (PairLast position)
        center - half_pa
    } else {
        center + half_pa - TWO_PI
    };

    let mut cur = first_angle;
    for i in 0..n {
        if i > 0 {
            let step = step_between(
                &elements[i - 1],
                &elements[i],
                pair_a,
                arc_a,
                nick_a,
                is_external,
            );
            if is_external {
                cur -= step;
            } else {
                cur += step;
            }
        }
        angles[i] = cur;
    }
    angles
}

/// Angular step between consecutive elements in the traversal.
///
/// PF→PL of same child pair: always pair_a (entering pair bond).
/// PL→PF of same child pair: depends on direction.
///   - External (CW): arc_a (going the long way around)
///   - Internal (CCW): pair_a (going the short way = pair bond)
fn step_between(
    prev: &Elem,
    curr: &Elem,
    pair_a: f64,
    arc_a: f64,
    nick_a: f64,
    is_external: bool,
) -> f64 {
    if matches!(curr, Elem::Nick) {
        return 0.0;
    }
    if matches!(prev, Elem::Nick) {
        return nick_a;
    }
    // PF→PL: always pair bond
    if matches!(
        (prev, curr),
        (Elem::PairFirst(_, l1, false), Elem::PairLast(f2, _, false)) if *l1 == *f2
    ) {
        return pair_a;
    }
    // PL→PF: pair bond only for internal loops (CCW direction)
    if matches!(
        (prev, curr),
        (Elem::PairLast(_, l1, false), Elem::PairFirst(f2, _, false)) if *l1 == *f2
    ) {
        return if is_external { arc_a } else { pair_a };
    }
    arc_a
}

// ── Base coordinate computation ─────────────────────────────────────

fn compute_bases(loops: &[Loop], infos: &[LoopInfo], pt: &PairTable, centers: &[f64]) -> Vec<Base> {
    let n = pt.n_bases;
    let mut bases = vec![
        Base {
            angle1: 0.0,
            angle2: 0.0,
            length1: 0.5,
            length2: 0.5,
            loop1: 0,
            loop2: 0,
            x: 0.0,
            xt: 0.0,
            y: 0.0,
            yt: 0.0,
        };
        n
    ];

    for (li, info) in infos.iter().enumerate() {
        let lp = &loops[li];
        let elements = collect_elements(info, pt);
        if elements.is_empty() {
            continue;
        }

        let half_pa = (HALF_PAIR / lp.radius).asin();
        let pair_a = 2.0 * half_pa;
        let arc_a = lp.arc_angle;
        let nick_a = arc_a * NICK_WEIGHT;
        let is_external = info.parent_pair.is_none();

        let angles = assign_angles(
            &elements,
            half_pa,
            pair_a,
            arc_a,
            nick_a,
            is_external,
            centers[li],
        );

        for (i, elem) in elements.iter().enumerate() {
            let base_idx = match elem {
                Elem::PairFirst(b, _, _) | Elem::PairLast(b, _, _) | Elem::Unpaired(b) => *b,
                Elem::Nick => continue,
            };

            let angle = angles[i];
            let x = lp.x + lp.radius * angle.cos();
            let y = lp.y + lp.radius * angle.sin();

            bases[base_idx].x = x;
            bases[base_idx].y = y;

            let partner = pt.pairs[base_idx];
            if partner != base_idx {
                // Paired base: PairFirst → angle1/loop1, PairLast → angle2/loop2
                match elem {
                    Elem::PairFirst(_, _, _) => {
                        bases[base_idx].angle1 = angle;
                        bases[base_idx].loop1 = li;
                    }
                    Elem::PairLast(_, _, _) => {
                        bases[base_idx].angle2 = angle;
                        bases[base_idx].loop2 = li;
                    }
                    _ => {}
                }
            } else {
                // Unpaired
                bases[base_idx].angle1 = angle;
                bases[base_idx].angle2 = angle;
                bases[base_idx].loop1 = li;
                bases[base_idx].loop2 = li;
            }
        }
    }

    // Build strand start/end sets from nick positions.
    // nicks[0] is always 0 (start of first strand).
    // Each nick value is the base index of the first base in a new strand.
    let mut strand_starts: Vec<bool> = vec![false; n];
    let mut strand_ends: Vec<bool> = vec![false; n];
    for &nick in &pt.nicks {
        strand_starts[nick] = true;
        // The base before the nick is a strand end
        let prev = if nick == 0 { n - 1 } else { nick - 1 };
        strand_ends[prev] = true;
    }

    // Compute xt, yt, lengths
    for i in 0..n {
        let j = pt.pairs[i];
        if j != i && i < j {
            // Paired base: xt/yt = midpoint of pair
            let mx = (bases[i].x + bases[j].x) / 2.0;
            let my = (bases[i].y + bases[j].y) / 2.0;
            bases[i].xt = mx;
            bases[j].xt = mx;
            bases[i].yt = my;
            bases[j].yt = my;
            // length = 0.69 at strand boundaries, 0.5 otherwise
            bases[i].length1 = if strand_starts[i] { 0.69 } else { 0.5 };
            bases[i].length2 = if strand_ends[i] { 0.69 } else { 0.5 };
            bases[j].length1 = if strand_starts[j] { 0.69 } else { 0.5 };
            bases[j].length2 = if strand_ends[j] { 0.69 } else { 0.5 };
        } else if j == i {
            // Unpaired base: xt/yt = position at (radius + 0.5) from loop center
            // This places the label 0.5 units outward from the loop circle
            let li = bases[i].loop1;
            let lp = &loops[li];
            let angle = bases[i].angle1;
            bases[i].xt = lp.x + (lp.radius + HALF_PAIR) * angle.cos();
            bases[i].yt = lp.y + (lp.radius + HALF_PAIR) * angle.sin();
            bases[i].length1 = if strand_starts[i] { 0.69 } else { 0.5 };
            bases[i].length2 = if strand_ends[i] { 0.69 } else { 0.5 };
        }
    }

    bases
}

// ── Bounding box centering (Phase 4) ────────────────────────

fn center_coordinates(loops: &mut [Loop], bases: &mut [Base]) {
    let mut min_x = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_y = f64::NEG_INFINITY;

    // Include loop centers
    for lp in loops.iter() {
        min_x = min_x.min(lp.x);
        max_x = max_x.max(lp.x);
        min_y = min_y.min(lp.y);
        max_y = max_y.max(lp.y);
    }

    // Include base (x, y) and (xt, yt)
    for b in bases.iter() {
        min_x = min_x.min(b.x).min(b.xt);
        max_x = max_x.max(b.x).max(b.xt);
        min_y = min_y.min(b.y).min(b.yt);
        max_y = max_y.max(b.y).max(b.yt);
    }

    let shift_x = -0.5 * (min_x + max_x);
    let shift_y = -0.5 * (min_y + max_y);

    for lp in loops.iter_mut() {
        lp.x += shift_x;
        lp.y += shift_y;
    }
    for b in bases.iter_mut() {
        b.x += shift_x;
        b.y += shift_y;
        b.xt += shift_x;
        b.yt += shift_y;
    }
}

// ── Helpers ─────────────────────────────────────────────────────────

fn find_neighbor_loop(infos: &[LoopInfo], current: usize, first: usize, last: usize) -> usize {
    // If this pair is the parent pair of current loop, find the parent loop
    if let Some((pi, pj)) = infos[current].parent_pair {
        if (first == pj && last == pi) || (first == pi && last == pj) {
            // Find loop that has (pi, pj) as child pair
            for (li, info) in infos.iter().enumerate() {
                if li == current {
                    continue;
                }
                for &(ci, cj) in &info.child_pairs {
                    if (ci == pi && cj == pj) || (ci == pj && cj == pi) {
                        return li;
                    }
                }
            }
            return 0;
        }
    }
    // Otherwise find loop that has (first, last) as parent pair
    for (li, info) in infos.iter().enumerate() {
        if let Some((a, b)) = info.parent_pair {
            if (a == first && b == last) || (a == last && b == first) {
                return li;
            }
        }
    }
    0
}
