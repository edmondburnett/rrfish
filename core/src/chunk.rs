use std::ops::Range;

/// Byte ranges of the parts of `text` between occurrences of `separator`.
fn split_ranges(text: &str, separator: &str) -> Vec<Range<usize>> {
    let mut ranges = Vec::new();
    let mut start = 0;
    for (index, _) in text.match_indices(separator) {
        ranges.push(start..index);
        start = index + separator.len();
    }
    ranges.push(start..text.len());
    ranges
}

/// Merge neighbouring ranges while the merged span stays within `max_bytes`.
fn pack(units: Vec<Range<usize>>, max_bytes: usize) -> Vec<Range<usize>> {
    let mut chunks: Vec<Range<usize>> = Vec::new();
    for unit in units {
        match chunks.last_mut() {
            Some(current) if unit.end - current.start <= max_bytes => current.end = unit.end,
            _ => chunks.push(unit),
        }
    }
    chunks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_ranges_covers_the_gaps() {
        assert_eq!(split_ranges("a\n\nbb\n\nc", "\n\n"), [0..1, 3..5, 7..8]);
        assert_eq!(split_ranges("abc", "\n\n"), [Range { start: 0, end: 3 }]);
    }

    #[test]
    fn pack_merges_while_it_fits() {
        assert_eq!(pack(vec![0..3, 5..8, 10..20], 8), [0..8, 10..20]);
        assert_eq!(pack(vec![0..3, 5..8], 7), [0..3, 5..8]);
    }
}
