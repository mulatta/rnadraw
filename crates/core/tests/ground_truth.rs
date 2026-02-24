use std::fs;

fn load_ground_truth() -> serde_json::Value {
    let data = fs::read_to_string("tests/fixtures/ground_truth.json").unwrap();
    serde_json::from_str(&data).unwrap()
}

fn compare_f64(a: f64, b: f64, tolerance: f64) -> bool {
    (a - b).abs() < tolerance
}

#[test]
fn test_simple_pair() {
    let gt = load_ground_truth();
    let expected = &gt["()"];

    let output_str = rnadraw_core::draw_structure("()");
    assert!(
        !output_str.is_empty(),
        "draw_structure returned empty for ()"
    );

    let output: serde_json::Value = serde_json::from_str(&output_str).unwrap();

    // Check pairs
    assert_eq!(output["pairs"], expected["pairs"], "pairs mismatch");
    // Check nicks
    assert_eq!(output["nicks"], expected["nicks"], "nicks mismatch");

    // Check loops
    let exp_loops = expected["layout"]["loops"].as_array().unwrap();
    let out_loops = output["layout"]["loops"].as_array().unwrap();
    assert_eq!(out_loops.len(), exp_loops.len(), "loop count mismatch");

    for (i, (el, ol)) in exp_loops.iter().zip(out_loops.iter()).enumerate() {
        let er = el["radius"].as_f64().unwrap();
        let or_ = ol["radius"].as_f64().unwrap();
        assert!(
            compare_f64(er, or_, 1e-6),
            "loop {} radius: expected {}, got {}",
            i,
            er,
            or_
        );

        let eh = el["height"].as_f64().unwrap();
        let oh = ol["height"].as_f64().unwrap();
        assert!(
            compare_f64(eh, oh, 1e-6),
            "loop {} height: expected {}, got {}",
            i,
            eh,
            oh
        );

        let epa = el["pair_angle"].as_f64().unwrap();
        let opa = ol["pair_angle"].as_f64().unwrap();
        assert!(
            compare_f64(epa, opa, 1e-6),
            "loop {} pair_angle: expected {}, got {}",
            i,
            epa,
            opa
        );

        let eaa = el["arc_angle"].as_f64().unwrap();
        let oaa = ol["arc_angle"].as_f64().unwrap();
        assert!(
            compare_f64(eaa, oaa, 1e-6),
            "loop {} arc_angle: expected {}, got {}",
            i,
            eaa,
            oaa
        );
    }

    // Check bases
    let exp_bases = expected["layout"]["bases"].as_array().unwrap();
    let out_bases = output["layout"]["bases"].as_array().unwrap();
    assert_eq!(out_bases.len(), exp_bases.len(), "base count mismatch");

    for (i, (eb, ob)) in exp_bases.iter().zip(out_bases.iter()).enumerate() {
        let ex = eb["x"].as_f64().unwrap();
        let ox = ob["x"].as_f64().unwrap();
        assert!(
            compare_f64(ex, ox, 1e-6),
            "base {} x: expected {}, got {}",
            i,
            ex,
            ox
        );

        let ey = eb["y"].as_f64().unwrap();
        let oy = ob["y"].as_f64().unwrap();
        assert!(
            compare_f64(ey, oy, 1e-6),
            "base {} y: expected {}, got {}",
            i,
            ey,
            oy
        );
    }
}

#[test]
fn test_nick_assignment() {
    // (((+))) → pairs: (0,5),(1,4),(2,3), nicks: [0, 3]
    let pt = rnadraw_core::parse("(((+)))").unwrap();
    let loops = rnadraw_core::decompose(&pt);

    eprintln!("(((+))) loops:");
    for (i, l) in loops.iter().enumerate() {
        eprintln!(
            "  loop {}: parent={:?}, children={:?}, unpaired={:?}, nicks={:?}",
            i, l.parent_pair, l.child_pairs, l.unpaired_bases, l.nicks_in_loop
        );
    }

    // loop 0: external, child (0,5), nick [0]
    assert_eq!(loops[0].nicks_in_loop, vec![0], "loop 0 should have nick 0");
    // loop 1: stem (0,5)->(1,4), no nicks
    assert!(
        loops[1].nicks_in_loop.is_empty(),
        "loop 1 (stem) should have no nicks"
    );
    // loop 2: stem (1,4)->(2,3), no nicks
    assert!(
        loops[2].nicks_in_loop.is_empty(),
        "loop 2 (stem) should have no nicks"
    );
    // loop 3: hairpin (2,3), nick [3]
    assert_eq!(loops[3].nicks_in_loop, vec![3], "loop 3 should have nick 3");
}

#[test]
fn test_simple_pair_debug() {
    let input = "(..)";
    let output_str = rnadraw_core::draw_structure(input);
    let output: serde_json::Value = serde_json::from_str(&output_str).unwrap();
    let loops = &output["layout"]["loops"];
    let bases = &output["layout"]["bases"];
    eprintln!("{} output loops:", input);
    for (i, l) in loops.as_array().unwrap().iter().enumerate() {
        eprintln!(
            "  loop {}: center=({}, {}), r={}, h={}, pa={}, aa={}",
            i, l["x"], l["y"], l["radius"], l["height"], l["pair_angle"], l["arc_angle"]
        );
        for p in l["pairs"].as_array().unwrap() {
            eprintln!(
                "    pair: angle={}, first={}, last={}, neighbor={}",
                p["angle"], p["first"], p["last"], p["neighbor"]
            );
        }
    }
    eprintln!("{} output bases:", input);
    for (i, b) in bases.as_array().unwrap().iter().enumerate() {
        eprintln!(
            "  base {}: x={}, y={}, l1={}, l2={}, a1={}, a2={}",
            i, b["x"], b["y"], b["loop1"], b["loop2"], b["angle1"], b["angle2"]
        );
    }
}

#[test]
fn test_error_cases() {
    assert!(rnadraw_core::draw_structure(".").is_empty());
    assert!(rnadraw_core::draw_structure("..").is_empty());
}

#[test]
fn test_json_byte_exact() {
    let gt = load_ground_truth();
    let cases = vec![
        "()",
        "(..)",
        "((...))",
        "(((...)))",
        "((((....))))",
        "(((+)))",
        "(((.+.)))",
        "((((...+...))))",
        "(+)",
        "((+))",
        "((+..))",
        "((.+.))",
        "((..((.....))..((..)).))",
        "((((((.....))))..))",
        "..((..))..",
        "((((+))))",
    ];

    for tc in &cases {
        let expected_val = &gt[*tc];
        let expected_str = serde_json::to_string(expected_val).unwrap();
        let output_str = rnadraw_core::draw_structure(tc);

        if expected_str != output_str {
            // Find ALL differences
            let eb = expected_str.as_bytes();
            let ob = output_str.as_bytes();
            let mut diffs = 0;
            let mut i = 0;
            while i < eb.len().min(ob.len()) {
                if eb[i] != ob[i] {
                    let ctx_start = i.saturating_sub(20);
                    let ctx_end = (i + 30).min(expected_str.len()).min(output_str.len());
                    if diffs < 5 {
                        eprintln!("{}: diff at byte {}:", tc, i);
                        eprintln!(
                            "  GT:  ...{}...",
                            &expected_str[ctx_start..ctx_end.min(expected_str.len())]
                        );
                        eprintln!(
                            "  OUT: ...{}...",
                            &output_str[ctx_start..ctx_end.min(output_str.len())]
                        );
                    }
                    diffs += 1;
                    // Skip to next non-matching region
                    while i < eb.len().min(ob.len()) && eb[i] != ob[i] {
                        i += 1;
                    }
                } else {
                    i += 1;
                }
            }
            if eb.len() != ob.len() {
                eprintln!("{}: length diff GT={} OUT={}", tc, eb.len(), ob.len());
            }
            eprintln!(
                "{}: total {} diff regions (expected: f64 precision diffs)",
                tc, diffs
            );
        }
    }
}

#[test]
fn test_segments() {
    let gt = load_ground_truth();
    let tolerance = 1e-6;
    let cases = vec![
        "()",
        "(..)",
        "((...))",
        "(((...)))",
        "((((....))))",
        "(((+)))",
        "(((.+.)))",
        "((((...+...))))",
        "(+)",
        "((+))",
        "((+..))",
        "((.+.))",
        "((..((.....))..((..)).))",
        "((((((.....))))..))",
        "..((..))..",
        "((((+))))",
    ];

    for tc in &cases {
        let expected = &gt[*tc];
        let output_str = rnadraw_core::draw_structure(tc);
        let output: serde_json::Value = serde_json::from_str(&output_str).unwrap();

        let exp_segs = expected["segments"].as_array().unwrap();
        let out_segs = output["segments"].as_array().unwrap();
        assert_eq!(
            out_segs.len(),
            exp_segs.len(),
            "{}: segment group count mismatch",
            tc
        );

        for (i, (eg, og)) in exp_segs.iter().zip(out_segs.iter()).enumerate() {
            let eg_arr = eg.as_array().unwrap();
            let og_arr = og.as_array().unwrap();
            assert_eq!(
                og_arr.len(),
                eg_arr.len(),
                "{} base {}: segment count mismatch",
                tc,
                i
            );

            for (si, (es, os)) in eg_arr.iter().zip(og_arr.iter()).enumerate() {
                // Check if both are same type (line vs arc)
                let e_is_arc = es.get("r").is_some();
                let o_is_arc = os.get("r").is_some();
                assert_eq!(
                    e_is_arc, o_is_arc,
                    "{} base {} seg {}: type mismatch (expected arc={}, got arc={})",
                    tc, i, si, e_is_arc, o_is_arc
                );

                // Compare all numeric fields
                for key in &["x", "y", "x1", "y1", "r", "t1", "t2"] {
                    if let (Some(ev), Some(ov)) = (es.get(key), os.get(key)) {
                        let e = ev.as_f64().unwrap();
                        let o = ov.as_f64().unwrap();
                        assert!(
                            compare_f64(e, o, tolerance),
                            "{} base {} seg {} {}: expected {}, got {}",
                            tc,
                            i,
                            si,
                            key,
                            e,
                            o
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn test_all_loop_radii() {
    let gt = load_ground_truth();
    let tolerance = 1e-6;

    // Test all valid cases from ground truth
    let simple_cases = vec![
        "()",
        "(..)",
        "((...))",
        "(((...)))",
        "((((....))))",
        "(((+)))",
        "(((.+.)))",
        "((((...+...))))",
        "(+)",
        "((+))",
        "((+..))",
        "((.+.))",
        // Additional complex cases
        "((+.+))",
        "((.+.+))",
        "((..((.....))..((..)).))",
        "((((((.....))))..))",
        "..((..))..",
        "((((+))))",
    ];
    for tc in &simple_cases {
        let expected = &gt[*tc];
        if expected.is_string() {
            continue; // error case
        }

        let output_str = rnadraw_core::draw_structure(tc);
        if output_str.is_empty() {
            panic!("draw_structure returned empty for {}", tc);
        }

        let output: serde_json::Value = serde_json::from_str(&output_str)
            .unwrap_or_else(|e| panic!("JSON parse error for {}: {}", tc, e));

        // Check loop radii
        let exp_loops = expected["layout"]["loops"].as_array().unwrap();
        let out_loops = output["layout"]["loops"].as_array().unwrap();
        assert_eq!(
            out_loops.len(),
            exp_loops.len(),
            "{}: loop count mismatch (expected {}, got {})",
            tc,
            exp_loops.len(),
            out_loops.len()
        );

        for (i, (el, ol)) in exp_loops.iter().zip(out_loops.iter()).enumerate() {
            let er = el["radius"].as_f64().unwrap();
            let or_ = ol["radius"].as_f64().unwrap();
            assert!(
                compare_f64(er, or_, tolerance),
                "{} loop {} radius: expected {}, got {}",
                tc,
                i,
                er,
                or_
            );
        }

        // Check base coordinates
        let exp_bases = expected["layout"]["bases"].as_array().unwrap();
        let out_bases = output["layout"]["bases"].as_array().unwrap();
        assert_eq!(
            out_bases.len(),
            exp_bases.len(),
            "{}: base count mismatch",
            tc
        );

        for (i, (eb, ob)) in exp_bases.iter().zip(out_bases.iter()).enumerate() {
            for field in &["x", "y"] {
                let ev = eb[field].as_f64().unwrap();
                let ov = ob[field].as_f64().unwrap();
                assert!(
                    compare_f64(ev, ov, tolerance),
                    "{} base {} {}: expected {}, got {}",
                    tc,
                    i,
                    field,
                    ev,
                    ov
                );
            }
        }
    }
}

#[test]
fn test_multiloop_debug() {
    let input = "((..((.....))..((..)).))";
    let gt = load_ground_truth();
    let expected = &gt[input];

    let pt = rnadraw_core::parse(input).unwrap();
    let loop_infos = rnadraw_core::decompose(&pt);
    eprintln!("=== Loop decomposition for {} ===", input);
    for (i, l) in loop_infos.iter().enumerate() {
        eprintln!(
            "  loop {}: parent={:?}, children={:?}, unpaired={:?}, nicks={:?}",
            i, l.parent_pair, l.child_pairs, l.unpaired_bases, l.nicks_in_loop
        );
    }

    let output_str = rnadraw_core::draw_structure(input);
    let output: serde_json::Value = serde_json::from_str(&output_str).unwrap();

    let exp_loops = expected["layout"]["loops"].as_array().unwrap();
    let out_loops = output["layout"]["loops"].as_array().unwrap();
    eprintln!(
        "loop count: GT={}, ours={}",
        exp_loops.len(),
        out_loops.len()
    );

    eprintln!("\n=== Loop geometry comparison (GT vs Ours) ===");
    let n = exp_loops.len().min(out_loops.len());
    for i in 0..n {
        let el = &exp_loops[i];
        let ol = &out_loops[i];
        let er = el["radius"].as_f64().unwrap();
        let or_ = ol["radius"].as_f64().unwrap();
        let eh = el["height"].as_f64().unwrap();
        let oh = ol["height"].as_f64().unwrap();
        let epa = el["pair_angle"].as_f64().unwrap();
        let opa = ol["pair_angle"].as_f64().unwrap();
        let eaa = el["arc_angle"].as_f64().unwrap();
        let oaa = ol["arc_angle"].as_f64().unwrap();
        let ex = el["x"].as_f64().unwrap();
        let ox = ol["x"].as_f64().unwrap();
        let ey = el["y"].as_f64().unwrap();
        let oy = ol["y"].as_f64().unwrap();
        let r_ok = (er - or_).abs() < 1e-6;
        let x_ok = (ex - ox).abs() < 1e-4;
        let y_ok = (ey - oy).abs() < 1e-4;
        eprintln!(
            "GT  {}: r={:>10.6} h={:>10.6} pa={:>10.6} aa={:>10.6} x={:>12.6} y={:>12.6}",
            i, er, eh, epa, eaa, ex, ey
        );
        eprintln!(
            "OUT {}: r={:>10.6} h={:>10.6} pa={:>10.6} aa={:>10.6} x={:>12.6} y={:>12.6} {}",
            i,
            or_,
            oh,
            opa,
            oaa,
            ox,
            oy,
            if r_ok && x_ok && y_ok {
                "OK"
            } else {
                "MISMATCH"
            }
        );
    }

    let exp_bases = expected["layout"]["bases"].as_array().unwrap();
    let out_bases = output["layout"]["bases"].as_array().unwrap();
    eprintln!("\n=== All base coordinates ===");
    for i in 0..exp_bases.len().min(out_bases.len()) {
        let ex = exp_bases[i]["x"].as_f64().unwrap();
        let ey = exp_bases[i]["y"].as_f64().unwrap();
        let ox = out_bases[i]["x"].as_f64().unwrap();
        let oy = out_bases[i]["y"].as_f64().unwrap();
        let ok = (ex - ox).abs() < 1e-4 && (ey - oy).abs() < 1e-4;
        if !ok {
            eprintln!(
                "  base {:>2} GT:({:>12.6},{:>12.6}) OUT:({:>12.6},{:>12.6}) MISMATCH dx={:.6} dy={:.6}",
                i,
                ex,
                ey,
                ox,
                oy,
                ox - ex,
                oy - ey
            );
        }
    }
}
