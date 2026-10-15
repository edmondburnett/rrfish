use std::ops::Range;

use crate::doc::Section;

/// Default chunk budget in bytes (~400 tokens of English).
pub const MAX_CHUNK_BYTES: usize = 1500;

/// A piece of a section small enough to embed; what gets indexed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chunk {
    pub heading_path: Vec<String>,
    /// Approximate source line: the section's line plus the newlines before this chunk.
    pub line: u32,
    pub text: String,
    pub code_langs: Vec<String>,
}

/// Split a section into chunks of at most `max_bytes`, breaking at blank lines, then lines.
pub fn chunk_section(section: &Section, max_bytes: usize) -> Vec<Chunk> {
    let text = &section.text;
    let mut chunks = Vec::new();
    for range in pack(units(text, max_bytes), max_bytes) {
        let line_offset = text[..range.start].matches('\n').count() as u32;
        let line = section.line + line_offset;
        let chunk_text = text[range].to_string();
        let heading_path = section.heading_path.clone();
        let code_langs = section.code_langs.clone();
        chunks.push(Chunk {
            line,
            text: chunk_text,
            heading_path,
            code_langs,
        });
    }
    chunks
}

/// The smallest pieces `pack` may combine: blocks, or lines of an oversized block,
/// or hard cuts of an oversized line. Blank pieces are dropped.
fn units(text: &str, max_bytes: usize) -> Vec<Range<usize>> {
    let mut units = Vec::new();
    for block in split_ranges(text, "\n\n") {
        if block.len() <= max_bytes {
            units.push(block);
            continue;
        }
        for line in split_ranges(&text[block.clone()], "\n") {
            let line = block.start + line.start..block.start + line.end;
            if line.len() <= max_bytes {
                units.push(line);
            } else {
                units.extend(hard_split(text, line, max_bytes));
            }
        }
    }
    units.retain(|unit| !text[unit.clone()].trim().is_empty());
    units
}

/// Cut `range` into pieces of at most `max_bytes`, never inside a UTF-8 character.
fn hard_split(text: &str, range: Range<usize>, max_bytes: usize) -> Vec<Range<usize>> {
    let mut pieces = Vec::new();
    let mut start = range.start;
    while start < range.end {
        let mut end = text.floor_char_boundary((start + max_bytes).min(range.end));
        if end == start {
            end = text.ceil_char_boundary(start + 1);
        }
        pieces.push(start..end);
        start = end;
    }
    pieces
}

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

    fn section(text: &str) -> Section {
        Section {
            heading_path: vec!["A".to_string()],
            line: 10,
            text: text.to_string(),
            code_langs: vec![],
        }
    }

    fn texts(text: &str, max: usize) -> Vec<String> {
        chunk_section(&section(text), max).into_iter().map(|c| c.text).collect()
    }

    #[test]
    fn small_section_is_one_chunk() {
        assert_eq!(texts("hello\n\nworld", 100), ["hello\n\nworld"]);
    }

    #[test]
    fn splits_at_blank_lines_first() {
        assert_eq!(texts("aaaa\n\nbbbb\n\ncccc", 10), ["aaaa\n\nbbbb", "cccc"]);
    }

    #[test]
    fn oversized_block_splits_at_lines() {
        assert_eq!(texts("aaaa\nbbbb\ncccc", 10), ["aaaa\nbbbb", "cccc"]);
    }

    #[test]
    fn oversized_line_is_cut_on_char_boundaries() {
        assert_eq!(texts("ééééé", 4), ["éé", "éé", "é"]);
    }

    #[test]
    fn chunks_keep_context_and_track_lines() {
        let chunks = chunk_section(&section("aaaa\n\nbbbb\n\ncccc"), 10);
        assert_eq!(chunks[1].heading_path, ["A"]);
        assert_eq!(chunks[0].line, 10);
        assert_eq!(chunks[1].line, 14);
    }
}
