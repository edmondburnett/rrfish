use rrfish_core::embed::Embedder;
use rrfish_core::opensearch::{Hit, OpenSearch};
use rrfish_core::search::{bm25, knn};

const SIZE: usize = 10;

#[tokio::main]
async fn main() {
    let query = std::env::args().nth(1).expect("usage: search <query>");
    let embedder = Embedder::new("http://lain:8081");
    let opensearch = OpenSearch::new("http://lain:9200", "notes");

    let keyword = bm25(&opensearch, &query, SIZE).await.expect("bm25 search");
    let vector = knn(&opensearch, &embedder, &query, SIZE).await.expect("knn search");

    println!("{:<4}{:<50}kNN", "#", "BM25");
    for rank in 0..SIZE {
        let left = label(keyword.get(rank), &vector);
        let right = label(vector.get(rank), &keyword);
        println!("{:<4}{left:<50}{right}", rank + 1);
    }
}

/// `path:line`, marked with `*` when the other list found the same chunk.
fn label(hit: Option<&Hit>, other: &[Hit]) -> String {
    let Some(hit) = hit else {
        return "-".to_string();
    };
    let shared = other.iter().any(|o| o.id == hit.id);
    let marker = if shared { "*" } else { " " };
    format!("{marker}{}:{}", hit.path, hit.line)
}
