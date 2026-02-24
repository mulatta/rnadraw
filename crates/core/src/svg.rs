use crate::types::*;
use serde::Deserialize;
use std::f64::consts::PI;
use std::fmt::Write;

/// Default nucleotide-type colors: [A, U, G, C]
pub const DEFAULT_NUCLEOTIDE_COLORS: [&str; 4] = ["green", "red", "black", "blue"];

/// Equilibrium probability colormap (dark purple → blue → cyan → green → yellow → red → dark red).
/// 11 stops evenly spaced from 0.0 to 1.0.
const PROB_COLORMAP: [(f64, f64, f64); 11] = [
    (0.19, 0.03, 0.33), // 0.0  dark purple
    (0.28, 0.14, 0.54), // 0.1
    (0.28, 0.30, 0.69), // 0.2
    (0.17, 0.49, 0.72), // 0.3
    (0.12, 0.62, 0.64), // 0.4
    (0.30, 0.73, 0.40), // 0.5
    (0.56, 0.80, 0.22), // 0.6
    (0.80, 0.80, 0.11), // 0.7
    (0.96, 0.65, 0.11), // 0.8
    (0.89, 0.40, 0.10), // 0.9
    (0.55, 0.01, 0.01), // 1.0  dark red
];

/// Convert an equilibrium probability (0.0–1.0) to an RGB hex color.
pub fn probability_to_color(p: f64) -> String {
    let p = p.clamp(0.0, 1.0);
    let t = p * 10.0;
    let i = (t as usize).min(9);
    let frac = t - i as f64;
    let (r0, g0, b0) = PROB_COLORMAP[i];
    let (r1, g1, b1) = PROB_COLORMAP[i + 1];
    let r = ((r0 + (r1 - r0) * frac) * 255.0) as u8;
    let g = ((g0 + (g1 - g0) * frac) * 255.0) as u8;
    let b = ((b0 + (b1 - b0) * frac) * 255.0) as u8;
    format!("#{r:02x}{g:02x}{b:02x}")
}

/// Legend type to render alongside the structure.
#[derive(Clone, Default, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Legend {
    #[default]
    None,
    /// Color legend for nucleotide types (A, U/T, G, C)
    Nucleotide,
    /// Gradient colorbar for equilibrium probability (0.0–1.0)
    Probability,
}

/// Options controlling SVG rendering appearance.
///
/// Defaults match reference web frontend style at scale=50:
/// - base_unit = scale * 0.05 = 2.5
/// - All widths/radii are integer multiples of base_unit
#[derive(Deserialize)]
#[serde(default)]
pub struct SvgOptions {
    /// Pixels per geometry unit (default: 50.0)
    pub scale: f64,
    /// ViewBox padding in pixels (default: 20.0)
    pub padding: f64,
    /// Backbone stroke width — 2× base_unit (default: 5.0)
    pub backbone_width: f64,
    /// Backbone stroke color (default: "black")
    pub backbone_color: String,
    /// Pair bond stroke width — 1× base_unit (default: 2.5)
    pub pair_width: f64,
    /// Pair bond stroke color (default: "black")
    pub pair_color: String,
    /// Base marker circle radius — 3× base_unit (default: 7.5)
    pub base_radius: f64,
    /// Base marker fill color (default: "#900c00")
    pub base_fill: String,
    /// Base marker stroke width — 1× base_unit (default: 2.5)
    pub base_stroke_width: f64,
    /// Whether to show nucleotide labels (default: false)
    pub show_labels: bool,
    /// Font size for labels in pixels (default: 10.0)
    pub font_size: f64,
    /// Per-nucleotide-type colors: [A, U, G, C] (default: None, uses base_fill for all)
    pub base_colors: Option<[String; 4]>,
    /// Per-base colors (e.g. for probability coloring). Takes priority over base_colors and base_fill.
    pub per_base_colors: Option<Vec<String>>,
    /// Per-base equilibrium probabilities (0.0–1.0). Converted to per_base_colors via
    /// `probability_to_color()` and sets legend to Probability automatically.
    /// Takes priority over per_base_colors if both are set.
    pub probabilities: Option<Vec<f64>>,
    /// Whether to show 3' direction arrows at strand ends (default: true)
    pub show_arrows: bool,
    /// Whether to auto-rotate so the primary stem is vertical (default: true)
    pub align_stem: bool,
    /// Legend to render alongside the structure (default: None)
    pub legend: Legend,
}

impl Default for SvgOptions {
    fn default() -> Self {
        Self {
            scale: 50.0,
            padding: 20.0,
            backbone_width: 5.0,
            backbone_color: "black".into(),
            pair_width: 2.5,
            pair_color: "black".into(),
            base_radius: 7.5,
            base_fill: "#900c00".into(),
            base_stroke_width: 2.5,
            show_labels: false,
            font_size: 10.0,
            base_colors: None,
            per_base_colors: None,
            probabilities: None,
            show_arrows: true,
            align_stem: true,
            legend: Legend::None,
        }
    }
}

impl SvgOptions {
    /// Convert `probabilities` into `per_base_colors` and set legend to Probability.
    fn resolve_probabilities(&self) -> SvgOptions {
        let colors = self
            .probabilities
            .as_ref()
            .map(|ps| ps.iter().map(|&p| probability_to_color(p)).collect());
        SvgOptions {
            scale: self.scale,
            padding: self.padding,
            backbone_width: self.backbone_width,
            backbone_color: self.backbone_color.clone(),
            pair_width: self.pair_width,
            pair_color: self.pair_color.clone(),
            base_radius: self.base_radius,
            base_fill: self.base_fill.clone(),
            base_stroke_width: self.base_stroke_width,
            show_labels: self.show_labels,
            font_size: self.font_size,
            base_colors: self.base_colors.clone(),
            per_base_colors: colors,
            probabilities: None,
            show_arrows: self.show_arrows,
            align_stem: self.align_stem,
            legend: Legend::Probability,
        }
    }
}

/// Render a DrawResult as an SVG string.
pub fn render(result: &DrawResult, seq: Option<&str>, opts: &SvgOptions) -> String {
    // If probabilities are provided, convert to per_base_colors and set legend
    let resolved;
    let opts = if opts.probabilities.is_some() {
        resolved = opts.resolve_probabilities();
        &resolved
    } else {
        opts
    };

    // Strip strand break markers (+) from sequence so indices align with bases
    let clean_seq;
    let seq = match seq {
        Some(s) if s.contains('+') => {
            clean_seq = s.replace('+', "");
            Some(clean_seq.as_str())
        }
        other => other,
    };

    let bases = &result.layout.bases;
    let loops = &result.layout.loops;
    let segments = &result.segments;
    let pairs = &result.pairs;
    let nicks = &result.nicks;
    let scale = opts.scale;

    let (min_x, min_y, max_x, max_y) = compute_bbox(bases, loops, scale, opts);
    let pad = opts.padding;
    let vb_x = min_x - pad;
    let vb_y = min_y - pad;
    let struct_w = (max_x - min_x) + 2.0 * pad;
    let vb_h = (max_y - min_y) + 2.0 * pad;

    // Reserve space for legend on the right
    let legend_w = match opts.legend {
        Legend::None => 0.0,
        Legend::Nucleotide => 80.0,
        Legend::Probability => 100.0,
    };
    let vb_w = struct_w + legend_w;

    let mut svg = String::with_capacity(4096);
    let _ = write!(
        svg,
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="{:.2} {:.2} {:.2} {:.2}">"#,
        vb_x, vb_y, vb_w, vb_h
    );

    // Arrow marker definition (must be before first use)
    if opts.show_arrows {
        let _ = write!(
            svg,
            r#"<defs><marker markerWidth="3" markerHeight="3" refX="10" refY="10" viewBox="0 0 20 20" orient="auto" id="arrowblack" markerUnits="strokeWidth"><path d="M0 0 10 0 20 10 10 20 0 20 10 10Z" fill="{}"/></marker></defs>"#,
            opts.backbone_color
        );
    }

    // Layer order (back → front):
    // 1. Pair bonds (back)
    render_pair_bonds(&mut svg, bases, pairs, scale, opts);
    // 2. Backbone
    render_backbone(&mut svg, bases, segments, nicks, scale, opts);
    // 3. 3' arrows (on backbone, before circles)
    if opts.show_arrows {
        render_end_arrows(&mut svg, bases, segments, nicks, scale, opts);
    }
    // 4. Base markers (circles — on top, covering backbone/bond endpoints)
    render_base_markers(&mut svg, bases, seq, scale, opts);
    // 5. Labels (front, optional)
    if opts.show_labels {
        if let Some(sequence) = seq {
            render_labels(&mut svg, bases, sequence, scale, opts);
        }
    }

    // 6. Legend (rightmost)
    if opts.legend != Legend::None {
        let legend_x = vb_x + struct_w;
        render_legend(&mut svg, legend_x, vb_y, vb_h, opts);
    }

    svg.push_str("</svg>");
    svg
}

fn compute_bbox(
    bases: &[Base],
    loops: &[Loop],
    scale: f64,
    opts: &SvgOptions,
) -> (f64, f64, f64, f64) {
    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;

    // Account for base circle visual extent (radius + half stroke)
    let base_extent = opts.base_radius + opts.base_stroke_width * 0.5;

    for b in bases {
        let sx = b.x * scale;
        let sy = -b.y * scale;
        min_x = min_x.min(sx - base_extent);
        min_y = min_y.min(sy - base_extent);
        max_x = max_x.max(sx + base_extent);
        max_y = max_y.max(sy + base_extent);
    }

    for l in loops {
        let cx = l.x * scale;
        let cy = -l.y * scale;
        let r = l.radius * scale;
        min_x = min_x.min(cx - r);
        min_y = min_y.min(cy - r);
        max_x = max_x.max(cx + r);
        max_y = max_y.max(cy + r);
    }

    (min_x, min_y, max_x, max_y)
}

fn render_pair_bonds(
    svg: &mut String,
    bases: &[Base],
    pairs: &[usize],
    scale: f64,
    opts: &SvgOptions,
) {
    for (i, &j) in pairs.iter().enumerate() {
        if i >= j {
            continue;
        }
        let bi = &bases[i];
        let bj = &bases[j];
        let x1 = bi.x * scale;
        let y1 = -bi.y * scale;
        let x2 = bj.x * scale;
        let y2 = -bj.y * scale;
        let _ = write!(
            svg,
            r#"<line x1="{:.2}" y1="{:.2}" x2="{:.2}" y2="{:.2}" stroke-linecap="round" stroke-width="{}" stroke="{}" />"#,
            x1, y1, x2, y2, opts.pair_width, opts.pair_color
        );
    }
}

fn render_backbone(
    svg: &mut String,
    bases: &[Base],
    segments: &[Vec<Segment>],
    nicks: &[usize],
    scale: f64,
    opts: &SvgOptions,
) {
    let n = bases.len();
    if n == 0 {
        return;
    }

    let mut strand_starts: Vec<usize> = nicks.to_vec();
    strand_starts.sort_unstable();
    strand_starts.dedup();

    for si in 0..strand_starts.len() {
        let start = strand_starts[si];
        let end = if si + 1 < strand_starts.len() {
            strand_starts[si + 1]
        } else {
            n
        };
        if start >= end {
            continue;
        }

        // Render each half-segment as individual <line> or <path>.
        // Round stroke-linecap on each piece creates smooth overlapping joins.
        for i in start..(end - 1) {
            render_individual_segment(svg, &segments[i][1], scale, opts);
            render_individual_segment(svg, &segments[i + 1][0], scale, opts);
        }
    }
}

/// Render a single backbone half-segment as an individual SVG element.
///
/// LINE → `<line>`, ARC → `<path d="M...A...">`.
/// Each has `stroke-linecap="round"` so overlapping endpoints merge smoothly.
fn render_individual_segment(svg: &mut String, seg: &Segment, scale: f64, opts: &SvgOptions) {
    match seg {
        Segment::Line(line) => {
            let x1 = line.x * scale;
            let y1 = -line.y * scale;
            let x2 = line.x1 * scale;
            let y2 = -line.y1 * scale;
            let dx = x2 - x1;
            let dy = y2 - y1;
            if dx * dx + dy * dy < 0.01 {
                return;
            }
            let _ = write!(
                svg,
                r#"<line x1="{:.2}" y1="{:.2}" x2="{:.2}" y2="{:.2}" stroke-linecap="round" stroke-opacity="1" stroke-width="{}" stroke="{}" />"#,
                x1, y1, x2, y2, opts.backbone_width, opts.backbone_color
            );
        }
        Segment::Arc(arc) => {
            if (arc.t1 - arc.t2).abs() < 1e-12 {
                return;
            }
            let r = arc.r * scale;
            let sx = (arc.x + arc.r * arc.t2.cos()) * scale;
            let sy = -(arc.y + arc.r * arc.t2.sin()) * scale;
            let ex = (arc.x + arc.r * arc.t1.cos()) * scale;
            let ey = -(arc.y + arc.r * arc.t1.sin()) * scale;

            let delta = normalize_angle(arc.t1 - arc.t2);
            let large_arc = if delta.abs() > PI { 1 } else { 0 };
            let sweep = if delta > 0.0 { 0 } else { 1 };

            let _ = write!(
                svg,
                r#"<path d="M{:.2} {:.2} A{:.2} {:.2} 0 {} {} {:.2} {:.2}" fill="none" stroke-linejoin="round" stroke-linecap="round" stroke-width="{}" stroke="{}" />"#,
                sx, sy, r, r, large_arc, sweep, ex, ey, opts.backbone_width, opts.backbone_color
            );
        }
    }
}

fn normalize_angle(mut a: f64) -> f64 {
    while a > PI {
        a -= 2.0 * PI;
    }
    while a < -PI {
        a += 2.0 * PI;
    }
    a
}

/// Render 3' arrows at the end of each strand.
fn render_end_arrows(
    svg: &mut String,
    bases: &[Base],
    segments: &[Vec<Segment>],
    nicks: &[usize],
    scale: f64,
    opts: &SvgOptions,
) {
    let n = bases.len();
    if n == 0 {
        return;
    }

    let mut strand_starts: Vec<usize> = nicks.to_vec();
    strand_starts.sort_unstable();
    strand_starts.dedup();

    for si in 0..strand_starts.len() {
        let start = strand_starts[si];
        let end_idx = if si + 1 < strand_starts.len() {
            strand_starts[si + 1] - 1
        } else {
            n - 1
        };

        // Need at least 2 bases for arrow direction
        if end_idx <= start {
            continue;
        }

        let base = &bases[end_idx];
        let bx = base.x * scale;
        let by = -base.y * scale;

        // Use INCOMING segment to determine 3' arrow direction
        // (outgoing points back into the structure via external loop)
        let Some((ax, ay)) = arrow_endpoint_from_incoming(base, &segments[end_idx][0], scale)
        else {
            continue;
        };

        // Skip if arrow would be zero-length
        let dx = ax - bx;
        let dy = ay - by;
        if dx * dx + dy * dy < 1e-6 {
            continue;
        }

        let _ = write!(
            svg,
            r#"<line x1="{:.2}" y1="{:.2}" x2="{:.2}" y2="{:.2}" stroke-linecap="round" stroke-width="{}" stroke="{}" marker-end="url(#arrowblack)" />"#,
            bx, by, ax, ay, opts.backbone_width, opts.backbone_color
        );
    }
}

/// Compute arrow endpoint by extending the incoming backbone direction past the base.
///
/// For LINE: extends the midpoint→base direction by the same distance.
/// For ARC: computes the tangent at the base position and extends by half the arc length.
fn arrow_endpoint_from_incoming(
    base: &Base,
    incoming_seg: &Segment,
    scale: f64,
) -> Option<(f64, f64)> {
    match incoming_seg {
        Segment::Line(line) => {
            // Direction: midpoint(x1,y1) → base(x,y) in math coords
            let dx = base.x - line.x1;
            let dy = base.y - line.y1;
            if dx * dx + dy * dy < 1e-12 {
                return None;
            }
            // Extend same distance beyond base, then convert to SVG
            let ax = (base.x + dx) * scale;
            let ay = -(base.y + dy) * scale;
            Some((ax, ay))
        }
        Segment::Arc(arc) => {
            if (arc.t1 - arc.t2).abs() < 1e-12 {
                return None;
            }
            // Radius vector from center to base position (math coords)
            let rvx = base.x - arc.x;
            let rvy = base.y - arc.y;
            let rv_len = (rvx * rvx + rvy * rvy).sqrt();
            if rv_len < 1e-12 {
                return None;
            }

            // Traversal direction: t2 → t1
            let delta = normalize_angle(arc.t1 - arc.t2);
            // Tangent perpendicular to radius in traversal direction (math coords)
            let (tx, ty) = if delta > 0.0 {
                (-rvy, rvx) // CCW: rotate radius 90° CCW
            } else {
                (rvy, -rvx) // CW: rotate radius 90° CW
            };

            // Arrow length ≈ half the arc segment length
            let half_arc_angle = (arc.t1 - arc.t2).abs() * 0.5;
            let arrow_len = arc.r * half_arc_angle;
            let t_len = (tx * tx + ty * ty).sqrt();
            let factor = arrow_len / t_len;

            // Extend from base in tangent direction, then convert to SVG
            let ax = (base.x + tx * factor) * scale;
            let ay = -(base.y + ty * factor) * scale;
            Some((ax, ay))
        }
    }
}

fn render_base_markers(
    svg: &mut String,
    bases: &[Base],
    seq: Option<&str>,
    scale: f64,
    opts: &SvgOptions,
) {
    let seq_bytes = seq.map(|s| s.as_bytes());

    for (i, b) in bases.iter().enumerate() {
        let cx = b.x * scale;
        let cy = -b.y * scale;

        let fill = get_base_fill(i, seq_bytes, opts);

        // Fill and stroke same color
        let _ = write!(
            svg,
            r#"<circle r="{}" cx="{:.2}" cy="{:.2}" fill="{}" stroke-width="{}" stroke="{}" />"#,
            opts.base_radius, cx, cy, fill, opts.base_stroke_width, fill
        );
    }
}

/// Determine the fill color for a base, checking per-base → per-nucleotide → uniform.
fn get_base_fill<'a>(i: usize, seq_bytes: Option<&[u8]>, opts: &'a SvgOptions) -> &'a str {
    // Priority 1: per-base colors
    if let Some(colors) = &opts.per_base_colors {
        if i < colors.len() {
            return &colors[i];
        }
    }
    // Priority 2: per-nucleotide-type colors
    if let (Some(colors), Some(sb)) = (&opts.base_colors, seq_bytes) {
        if i < sb.len() {
            return match sb[i] {
                b'A' | b'a' => &colors[0],
                b'U' | b'u' | b'T' | b't' => &colors[1],
                b'G' | b'g' => &colors[2],
                b'C' | b'c' => &colors[3],
                _ => &opts.base_fill,
            };
        }
    }
    // Priority 3: uniform
    &opts.base_fill
}

fn render_labels(svg: &mut String, bases: &[Base], seq: &str, scale: f64, opts: &SvgOptions) {
    let chars: Vec<char> = seq.chars().collect();
    for (i, b) in bases.iter().enumerate() {
        if i >= chars.len() {
            break;
        }
        let tx = b.xt * scale;
        let ty = -b.yt * scale;
        let _ = write!(
            svg,
            r#"<text x="{:.2}" y="{:.2}" font-size="{}" text-anchor="middle" dominant-baseline="central">{}</text>"#,
            tx, ty, opts.font_size, chars[i]
        );
    }
}

fn render_legend(svg: &mut String, x: f64, vb_y: f64, vb_h: f64, opts: &SvgOptions) {
    match opts.legend {
        Legend::None => {}
        Legend::Nucleotide => render_nucleotide_legend(svg, x, vb_y, vb_h, opts),
        Legend::Probability => render_probability_legend(svg, x, vb_y, vb_h),
    }
}

fn render_nucleotide_legend(svg: &mut String, x: f64, vb_y: f64, vb_h: f64, opts: &SvgOptions) {
    let colors = opts
        .base_colors
        .as_ref()
        .map(|c| [c[0].as_str(), c[1].as_str(), c[2].as_str(), c[3].as_str()])
        .unwrap_or(DEFAULT_NUCLEOTIDE_COLORS);
    let labels = ["A", "C", "G", "U"];
    let color_idx = [0, 3, 2, 1]; // A, C, G, U → indices into colors array [A, U, G, C]

    let r = opts.base_radius;
    let font_size = 14.0;
    let row_height = r * 2.0 + 8.0;
    let total_h = row_height * 4.0;
    let start_y = vb_y + (vb_h - total_h) / 2.0;
    let cx = x + 10.0 + r;

    for (row, &label) in labels.iter().enumerate() {
        let cy = start_y + row as f64 * row_height + r;
        let fill = colors[color_idx[row]];
        let _ = write!(
            svg,
            r#"<circle r="{r}" cx="{cx:.2}" cy="{cy:.2}" fill="{fill}" stroke-width="{sw}" stroke="{fill}" />"#,
            sw = opts.base_stroke_width,
        );
        let _ = write!(
            svg,
            r#"<text x="{tx:.2}" y="{cy:.2}" font-family="sans-serif" font-size="{font_size}" dominant-baseline="central">{label}</text>"#,
            tx = cx + r + 8.0,
        );
    }
}

fn render_probability_legend(svg: &mut String, x: f64, vb_y: f64, vb_h: f64) {
    let bar_w = 20.0;
    let bar_h = vb_h * 0.6;
    let bar_x = x + 10.0;
    let bar_y = vb_y + (vb_h - bar_h) / 2.0;
    let n_stops = PROB_COLORMAP.len();
    let font_size = 12.0;

    // Gradient definition
    svg.push_str(r#"<defs><linearGradient id="prob-grad" x1="0" y1="0" x2="0" y2="1">"#);
    for (i, &(r, g, b)) in PROB_COLORMAP.iter().rev().enumerate() {
        let offset = i as f64 / (n_stops - 1) as f64 * 100.0;
        let ri = (r * 255.0) as u8;
        let gi = (g * 255.0) as u8;
        let bi = (b * 255.0) as u8;
        let _ = write!(
            svg,
            "<stop offset=\"{offset:.1}%\" stop-color=\"#{ri:02x}{gi:02x}{bi:02x}\"/>",
        );
    }
    svg.push_str("</linearGradient></defs>");

    // Color bar
    let _ = write!(
        svg,
        r#"<rect x="{bar_x:.2}" y="{bar_y:.2}" width="{bar_w}" height="{bar_h:.2}" fill="url(#prob-grad)" stroke="none"/>"#,
    );

    // Tick labels: 0.0 to 1.0 in steps of 0.1
    let text_x = bar_x + bar_w + 5.0;
    for i in 0..=10 {
        let val = i as f64 / 10.0;
        let ty = bar_y + bar_h * (1.0 - val);
        let _ = write!(
            svg,
            r#"<text x="{text_x:.2}" y="{ty:.2}" font-family="sans-serif" font-size="{font_size}" dominant-baseline="central">{val:.1}</text>"#,
        );
    }

    // Rotated label
    let label_x = text_x + 35.0;
    let label_y = bar_y + bar_h / 2.0;
    let _ = write!(
        svg,
        r#"<text x="{label_x:.2}" y="{label_y:.2}" font-family="sans-serif" font-size="{font_size}" text-anchor="middle" dominant-baseline="central" transform="rotate(90,{label_x:.2},{label_y:.2})">Equilibrium probability</text>"#,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_angle() {
        let eps = 1e-10;
        assert!((normalize_angle(0.0)).abs() < eps);
        assert!((normalize_angle(PI) - PI).abs() < eps);
        assert!((normalize_angle(-PI) - (-PI)).abs() < eps);
        assert!((normalize_angle(3.0 * PI / 2.0) - (-PI / 2.0)).abs() < eps);
        assert!((normalize_angle(-3.0 * PI / 2.0) - (PI / 2.0)).abs() < eps);
    }

    #[test]
    fn test_arc_90_degrees_ccw() {
        let arc = ArcSegment {
            x: 0.0,
            y: 0.0,
            r: 1.0,
            t1: PI / 2.0,
            t2: 0.0,
        };
        let mut svg = String::new();
        let opts = SvgOptions::default();
        render_individual_segment(&mut svg, &Segment::Arc(arc), 50.0, &opts);
        assert!(svg.contains("A50.00 50.00 0 0 0"));
        assert!(svg.contains("0.00 -50.00"));
    }

    #[test]
    fn test_arc_180_degrees() {
        let arc = ArcSegment {
            x: 0.0,
            y: 0.0,
            r: 1.0,
            t1: PI,
            t2: 0.0,
        };
        let mut svg = String::new();
        let opts = SvgOptions::default();
        render_individual_segment(&mut svg, &Segment::Arc(arc), 50.0, &opts);
        assert!(svg.contains("A50.00 50.00 0 0 0"));
    }

    #[test]
    fn test_arc_cw_direction() {
        let arc = ArcSegment {
            x: 0.0,
            y: 0.0,
            r: 1.0,
            t1: 0.0,
            t2: PI / 2.0,
        };
        let mut svg = String::new();
        let opts = SvgOptions::default();
        render_individual_segment(&mut svg, &Segment::Arc(arc), 50.0, &opts);
        assert!(svg.contains("0 0 1"));
    }

    #[test]
    fn test_degenerate_arc_skipped() {
        let arc = ArcSegment {
            x: 0.0,
            y: 0.0,
            r: 1.0,
            t1: 1.5,
            t2: 1.5,
        };
        let mut svg = String::new();
        let opts = SvgOptions::default();
        render_individual_segment(&mut svg, &Segment::Arc(arc), 50.0, &opts);
        assert!(svg.is_empty());
    }

    #[test]
    fn test_draw_svg_simple_pair() {
        let svg = crate::draw_svg("()", None, &SvgOptions::default());
        assert!(!svg.is_empty());
        assert!(svg.starts_with("<svg"));
        assert!(svg.ends_with("</svg>"));
        assert!(svg.contains("<line"));
        assert!(svg.contains("<circle"));
    }

    #[test]
    fn test_draw_svg_hairpin() {
        let svg = crate::draw_svg("(((...)))", None, &SvgOptions::default());
        assert!(!svg.is_empty());
        assert!(svg.contains("<svg"));
        assert!(svg.contains("<path"));
        assert!(svg.contains("<circle"));
    }

    #[test]
    fn test_draw_svg_with_nick() {
        let svg = crate::draw_svg("((.+.))", None, &SvgOptions::default());
        assert!(!svg.is_empty());
        // Two strands → backbone elements for each; plus arrow marker path
        let line_count = svg.matches("<line").count();
        let path_count = svg.matches("<path").count();
        assert!(
            line_count + path_count >= 4,
            "expected ≥4 backbone/pair elements for nicked structure, got {}",
            line_count + path_count
        );
    }

    #[test]
    fn test_draw_svg_with_sequence() {
        let opts = SvgOptions {
            show_labels: true,
            ..SvgOptions::default()
        };
        let svg = crate::draw_svg("(((...)))", Some("GGGAAACCC"), &opts);
        assert!(svg.contains("<text"));
        assert!(svg.contains(">G<"));
        assert!(svg.contains(">A<"));
        assert!(svg.contains(">C<"));
    }

    #[test]
    fn test_draw_svg_nucleotide_colors() {
        let opts = SvgOptions {
            base_colors: Some([
                "#ff0000".into(),
                "#00ff00".into(),
                "#0000ff".into(),
                "#ffff00".into(),
            ]),
            ..SvgOptions::default()
        };
        let svg = crate::draw_svg("((..))", Some("GACUGC"), &opts);
        assert!(svg.contains(r##"fill="#0000ff""##)); // G
        assert!(svg.contains(r##"fill="#ff0000""##)); // A
        assert!(svg.contains(r##"fill="#ffff00""##)); // C
        assert!(svg.contains(r##"fill="#00ff00""##)); // U
    }

    #[test]
    fn test_draw_svg_per_base_colors() {
        let opts = SvgOptions {
            per_base_colors: Some(vec![
                "#aaa".into(),
                "#bbb".into(),
                "#ccc".into(),
                "#ddd".into(),
                "#eee".into(),
                "#fff".into(),
            ]),
            ..SvgOptions::default()
        };
        let svg = crate::draw_svg("((..))", Some("GACUGC"), &opts);
        // per_base_colors takes priority over base_colors
        assert!(svg.contains(r##"fill="#aaa""##));
        assert!(svg.contains(r##"fill="#bbb""##));
        assert!(svg.contains(r##"fill="#fff""##));
    }

    #[test]
    fn test_per_base_colors_priority() {
        let opts = SvgOptions {
            base_colors: Some(["red".into(), "green".into(), "blue".into(), "yellow".into()]),
            // Only first 2 bases get per_base_colors; rest fall through to base_colors
            per_base_colors: Some(vec!["pink".into(), "orange".into()]),
            ..SvgOptions::default()
        };
        // Sequence: G A C A G C — 'A' at index 3 falls through to base_colors[0]="red"
        let svg = crate::draw_svg("((..))", Some("GACAGC"), &opts);
        // First two bases use per_base_colors
        assert!(svg.contains(r#"fill="pink""#));
        assert!(svg.contains(r#"fill="orange""#));
        // Base 3 is 'A' → base_colors[0] = "red"
        assert!(svg.contains(r#"fill="red""#));
    }

    #[test]
    fn test_draw_svg_empty_input() {
        let svg = crate::draw_svg("", None, &SvgOptions::default());
        assert!(svg.is_empty());
    }

    #[test]
    fn test_draw_svg_invalid_input() {
        let svg = crate::draw_svg("((", None, &SvgOptions::default());
        assert!(svg.is_empty());
    }

    #[test]
    fn test_layer_order() {
        let svg = crate::draw_svg("(((...)))", None, &SvgOptions::default());
        // Layer order: pair bonds → backbone → circles (on top)
        // Use "fill=\"none\"" to distinguish backbone <path> from marker <path> in <defs>
        let line_pos = svg.find("<line x1=").unwrap(); // pair bond (not arrow)
        let backbone_pos = svg.find("fill=\"none\"").unwrap();
        let circle_pos = svg.find("<circle").unwrap();
        assert!(
            line_pos < backbone_pos,
            "pair bonds should render before backbone"
        );
        assert!(
            backbone_pos < circle_pos,
            "backbone should render before circles"
        );
    }

    #[test]
    fn test_style_defaults() {
        let svg = crate::draw_svg("(((...)))", None, &SvgOptions::default());
        // Backbone: black, stroke-width 5
        assert!(svg.contains(r#"stroke-width="5" stroke="black""#));
        // Pair bonds: black, stroke-width 2.5, round linecap
        assert!(svg.contains(r#"stroke-linecap="round" stroke-width="2.5" stroke="black""#));
        // Base circles: fill and stroke same color, radius 7.5
        assert!(svg.contains(r#"r="7.5""#));
        assert!(svg.contains(r##"stroke-width="2.5" stroke="#900c00""##));
    }

    #[test]
    fn test_arrow_marker() {
        let svg = crate::draw_svg("(((...)))", None, &SvgOptions::default());
        assert!(svg.contains("marker-end=\"url(#arrowblack)\""));
        assert!(svg.contains("<defs>"));
        assert!(svg.contains("<marker"));
    }

    #[test]
    fn test_no_arrows() {
        let opts = SvgOptions {
            show_arrows: false,
            ..SvgOptions::default()
        };
        let svg = crate::draw_svg("(((...)))", None, &opts);
        assert!(!svg.contains("marker-end"));
        assert!(!svg.contains("<defs>"));
    }

    #[test]
    fn test_default_nucleotide_preset() {
        assert_eq!(DEFAULT_NUCLEOTIDE_COLORS, ["green", "red", "black", "blue"]);
    }
}
