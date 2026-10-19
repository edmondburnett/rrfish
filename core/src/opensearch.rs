use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::index::IndexDoc;

/// The index settings and mapping, compiled into the binary.
const MAPPING: &str = include_str!("../mapping.json");

/// Minimal OpenSearch client for one index.
pub struct OpenSearch {
    client: reqwest::Client,
    index_url: String,
}

/// What `_bulk` returns; `errors` is true if any single document failed.
#[derive(Debug, Deserialize)]
pub struct BulkResponse {
    pub errors: bool,
}

/// A document as stored: every `IndexDoc` field, plus its embedding.
#[derive(Serialize)]
struct StoredDoc<'a> {
    #[serde(flatten)]
    doc: &'a IndexDoc,
    embedding: &'a [f32],
}

impl OpenSearch {
    /// `base_url` is the cluster root, e.g. `http://lain:9200`.
    pub fn new(base_url: &str, index: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            index_url: format!("{base_url}/{index}"),
        }
    }

    /// Delete the index if it exists, then create it from `mapping.json`.
    pub async fn recreate_index(&self) -> reqwest::Result<()> {
        self.client.delete(&self.index_url).send().await?;
        self.client
            .put(&self.index_url)
            .header("Content-Type", "application/json")
            .body(MAPPING)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    /// Index a batch of documents with their embeddings in one `_bulk` request.
    pub async fn bulk_index(&self, docs: &[IndexDoc], embeddings: &[Vec<f32>]) -> reqwest::Result<BulkResponse> {
        self.client
            .post(format!("{}/_bulk", self.index_url))
            .header("Content-Type", "application/x-ndjson")
            .body(bulk_body(docs, embeddings))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }

    /// Make everything indexed so far visible to search.
    pub async fn refresh(&self) -> reqwest::Result<()> {
        self.client
            .post(format!("{}/_refresh", self.index_url))
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }
}

/// NDJSON for `_bulk`: an action line, then the document line, for each doc.
fn bulk_body(docs: &[IndexDoc], embeddings: &[Vec<f32>]) -> String {
    let mut body = String::new();
    for (doc, embedding) in docs.iter().zip(embeddings) {
        let action_line = json!({ "index": { "_id": doc.id } }).to_string();
        let doc_line = serde_json::to_string(&StoredDoc { doc, embedding }).expect("IndexDoc always serializes");
        body += &format!("{action_line}\n{doc_line}\n");
    }
    body
}

#[cfg(test)]
mod tests {
    use super::*;

    fn doc(id: &str) -> IndexDoc {
        IndexDoc {
            id: id.to_string(),
            path: "a.md".to_string(),
            collection: None,
            format: "md",
            title: "A".to_string(),
            tags: vec![],
            heading_path: vec![],
            line: 1,
            text: "hello".to_string(),
            code_langs: vec![],
        }
    }

    #[test]
    fn bulk_body_is_action_then_document_per_line() {
        let body = bulk_body(&[doc("a.md#0")], &[vec![0.5, 1.0]]);
        let lines: Vec<&str> = body.lines().collect();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], r#"{"index":{"_id":"a.md#0"}}"#);
        let stored: serde_json::Value = serde_json::from_str(lines[1]).unwrap();
        assert_eq!(stored["id"], "a.md#0");
        assert_eq!(stored["text"], "hello");
        assert_eq!(stored["embedding"], serde_json::json!([0.5, 1.0]));
        assert!(body.ends_with('\n'));
    }

    #[test]
    fn mapping_is_valid_json_with_the_vector_field() {
        let mapping: serde_json::Value = serde_json::from_str(MAPPING).unwrap();
        assert_eq!(mapping["mappings"]["properties"]["embedding"]["dimension"], 1024);
    }
}
