use crate::opensearch::Hit;

/// Which retriever a ranked list came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Retriever {
    Bm25,
    Knn,
}

/// A hit after fusion: its combined score and where each retriever ranked it (1-based).
#[derive(Debug, Clone, PartialEq)]
pub struct Fused {
    pub hit: Hit,
    pub score: f32,
    pub bm25_rank: Option<usize>,
    pub knn_rank: Option<usize>,
}

impl Fused {
    /// Provenance label: `"BV"` if both retrievers found it, else `"B"` or `"V"`.
    pub fn label(&self) -> &'static str {
        match (self.bm25_rank, self.knn_rank) {
            (Some(_), Some(_)) => "BV",
            (Some(_), None) => "B",
            _ => "V",
        }
    }

    fn set_rank(&mut self, retriever: Retriever, rank: usize) {
        match retriever {
            Retriever::Bm25 => self.bm25_rank = Some(rank),
            Retriever::Knn => self.knn_rank = Some(rank),
        }
    }
}

/// The usual RRF constant: damps the difference between the very top ranks.
pub const RRF_K: f32 = 60.0;

/// Reciprocal Rank Fusion: each list adds `1 / (k + rank)` to every hit it contains.
pub fn rrf(bm25: &[Hit], knn: &[Hit], k: f32) -> Vec<Fused> {
    let mut fused = Vec::new();
    add_list(&mut fused, bm25, Retriever::Bm25, k);
    add_list(&mut fused, knn, Retriever::Knn, k);
    fused.sort_by(|a, b| b.score.total_cmp(&a.score));
    fused
}

fn add_list(fused: &mut Vec<Fused>, hits: &[Hit], retriever: Retriever, k: f32) {
    for (index, hit) in hits.iter().enumerate() {
        let rank = index + 1;
        let contribution = 1.0 / (k + rank as f32);
        match fused.iter_mut().find(|f| f.hit.id == hit.id) {
            Some(existing) => {
                existing.score += contribution;
                existing.set_rank(retriever, rank);
            }
            None => {
                let mut f = Fused {
                    hit: hit.clone(),
                    score: contribution,
                    bm25_rank: None,
                    knn_rank: None,
                };
                f.set_rank(retriever, rank);
                fused.push(f);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hit(id: &str) -> Hit {
        Hit {
            id: id.to_string(),
            path: id.to_string(),
            line: 1,
            heading_path: vec![],
            score: 0.0,
        }
    }

    fn ids(fused: &[Fused]) -> Vec<&str> {
        fused.iter().map(|f| f.hit.id.as_str()).collect()
    }

    #[test]
    fn found_by_both_beats_found_by_one() {
        let bm25 = [hit("a"), hit("shared")];
        let knn = [hit("b"), hit("shared")];
        let fused = rrf(&bm25, &knn, RRF_K);
        assert_eq!(ids(&fused)[0], "shared");
        assert_eq!(fused.len(), 3);
    }

    #[test]
    fn score_is_sum_of_reciprocal_ranks() {
        let fused = rrf(&[hit("x")], &[hit("y"), hit("x")], 60.0);
        let x = fused.iter().find(|f| f.hit.id == "x").unwrap();
        assert_eq!(x.score, 1.0 / 61.0 + 1.0 / 62.0);
        assert_eq!((x.bm25_rank, x.knn_rank), (Some(1), Some(2)));
    }

    #[test]
    fn labels_show_provenance() {
        let fused = rrf(&[hit("both"), hit("b")], &[hit("both"), hit("v")], RRF_K);
        let label = |id: &str| fused.iter().find(|f| f.hit.id == id).unwrap().label();
        assert_eq!(label("both"), "BV");
        assert_eq!(label("b"), "B");
        assert_eq!(label("v"), "V");
    }
}
