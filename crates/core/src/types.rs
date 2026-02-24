use serde::Serialize;

/// Pair table from parsing dot-bracket-plus notation
pub struct PairTable {
    /// pairs[i] = j means base i is paired with base j; pairs[i] = i means unpaired
    pub pairs: Vec<usize>,
    /// Strand break positions. Always starts with 0.
    pub nicks: Vec<usize>,
    pub n_bases: usize,
}

/// A pair entry within a loop
#[derive(Serialize, Clone, Debug)]
pub struct LoopPair {
    pub angle: f64,
    pub first: usize,
    pub last: usize,
    pub neighbor: usize,
}

/// Loop geometry — fields in alphabetical order for JSON serialization
#[derive(Serialize, Clone, Debug)]
pub struct Loop {
    pub arc_angle: f64,
    pub height: f64,
    pub pair_angle: f64,
    pub pairs: Vec<LoopPair>,
    pub radius: f64,
    pub x: f64,
    pub y: f64,
}

/// Base coordinates — fields in alphabetical order
#[derive(Serialize, Clone, Debug)]
pub struct Base {
    pub angle1: f64,
    pub angle2: f64,
    pub length1: f64,
    pub length2: f64,
    pub loop1: usize,
    pub loop2: usize,
    pub x: f64,
    pub xt: f64,
    pub y: f64,
    pub yt: f64,
}

/// Layout containing loops and bases
#[derive(Serialize, Clone, Debug)]
pub struct Layout {
    pub bases: Vec<Base>,
    pub loops: Vec<Loop>,
}

/// Line segment
#[derive(Serialize, Clone, Debug)]
pub struct LineSegment {
    pub x: f64,
    pub x1: f64,
    pub y: f64,
    pub y1: f64,
}

/// Arc segment
#[derive(Serialize, Clone, Debug)]
pub struct ArcSegment {
    pub r: f64,
    pub t1: f64,
    pub t2: f64,
    pub x: f64,
    pub y: f64,
}

/// A segment is either a line or an arc (untagged for JSON)
#[derive(Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum Segment {
    Line(LineSegment),
    Arc(ArcSegment),
}

/// Final draw result — fields in alphabetical order
#[derive(Serialize, Clone, Debug)]
pub struct DrawResult {
    pub layout: Layout,
    pub nicks: Vec<usize>,
    pub pairs: Vec<usize>,
    pub segments: Vec<Vec<Segment>>,
}

/// Internal loop info used during decomposition (not serialized)
#[derive(Debug, Clone)]
pub struct LoopInfo {
    /// Index of the pair that creates this loop (parent pair).
    /// For external loop, this is the outermost pair.
    pub parent_pair: Option<(usize, usize)>,
    /// Child pairs contained directly in this loop: (base_i, base_j) where i < j
    pub child_pairs: Vec<(usize, usize)>,
    /// Unpaired bases directly in this loop
    pub unpaired_bases: Vec<usize>,
    /// Nick positions within this loop (base index AFTER the nick)
    pub nicks_in_loop: Vec<usize>,
}
