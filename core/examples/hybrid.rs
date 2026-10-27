use rrfish_core::embed::Embedder;
use rrfish_core::index::with_context;
use rrfish_core::opensearch::OpenSearch;
use rrfish_core::rerank::{Reranker, reorder};
use rrfish_core::rrf::{Fused, RRF_K, rrf};
use rrfish_core::search::{bm25, knn};
use std::time::Instant;

/// How many results each retriever returns.
const SIZE: usize = 20;
/// How many fused results the reranker reads.
const CANDIDATES: usize = 20;
/// How many results to show.
const SHOWN: usize = 10;

#[tokio::main]
async fn main() {
    let query = std::env::args().nth(1).expect("usage: hybrid <query>");
    let embedder = Embedder::new("http://lain:8081");
    let opensearch = OpenSearch::new("http://lain:9200", "notes");
    let reranker = Reranker::new("http://lain:8082");

    let start = Instant::now();
    let keyword = bm25(&opensearch, &query, SIZE).await.expect("bm25 search");
    eprintln!("bm25:   {:?}", start.elapsed());

    let start = Instant::now();
    let vector = knn(&opensearch, &embedder, &query, SIZE).await.expect("knn search");
    eprintln!("knn:    {:?}", start.elapsed());

    let candidates: Vec<Fused> = rrf(&keyword, &vector, RRF_K).into_iter().take(CANDIDATES).collect();

    let documents: Vec<String> = candidates
        .iter()
        .map(|f| with_context(&f.hit.title, &f.hit.heading_path, &f.hit.text))
        .collect();

    let start = Instant::now();
    let scores = reranker.scores(&query, &documents).await.expect("rerank");
    eprintln!("rerank: {:?}", start.elapsed());

    let reranked = reorder(candidates, &scores);

    println!("{:<4}{:<4}{:<8}{:>6}  location", "#", "src", "rerank", "fused");
    for (index, entry) in reranked.iter().take(SHOWN).enumerate() {
        println!(
            "{:<4}{:<4}{:<8.3}{:>6}  {}:{}",
            index + 1,
            entry.result.label(),
            entry.rerank_score,
            entry.fused_rank,
            entry.result.hit.path,
            entry.result.hit.line,
        );
    }
}
