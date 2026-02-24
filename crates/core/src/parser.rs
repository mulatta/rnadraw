use crate::types::PairTable;

/// Parse dot-bracket-plus notation into a pair table.
///
/// Characters: `(` = open pair, `)` = close pair, `.` = unpaired, `+` = strand break
///
/// Returns Err on invalid input (bad characters, unmatched parens).
pub fn parse(input: &str) -> Result<PairTable, String> {
    let mut pairs: Vec<usize> = Vec::new();
    let mut nicks: Vec<usize> = vec![0]; // always starts with 0
    let mut stack: Vec<usize> = Vec::new();
    let mut base_idx: usize = 0;

    for ch in input.chars() {
        match ch {
            '(' => {
                pairs.push(0); // placeholder
                stack.push(base_idx);
                base_idx += 1;
            }
            ')' => {
                let j = stack.pop().ok_or("unmatched ) parenthesis")?;
                pairs.push(0); // placeholder
                pairs[j] = base_idx;
                pairs[base_idx] = j;
                base_idx += 1;
            }
            '.' => {
                pairs.push(base_idx); // self-paired = unpaired
                base_idx += 1;
            }
            '+' => {
                nicks.push(base_idx);
            }
            _ => {
                return Err("bad dot-parens character".to_string());
            }
        }
    }

    if !stack.is_empty() {
        return Err("unmatched ( parenthesis".to_string());
    }

    let n_bases = base_idx;
    Ok(PairTable {
        pairs,
        nicks,
        n_bases,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_pair() {
        let pt = parse("()").unwrap();
        assert_eq!(pt.pairs, vec![1, 0]);
        assert_eq!(pt.nicks, vec![0]);
        assert_eq!(pt.n_bases, 2);
    }

    #[test]
    fn test_nested() {
        let pt = parse("(((...)))").unwrap();
        assert_eq!(pt.pairs, vec![8, 7, 6, 3, 4, 5, 2, 1, 0]);
        assert_eq!(pt.n_bases, 9);
    }

    #[test]
    fn test_nick() {
        let pt = parse("(((.+.)))").unwrap();
        assert_eq!(pt.pairs, vec![7, 6, 5, 3, 4, 2, 1, 0]);
        assert_eq!(pt.nicks, vec![0, 4]);
        assert_eq!(pt.n_bases, 8);
    }

    #[test]
    fn test_unmatched_open() {
        assert!(parse("((..)").is_err());
    }

    #[test]
    fn test_unmatched_close() {
        assert!(parse("())").is_err());
    }

    #[test]
    fn test_bad_char() {
        assert!(parse("(x)").is_err());
    }
}
