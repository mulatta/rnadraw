# rnadraw

RNA secondary structure SVG renderer. Takes dot-bracket notation and produces publication-quality SVG graphics.

## Features

- Dot-bracket and dot-bracket-plus (`+` strand breaks) notation
- Nucleotide coloring (A/U/G/C) and equilibrium probability gradient
- Automatic stem alignment
- 3' direction arrows
- Available as CLI, Rust library, and WASM module

## Usage

### CLI

```sh
# Nucleotide coloring
rnadraw -s "(((...)))" -q GGGAAACCC -c

# Probability coloring
rnadraw -s "(((...)))" -q GGGAAACCC -p 0.9,0.8,0.7,0.5,0.3,0.5,0.7,0.8,0.9

# JSON coordinate output
rnadraw -s "(((...)))" -f json

# Multi-strand (dot-bracket-plus)
rnadraw -s "((.+.))" -q GGAACCC -c
```

### WASM

```js
import init, { draw_svg_with_options } from './rnadraw_wasm.js';

await init();

const svg = draw_svg_with_options(
  '(((...)))',
  'GGGAAACCC',
  JSON.stringify({ probabilities: [0.9, 0.8, 0.7, 0.5, 0.3, 0.5, 0.7, 0.8, 0.9] })
);
```

### Rust

```rust
use rnadraw_core::svg::SvgOptions;

let svg = rnadraw_core::draw_svg("(((...)))", Some("GGGAAACCC"), &SvgOptions::default());
```

## Build

Requires [Nix](https://nixos.org/) with flakes enabled.

```sh
nix build          # CLI binary
nix build .#wasm   # WASM package (wasm-bindgen)
nix build .#wasi   # WASI binary
```

## Acknowledgments

The layout algorithm was developed by studying the visualization output
of the [NUPACK](https://nupack.org/) web application. This project
contains no NUPACK source code and is not affiliated with or endorsed
by Caltech or the NUPACK team.

## License

MIT
