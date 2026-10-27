use serde_json::{Value, json};

use crate::embed::{Embedder, query_text};
use crate::opensearch::{Hit, OpenSearch};

/// Fields returned with each hit; the embedding is left out on purpose.
const SOURCE_FIELDS: [&str; 5] = ["path", "line", "title", "heading_path", "text"];

/// Keyword search: BM25 over title (boosted), heading path and text.
pub async fn bm25(opensearch: &OpenSearch, query: &str, size: usize) -> reqwest::Result<Vec<Hit>> {
    opensearch.search(&bm25_query(query, size)).await
}

pub async fn knn(opensearch: &OpenSearch, embedder: &Embedder, query: &str, size: usize) -> reqwest::Result<Vec<Hit>> {
    let mut vectors = embedder.embed(&[query_text(query)]).await?;
    let vector = vectors.remove(0);
    opensearch.search(&knn_query(&vector, size)).await
}

fn bm25_query(query: &str, size: usize) -> Value {
    json!({
        "size": size,
        "_source": SOURCE_FIELDS,
        "query": {
            "multi_match": {
                "query": query,
                "fields": ["title^2", "heading_path", "text"]
            }
        }
    })
}

fn knn_query(vector: &[f32], size: usize) -> Value {
    json!({
        "size": size,
        "_source": SOURCE_FIELDS,
        "query": {
            "knn": {
                "embedding": { "vector": vector, "k": size }
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bm25_query_searches_text_fields() {
        let body = bm25_query("pipewire", 5);
        assert_eq!(body["size"], 5);
        assert_eq!(body["query"]["multi_match"]["query"], "pipewire");
        assert_eq!(body["query"]["multi_match"]["fields"][0], "title^2");
    }

    #[test]
    fn knn_query_targets_the_embedding_field() {
        let body = knn_query(&[0.5, 0.25], 5);
        assert_eq!(body["query"]["knn"]["embedding"]["k"], 5);
        assert_eq!(body["query"]["knn"]["embedding"]["vector"], json!([0.5, 0.25]));
        assert_eq!(
            body["_source"],
            json!(["path", "line", "title", "heading_path", "text"])
        );
    }
}
