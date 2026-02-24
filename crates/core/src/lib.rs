mod geometry;
mod loops;
mod parser;
mod segments;
pub mod svg;
mod types;
use std::f64::consts::PI;

pub use loops::decompose;
pub use parser::parse;
pub use types::*;

fn compute_draw_result(input: &str) -> Option<DrawResult> {
    let pt = parser::parse(input).ok()?;
    if pt.n_bases == 0 {
        return None;
    }
    let loop_infos = loops::decompose(&pt);
    if loop_infos.is_empty() {
        return None;
    }
    let (layout_loops, bases) = geometry::calculate(&loop_infos, &pt);
    let segs = segments::generate(&layout_loops, &bases, &pt, &loop_infos);
    Some(DrawResult {
        layout: Layout {
            bases,
            loops: layout_loops,
        },
        nicks: pt.nicks,
        pairs: pt.pairs,
        segments: segs,
    })
}

/// Main entry point: takes dot-bracket-plus notation, returns JSON string.
pub fn draw_structure(input: &str) -> String {
    compute_draw_result(input)
        .and_then(|r| serde_json::to_string(&r).ok())
        .unwrap_or_default()
}

/// Render dot-bracket-plus notation as SVG.
pub fn draw_svg(input: &str, seq: Option<&str>, opts: &svg::SvgOptions) -> String {
    compute_draw_result(input)
        .map(|mut r| {
            if opts.align_stem {
                if let Some(angle) = compute_stem_rotation(&r) {
                    rotate_result(&mut r, angle);
                }
            }
            svg::render(&r, seq, opts)
        })
        .unwrap_or_default()
}

/// Compute the rotation angle needed to align the primary stem vertically.
///
/// Strategy:
/// 1. If nested inner pair exists, use the direction between pair midpoints (most accurate).
/// 2. Otherwise, make the first pair bond horizontal (stem perpendicular = vertical).
fn compute_stem_rotation(result: &DrawResult) -> Option<f64> {
    let pairs = &result.pairs;
    let bases = &result.layout.bases;

    // Find first pair (smallest i where pairs[i] > i)
    let i = pairs
        .iter()
        .enumerate()
        .find(|&(idx, &p)| p != idx && idx < p)
        .map(|(idx, _)| idx)?;
    let j = pairs[i];

    // Primary: midpoint direction when nested inner pair exists
    if j > 1 && i + 1 < j - 1 && pairs[i + 1] == j - 1 {
        let mx1 = (bases[i].x + bases[j].x) / 2.0;
        let my1 = (bases[i].y + bases[j].y) / 2.0;
        let mx2 = (bases[i + 1].x + bases[j - 1].x) / 2.0;
        let my2 = (bases[i + 1].y + bases[j - 1].y) / 2.0;
        let stem_angle = (my2 - my1).atan2(mx2 - mx1);
        return Some(PI / 2.0 - stem_angle);
    }

    // Fallback: make pair bond horizontal (rotate by -bond_angle)
    let dx = bases[j].x - bases[i].x;
    let dy = bases[j].y - bases[i].y;
    if dx * dx + dy * dy < 1e-12 {
        return None;
    }
    let pair_angle = dy.atan2(dx);
    Some(-pair_angle)
}

/// Rotate all coordinates in a DrawResult around the origin.
fn rotate_result(result: &mut DrawResult, angle: f64) {
    let cos_a = angle.cos();
    let sin_a = angle.sin();

    for b in &mut result.layout.bases {
        let (x, y) = (b.x, b.y);
        b.x = x * cos_a - y * sin_a;
        b.y = x * sin_a + y * cos_a;
        let (xt, yt) = (b.xt, b.yt);
        b.xt = xt * cos_a - yt * sin_a;
        b.yt = xt * sin_a + yt * cos_a;
        b.angle1 += angle;
        b.angle2 += angle;
    }

    for l in &mut result.layout.loops {
        let (x, y) = (l.x, l.y);
        l.x = x * cos_a - y * sin_a;
        l.y = x * sin_a + y * cos_a;
        for p in &mut l.pairs {
            p.angle += angle;
        }
    }

    for segs in &mut result.segments {
        for seg in segs {
            match seg {
                Segment::Line(line) => {
                    let (x, y) = (line.x, line.y);
                    line.x = x * cos_a - y * sin_a;
                    line.y = x * sin_a + y * cos_a;
                    let (x1, y1) = (line.x1, line.y1);
                    line.x1 = x1 * cos_a - y1 * sin_a;
                    line.y1 = x1 * sin_a + y1 * cos_a;
                }
                Segment::Arc(arc) => {
                    let (x, y) = (arc.x, arc.y);
                    arc.x = x * cos_a - y * sin_a;
                    arc.y = x * sin_a + y * cos_a;
                    arc.t1 += angle;
                    arc.t2 += angle;
                }
            }
        }
    }
}
