use crate::types::*;

/// Generate backbone segments from layout.
///
/// Each base gets exactly 2 segments: [incoming, outgoing].
/// - Nick in pair-only loop → zero-length LINE at base position
/// - Nick in unpaired loop → degenerate ARC (t1 == t2)
/// - Pair-only loop connection → LINE from base to midpoint of consecutive bases
/// - Loop with unpaired bases → ARC on loop circle, split at midpoint angle
pub fn generate(
    loops: &[Loop],
    bases: &[Base],
    pt: &PairTable,
    loop_infos: &[LoopInfo],
) -> Vec<Vec<Segment>> {
    let n = pt.n_bases;
    if n == 0 {
        return vec![];
    }

    let nick_set: std::collections::HashSet<usize> = pt.nicks.iter().copied().collect();

    let mut segments: Vec<Vec<Segment>> = Vec::with_capacity(n);

    for i in 0..n {
        let incoming = compute_incoming(i, n, bases, loops, loop_infos, &nick_set);
        let outgoing = compute_outgoing(i, n, bases, loops, loop_infos, &nick_set);
        segments.push(vec![incoming, outgoing]);
    }

    segments
}

/// Incoming segment for base i (from base i-1 side).
fn compute_incoming(
    i: usize,
    n: usize,
    bases: &[Base],
    loops: &[Loop],
    infos: &[LoopInfo],
    nicks: &std::collections::HashSet<usize>,
) -> Segment {
    // Nick at position i means break between base i-1 and base i
    if nicks.contains(&i) {
        let shared = bases[i].loop1;
        if loop_has_unpaired(infos, shared) {
            // Degenerate ARC at base's own angle
            let lp = &loops[shared];
            let a = bases[i].angle1;
            return Segment::Arc(ArcSegment {
                x: lp.x,
                y: lp.y,
                r: lp.radius,
                t1: a,
                t2: a,
            });
        } else {
            return zero_line(bases[i].x, bases[i].y);
        }
    }

    let j = if i == 0 { n - 1 } else { i - 1 };
    let shared = bases[i].loop1;

    if loop_has_unpaired(infos, shared) {
        // ARC: t1 = base_i angle, t2 = midpoint angle
        let lp = &loops[shared];
        let angle_i = bases[i].angle1;
        let angle_j = bases[j].angle2;
        let mid = (angle_i + angle_j) / 2.0;
        Segment::Arc(ArcSegment {
            x: lp.x,
            y: lp.y,
            r: lp.radius,
            t1: angle_i,
            t2: mid,
        })
    } else {
        // LINE: from base i to midpoint of bases i and j
        let mx = (bases[i].x + bases[j].x) / 2.0;
        let my = (bases[i].y + bases[j].y) / 2.0;
        Segment::Line(LineSegment {
            x: bases[i].x,
            y: bases[i].y,
            x1: mx,
            y1: my,
        })
    }
}

/// Outgoing segment for base i (to base i+1 side).
fn compute_outgoing(
    i: usize,
    n: usize,
    bases: &[Base],
    loops: &[Loop],
    infos: &[LoopInfo],
    nicks: &std::collections::HashSet<usize>,
) -> Segment {
    let next_pos = (i + 1) % n;
    if nicks.contains(&next_pos) {
        let shared = bases[i].loop2;
        if loop_has_unpaired(infos, shared) {
            // Degenerate ARC at midpoint of base_i and base_next angles
            let lp = &loops[shared];
            let angle_i = bases[i].angle2;
            let angle_j = bases[next_pos].angle1;
            let mid = (angle_i + angle_j) / 2.0;
            return Segment::Arc(ArcSegment {
                x: lp.x,
                y: lp.y,
                r: lp.radius,
                t1: mid,
                t2: mid,
            });
        } else {
            return zero_line(bases[i].x, bases[i].y);
        }
    }

    let j = next_pos;
    let shared = bases[i].loop2;

    if loop_has_unpaired(infos, shared) {
        // ARC: t1 = midpoint angle, t2 = base_i angle
        let lp = &loops[shared];
        let angle_i = bases[i].angle2;
        let angle_j = bases[j].angle1;
        let mid = (angle_i + angle_j) / 2.0;
        Segment::Arc(ArcSegment {
            x: lp.x,
            y: lp.y,
            r: lp.radius,
            t1: mid,
            t2: angle_i,
        })
    } else {
        // LINE: from base i to midpoint of bases i and j
        let mx = (bases[i].x + bases[j].x) / 2.0;
        let my = (bases[i].y + bases[j].y) / 2.0;
        Segment::Line(LineSegment {
            x: bases[i].x,
            y: bases[i].y,
            x1: mx,
            y1: my,
        })
    }
}

/// Check if a loop uses arc segments (circular layout).
/// Arcs are used when the loop has unpaired bases OR has 3+ pairs
/// (NR-solved radius, not a simple stem or empty hairpin).
fn loop_has_unpaired(infos: &[LoopInfo], li: usize) -> bool {
    let info = &infos[li];
    if !info.unpaired_bases.is_empty() {
        return true;
    }
    let n_pairs = info.child_pairs.len() + if info.parent_pair.is_some() { 1 } else { 0 };
    n_pairs >= 3
}

fn zero_line(x: f64, y: f64) -> Segment {
    Segment::Line(LineSegment { x, y, x1: x, y1: y })
}
