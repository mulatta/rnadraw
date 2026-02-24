use rnadraw_core::svg::{SvgOptions, probability_to_color as prob_to_color};
use wasm_bindgen::prelude::*;

/// Convert an equilibrium probability (0.0–1.0) to an RGB hex color string.
#[wasm_bindgen]
pub fn probability_to_color(p: f64) -> String {
    prob_to_color(p)
}

/// Compute structure layout and return JSON.
#[wasm_bindgen]
pub fn draw_structure(input: &str) -> String {
    rnadraw_core::draw_structure(input)
}

/// Render structure as SVG with sequence and JSON options.
///
/// `opts_json` is parsed as `SvgOptions` with `#[serde(default)]`,
/// so any omitted field uses the default value.
#[wasm_bindgen]
pub fn draw_svg_with_options(input: &str, seq: &str, opts_json: &str) -> String {
    let opts: SvgOptions = serde_json::from_str(opts_json).unwrap_or_default();
    let seq = if seq.is_empty() { None } else { Some(seq) };
    rnadraw_core::draw_svg(input, seq, &opts)
}
