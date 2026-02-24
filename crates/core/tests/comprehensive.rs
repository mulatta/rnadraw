use std::collections::HashMap;
use std::fs;

fn compare_f64(a: f64, b: f64, tolerance: f64) -> bool {
    (a - b).abs() < tolerance
}

#[test]
fn test_comprehensive() {
    let data =
        fs::read_to_string("tests/fixtures/comprehensive_gt.json").expect("comprehensive_gt.json");
    let gt: HashMap<String, serde_json::Value> =
        serde_json::from_str(&data).expect("parse comprehensive_gt.json");

    let tolerance = 1e-6;
    let mut pass = 0u32;
    let mut fail = 0u32;
    let mut failures: Vec<String> = Vec::new();

    for (structure, expected) in &gt {
        let output_str = rnadraw_core::draw_structure(structure);
        if output_str.is_empty() {
            let msg = format!("{}: draw_structure returned empty", structure);
            failures.push(msg);
            fail += 1;
            continue;
        }

        let output: serde_json::Value = match serde_json::from_str(&output_str) {
            Ok(v) => v,
            Err(e) => {
                failures.push(format!("{}: JSON parse error: {}", structure, e));
                fail += 1;
                continue;
            }
        };

        let mut case_ok = true;
        let mut case_errors: Vec<String> = Vec::new();

        // ── pairs ────────────────────────────────────────────────────────
        if output["pairs"] != expected["pairs"] {
            case_errors.push(format!(
                "pairs mismatch: expected {:?}, got {:?}",
                expected["pairs"], output["pairs"]
            ));
            case_ok = false;
        }

        // ── nicks ────────────────────────────────────────────────────────
        if output["nicks"] != expected["nicks"] {
            case_errors.push(format!(
                "nicks mismatch: expected {:?}, got {:?}",
                expected["nicks"], output["nicks"]
            ));
            case_ok = false;
        }

        // ── loops ────────────────────────────────────────────────────────
        let exp_loops = expected["layout"]["loops"].as_array();
        let out_loops = output["layout"]["loops"].as_array();
        match (exp_loops, out_loops) {
            (Some(el), Some(ol)) => {
                if el.len() != ol.len() {
                    case_errors.push(format!(
                        "loop count: expected {}, got {}",
                        el.len(),
                        ol.len()
                    ));
                    case_ok = false;
                } else {
                    for (i, (e, o)) in el.iter().zip(ol.iter()).enumerate() {
                        for field in &["radius", "height", "pair_angle", "arc_angle", "x", "y"] {
                            let ev = e[field].as_f64();
                            let ov = o[field].as_f64();
                            match (ev, ov) {
                                (Some(a), Some(b)) => {
                                    if !compare_f64(a, b, tolerance) {
                                        case_errors.push(format!(
                                            "loop[{}].{}: expected {}, got {} (diff={})",
                                            i,
                                            field,
                                            a,
                                            b,
                                            (a - b).abs()
                                        ));
                                        case_ok = false;
                                    }
                                }
                                _ => {
                                    case_errors.push(format!(
                                        "loop[{}].{}: missing value (exp={:?}, out={:?})",
                                        i, field, ev, ov
                                    ));
                                    case_ok = false;
                                }
                            }
                        }
                    }
                }
            }
            _ => {
                case_errors.push("loops: missing array".to_string());
                case_ok = false;
            }
        }

        // ── bases ────────────────────────────────────────────────────────
        let exp_bases = expected["layout"]["bases"].as_array();
        let out_bases = output["layout"]["bases"].as_array();
        match (exp_bases, out_bases) {
            (Some(eb), Some(ob)) => {
                if eb.len() != ob.len() {
                    case_errors.push(format!(
                        "base count: expected {}, got {}",
                        eb.len(),
                        ob.len()
                    ));
                    case_ok = false;
                } else {
                    for (i, (e, o)) in eb.iter().zip(ob.iter()).enumerate() {
                        for field in &[
                            "x", "y", "xt", "yt", "angle1", "angle2", "length1", "length2",
                        ] {
                            let ev = e[field].as_f64();
                            let ov = o[field].as_f64();
                            match (ev, ov) {
                                (Some(a), Some(b)) => {
                                    if !compare_f64(a, b, tolerance) {
                                        case_errors.push(format!(
                                            "base[{}].{}: expected {}, got {} (diff={})",
                                            i,
                                            field,
                                            a,
                                            b,
                                            (a - b).abs()
                                        ));
                                        case_ok = false;
                                    }
                                }
                                _ => {} // some fields may be absent in certain cases
                            }
                        }
                        // Integer fields
                        for field in &["loop1", "loop2"] {
                            if e[field] != o[field] {
                                case_errors.push(format!(
                                    "base[{}].{}: expected {:?}, got {:?}",
                                    i, field, e[field], o[field]
                                ));
                                case_ok = false;
                            }
                        }
                    }
                }
            }
            _ => {
                case_errors.push("bases: missing array".to_string());
                case_ok = false;
            }
        }

        // ── segments ─────────────────────────────────────────────────────
        let exp_segs = expected["segments"].as_array();
        let out_segs = output["segments"].as_array();
        match (exp_segs, out_segs) {
            (Some(es), Some(os)) => {
                if es.len() != os.len() {
                    case_errors.push(format!(
                        "segment group count: expected {}, got {}",
                        es.len(),
                        os.len()
                    ));
                    case_ok = false;
                } else {
                    for (i, (eg, og)) in es.iter().zip(os.iter()).enumerate() {
                        let eg_arr = eg.as_array();
                        let og_arr = og.as_array();
                        match (eg_arr, og_arr) {
                            (Some(ea), Some(oa)) => {
                                if ea.len() != oa.len() {
                                    case_errors.push(format!(
                                        "seg group[{}] count: expected {}, got {}",
                                        i,
                                        ea.len(),
                                        oa.len()
                                    ));
                                    case_ok = false;
                                } else {
                                    for (si, (e, o)) in ea.iter().zip(oa.iter()).enumerate() {
                                        let e_is_arc = e.get("r").is_some();
                                        let o_is_arc = o.get("r").is_some();
                                        if e_is_arc != o_is_arc {
                                            case_errors.push(format!(
                                                "seg[{}][{}]: type mismatch (arc exp={}, got={})",
                                                i, si, e_is_arc, o_is_arc
                                            ));
                                            case_ok = false;
                                        }
                                        for key in &["x", "y", "x1", "y1", "r", "t1", "t2"] {
                                            if let (Some(ev), Some(ov)) = (e.get(key), o.get(key)) {
                                                let a = ev.as_f64().unwrap_or(0.0);
                                                let b = ov.as_f64().unwrap_or(0.0);
                                                if !compare_f64(a, b, tolerance) {
                                                    case_errors.push(format!(
                                                        "seg[{}][{}].{}: expected {}, got {}",
                                                        i, si, key, a, b
                                                    ));
                                                    case_ok = false;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            _ => {
                                case_errors.push(format!("seg group[{}]: missing array", i));
                                case_ok = false;
                            }
                        }
                    }
                }
            }
            _ => {
                case_errors.push("segments: missing array".to_string());
                case_ok = false;
            }
        }

        if case_ok {
            pass += 1;
        } else {
            // Limit error output per case
            let first_errors: Vec<&str> = case_errors.iter().take(3).map(|s| s.as_str()).collect();
            failures.push(format!(
                "{} ({} errors): {}",
                structure,
                case_errors.len(),
                first_errors.join("; ")
            ));
            fail += 1;
        }
    }

    // Collect field-level failure statistics
    let mut field_fails: HashMap<String, u32> = HashMap::new();
    for f in &failures {
        // Extract field names from error messages
        for field in &[
            "pairs",
            "nicks",
            "loop count",
            "loop",
            "radius",
            "height",
            "pair_angle",
            "arc_angle",
            "base count",
            "base",
            "length1",
            "length2",
            "angle1",
            "angle2",
            "xt",
            "yt",
            "loop1",
            "loop2",
            "segment",
            "seg",
        ] {
            if f.contains(field) {
                *field_fails.entry(field.to_string()).or_insert(0) += 1;
            }
        }
    }

    eprintln!(
        "\n=== Comprehensive test: {} pass, {} fail out of {} ===",
        pass,
        fail,
        gt.len()
    );
    if !field_fails.is_empty() {
        let mut sorted: Vec<_> = field_fails.iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(a.1));
        eprintln!("\nField failure counts:");
        for (field, count) in &sorted {
            eprintln!("  {}: {}", field, count);
        }
    }
    if !failures.is_empty() {
        let show = failures.len().min(20);
        eprintln!("\nFirst {} failures:", show);
        for f in &failures[..show] {
            eprintln!("  FAIL: {}", f);
        }
        if failures.len() > show {
            eprintln!("  ... and {} more", failures.len() - show);
        }
    }
    assert_eq!(
        fail,
        0,
        "{} of {} cases failed (first: {})",
        fail,
        pass + fail,
        failures.first().unwrap_or(&String::new())
    );
}
