use serde::{Deserialize, Serialize};

/// Client for an OpenAI-compatible `/v1/embeddings` endpoint (llama-server).
pub struct Embedder {
    client: reqwest::Client,
    url: String,
}

#[derive(Serialize)]
struct EmbedRequest<'a> {
    input: &'a [String],
}

#[derive(Deserialize)]
struct EmbedResponse {
    data: Vec<EmbedItem>,
}

#[derive(Deserialize)]
struct EmbedItem {
    index: usize,
    embedding: Vec<f32>,
}

impl Embedder {
    pub fn new(base_url: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            url: format!("{base_url}/v1/embeddings"),
        }
    }

    /// Embed a batch of texts; the result is in the same order as `texts`.
    pub async fn embed(&self, texts: &[String]) -> reqwest::Result<Vec<Vec<f32>>> {
        let request = EmbedRequest { input: texts };
        let response: EmbedResponse = self
            .client
            .post(&self.url)
            .json(&request)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        Ok(in_input_order(response))
    }
}

/// The server tags each embedding with the index of its input; put them back in that order.
fn in_input_order(mut response: EmbedResponse) -> Vec<Vec<f32>> {
    response.data.sort_by_key(|item| item.index);
    let mut embeddings: Vec<Vec<f32>> = Vec::with_capacity(response.data.len());
    for r in response.data {
        embeddings.push(r.embedding);
    }
    embeddings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embeddings_come_back_in_input_order() {
        let json = r#"{"data": [
            {"index": 1, "embedding": [0.0, 1.0], "object": "embedding"},
            {"index": 0, "embedding": [1.0, 0.0], "object": "embedding"}
        ], "model": "x"}"#;
        let response: EmbedResponse = serde_json::from_str(json).unwrap();
        assert_eq!(in_input_order(response), [vec![1.0, 0.0], vec![0.0, 1.0]]);
    }

    #[test]
    fn request_body_is_openai_shaped() {
        let texts = vec!["a".to_string(), "b".to_string()];
        let body = serde_json::to_string(&EmbedRequest { input: &texts }).unwrap();
        assert_eq!(body, r#"{"input":["a","b"]}"#);
    }
}
