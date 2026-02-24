use crate::types::{LoopInfo, PairTable};

/// Decompose the pair table into a hierarchical loop tree.
///
/// Returns a Vec<LoopInfo> where:
/// - Index 0 = external loop
/// - Hybrid ordering:
///   - External level: first child + DFS subtree, then remaining children
///     assigned in sequence, processed in reverse.
///   - Internal levels: assign ALL children in sequence, process subtrees
///     in reverse.
pub fn decompose(pt: &PairTable) -> Vec<LoopInfo> {
    let n = pt.n_bases;
    if n == 0 {
        return vec![];
    }

    // Step 1: Identify all base pairs (i < j)
    let mut all_pairs: Vec<(usize, usize)> = Vec::new();
    for i in 0..n {
        let j = pt.pairs[i];
        if j != i && i < j {
            all_pairs.push((i, j));
        }
    }

    if all_pairs.is_empty() {
        return vec![];
    }

    // Step 2: For each pair, find the smallest enclosing pair.
    let mut enclosing: Vec<Option<usize>> = vec![None; all_pairs.len()];
    let mut pair_indices: Vec<usize> = (0..all_pairs.len()).collect();
    pair_indices.sort_by_key(|&k| {
        let (i, j) = all_pairs[k];
        j - i
    });

    for idx in 0..pair_indices.len() {
        let k = pair_indices[idx];
        let (ki, kj) = all_pairs[k];
        let mut best: Option<usize> = None;
        let mut best_span = usize::MAX;
        for &m in &pair_indices {
            if m == k {
                continue;
            }
            let (mi, mj) = all_pairs[m];
            if mi < ki && kj < mj {
                let span = mj - mi;
                if span < best_span {
                    best_span = span;
                    best = Some(m);
                }
            }
        }
        enclosing[k] = best;
    }

    // Step 3: Group pairs by parent.
    let mut pair_to_loop: Vec<usize> = vec![0; all_pairs.len()];

    let mut external_children: Vec<usize> = Vec::new();
    for (k, enc) in enclosing.iter().enumerate() {
        if enc.is_none() {
            external_children.push(k);
        }
    }

    let mut children_of_pair: Vec<Vec<usize>> = vec![vec![]; all_pairs.len()];
    for (k, enc) in enclosing.iter().enumerate() {
        if let Some(&parent) = enc.as_ref() {
            children_of_pair[parent].push(k);
        }
    }

    external_children.sort_by_key(|&k| all_pairs[k].0);
    for children in &mut children_of_pair {
        children.sort_by_key(|&k| all_pairs[k].0);
    }

    // Step 4: Build LoopInfo with hybrid ordering.
    let mut loops: Vec<LoopInfo> = Vec::new();

    // External loop (loop 0)
    let ext_child_pairs: Vec<(usize, usize)> =
        external_children.iter().map(|&k| all_pairs[k]).collect();
    let mut ext_unpaired = Vec::new();
    {
        let mut enclosed = vec![false; n];
        for &k in &external_children {
            let (i, j) = all_pairs[k];
            enclosed[i..=j].fill(true);
        }
        for (b, &enc) in enclosed.iter().enumerate() {
            if !enc && pt.pairs[b] == b {
                ext_unpaired.push(b);
            }
        }
    }
    loops.push(LoopInfo {
        parent_pair: None,
        child_pairs: ext_child_pairs,
        unpaired_bases: ext_unpaired,
        nicks_in_loop: vec![],
    });

    // External level: first child DFS, remaining assign-all + reverse subtrees
    if !external_children.is_empty() {
        let first_k = external_children[0];
        push_loop(
            first_k,
            &mut loops,
            &mut pair_to_loop,
            &all_pairs,
            &children_of_pair,
            pt,
        );
        process_subtree(
            first_k,
            &mut loops,
            &mut pair_to_loop,
            &all_pairs,
            &children_of_pair,
            pt,
        );

        for &k in &external_children[1..] {
            push_loop(
                k,
                &mut loops,
                &mut pair_to_loop,
                &all_pairs,
                &children_of_pair,
                pt,
            );
        }
        for &k in external_children[1..].iter().rev() {
            process_subtree(
                k,
                &mut loops,
                &mut pair_to_loop,
                &all_pairs,
                &children_of_pair,
                pt,
            );
        }
    }

    // Step 5: Assign nicks to loops
    assign_nicks_to_loops(&mut loops, pt, &all_pairs);

    loops
}

/// Assign a pair k as a new LoopInfo entry.
fn push_loop(
    k: usize,
    loops: &mut Vec<LoopInfo>,
    pair_to_loop: &mut [usize],
    all_pairs: &[(usize, usize)],
    children_of_pair: &[Vec<usize>],
    pt: &PairTable,
) {
    pair_to_loop[k] = loops.len();
    let (i, j) = all_pairs[k];
    let children: Vec<(usize, usize)> = children_of_pair[k]
        .iter()
        .map(|&ck| all_pairs[ck])
        .collect();
    let mut unpaired = Vec::new();
    let mut child_enclosed = vec![false; pt.n_bases];
    for &ck in &children_of_pair[k] {
        let (ci, cj) = all_pairs[ck];
        child_enclosed[ci..=cj].fill(true);
    }
    #[allow(clippy::needless_range_loop)] // b indexes both child_enclosed and pt.pairs
    for b in (i + 1)..j {
        if !child_enclosed[b] && pt.pairs[b] == b {
            unpaired.push(b);
        }
    }
    loops.push(LoopInfo {
        parent_pair: Some((i, j)),
        child_pairs: children,
        unpaired_bases: unpaired,
        nicks_in_loop: vec![],
    });
}

/// Recursively process a subtree: assign all children of parent_k,
/// then process subtrees in reverse order.
/// Uses an explicit stack to handle deep stems without stack overflow.
fn process_subtree(
    parent_k: usize,
    loops: &mut Vec<LoopInfo>,
    pair_to_loop: &mut [usize],
    all_pairs: &[(usize, usize)],
    children_of_pair: &[Vec<usize>],
    pt: &PairTable,
) {
    let mut stack: Vec<usize> = vec![parent_k];

    while let Some(pk) = stack.pop() {
        let children = &children_of_pair[pk];
        if children.is_empty() {
            continue;
        }
        // Assign all children in sequence order
        for &k in children.iter() {
            push_loop(k, loops, pair_to_loop, all_pairs, children_of_pair, pt);
        }
        // Push in forward order → LIFO pops in reverse → correct reverse processing
        for &k in children.iter() {
            stack.push(k);
        }
    }
}

/// Determine which nicks belong to which loops.
fn assign_nicks_to_loops(loops: &mut [LoopInfo], pt: &PairTable, _all_pairs: &[(usize, usize)]) {
    let n = pt.n_bases;
    if n == 0 {
        return;
    }

    let mut base_loop: Vec<Vec<usize>> = vec![vec![]; n];

    for (li, linfo) in loops.iter().enumerate() {
        for &b in &linfo.unpaired_bases {
            base_loop[b].push(li);
        }
        if let Some((i, j)) = linfo.parent_pair {
            base_loop[i].push(li);
            base_loop[j].push(li);
        }
        for &(ci, cj) in &linfo.child_pairs {
            base_loop[ci].push(li);
            base_loop[cj].push(li);
        }
    }

    for &nick in &pt.nicks {
        let (base_a, base_b) = if nick == 0 {
            (n - 1, 0)
        } else {
            (nick - 1, nick)
        };

        let are_paired = pt.pairs[base_a] == base_b;

        if are_paired {
            let (pi, pj) = if base_a < base_b {
                (base_a, base_b)
            } else {
                (base_b, base_a)
            };
            let inside = if nick == 0 {
                false
            } else {
                pi < nick && nick <= pj
            };

            if inside {
                for linfo in loops.iter_mut() {
                    if linfo.parent_pair == Some((pi, pj)) {
                        linfo.nicks_in_loop.push(nick);
                        break;
                    }
                }
            } else {
                for linfo in loops.iter_mut() {
                    if linfo.child_pairs.contains(&(pi, pj)) {
                        linfo.nicks_in_loop.push(nick);
                        break;
                    }
                }
            }
        } else {
            let mut best_loop: Option<usize> = None;
            for &li in &base_loop[base_a] {
                if !base_loop[base_b].contains(&li) {
                    continue;
                }
                let linfo = &loops[li];
                let a_in = linfo.unpaired_bases.contains(&base_a)
                    || linfo
                        .child_pairs
                        .iter()
                        .any(|&(x, y)| x == base_a || y == base_a)
                    || linfo
                        .parent_pair
                        .is_some_and(|(x, y)| x == base_a || y == base_a);
                let b_in = linfo.unpaired_bases.contains(&base_b)
                    || linfo
                        .child_pairs
                        .iter()
                        .any(|&(x, y)| x == base_b || y == base_b)
                    || linfo
                        .parent_pair
                        .is_some_and(|(x, y)| x == base_b || y == base_b);
                if a_in && b_in {
                    best_loop = Some(match best_loop {
                        Some(prev) => prev.max(li),
                        None => li,
                    });
                }
            }
            if let Some(li) = best_loop {
                loops[li].nicks_in_loop.push(nick);
            }
        }
    }
}
