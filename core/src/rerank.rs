use serde::{Deserialize, Serialize};

use crate::rrf::Fused;

/// Client for llama-server's `/v1/rerank` endpoint (a cross-encoder).
pub struct Reranker {
    client: reqwest::Client,
    url: String,
}

#[derive(Serialize)]
struct RerankRequest<'a> {
    query: &'a str,
    documents: &'a [String],
}

#[derive(Deserialize)]
struct RerankResponse {
    results: Vec<RerankItem>,
}

#[derive(Deserialize)]
struct RerankItem {
    index: usize,
    relevance_score: f32,
}

/// A fused hit after reranking, remembering where fusion had put it.
#[derive(Debug, Clone, PartialEq)]
pub struct Reranked {
    pub result: Fused,
    pub fused_rank: usize,
    pub rerank_score: f32,
}

impl Reranker {
    /// `base_url` is the server root, e.g. `http://hostname:8082`.
    pub fn new(base_url: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            url: format!("{base_url}/v1/rerank"),
        }
    }

    /// Relevance of each document to the query, in the same order as `documents`.
    pub async fn scores(&self, query: &str, documents: &[String]) -> reqwest::Result<Vec<f32>> {
        let request = RerankRequest { query, documents };
        let response: RerankResponse = self
            .client
            .post(&self.url)
            .json(&request)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        Ok(scores_in_input_order(response))
    }
}

/// The server returns results sorted by score; put them back in document order.
fn scores_in_input_order(mut response: RerankResponse) -> Vec<f32> {
    response.results.sort_by_key(|item| item.index);
    let mut scores: Vec<f32> = Vec::with_capacity(response.results.len());
    for s in response.results {
        scores.push(s.relevance_score);
    }
    scores
}

/// Pair each fused result with its reranker score and sort by that score.
pub fn reorder(fused: Vec<Fused>, scores: &[f32]) -> Vec<Reranked> {
    let mut reranked = Vec::new();
    for (index, (result, score)) in fused.into_iter().zip(scores).enumerate() {
        reranked.push(Reranked {
            result,
            fused_rank: index + 1,
            rerank_score: *score,
        });
    }
    reranked.sort_by(|a, b| b.rerank_score.total_cmp(&a.rerank_score));
    reranked
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::opensearch::Hit;

    fn fused(id: &str) -> Fused {
        Fused {
            hit: Hit {
                id: id.to_string(),
                path: id.to_string(),
                line: 1,
                title: String::new(),
                heading_path: vec![],
                text: String::new(),
                score: 0.0,
            },
            score: 0.0,
            bm25_rank: None,
            knn_rank: None,
        }
    }

    #[test]
    fn scores_come_back_in_document_order() {
        let json = r#"{"model": "x", "results": [
            {"index": 1, "relevance_score": 0.9},
            {"index": 0, "relevance_score": 0.2}
        ]}"#;
        let response: RerankResponse = serde_json::from_str(json).unwrap();
        assert_eq!(scores_in_input_order(response), [0.2, 0.9]);
    }

    #[test]
    fn reorder_sorts_by_rerank_score_and_keeps_fused_rank() {
        let reranked = reorder(vec![fused("a"), fused("b"), fused("c")], &[0.1, 0.9, 0.5]);
        let order: Vec<(&str, usize)> = reranked
            .iter()
            .map(|r| (r.result.hit.id.as_str(), r.fused_rank))
            .collect();
        assert_eq!(order, [("b", 2), ("c", 3), ("a", 1)]);
    }
}
