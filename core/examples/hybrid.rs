use rrfish_core::embed::Embedder;
use rrfish_core::opensearch::OpenSearch;
use rrfish_core::rrf::{RRF_K, rrf};
use rrfish_core::search::{bm25, knn};

const SIZE: usize = 10;

#[tokio::main]
async fn main() {
    let query = std::env::args().nth(1).expect("usage: hybrid <query>");
    let embedder = Embedder::new("http://lain:8081");
    let opensearch = OpenSearch::new("http://lain:9200", "notes");

    let keyword = bm25(&opensearch, &query, SIZE).await.expect("bm25 search");
    let vector = knn(&opensearch, &embedder, &query, SIZE).await.expect("knn search");
    let fused = rrf(&keyword, &vector, RRF_K);

    println!("{:<4}{:<4}{:<10}{:>5}{:>5}  location", "#", "src", "rrf", "bm25", "knn");
    for (index, result) in fused.iter().take(SIZE).enumerate() {
        println!(
            "{:<4}{:<4}{:<10.5}{:>5}{:>5}  {}:{}",
            index + 1,
            result.label(),
            result.score,
            rank_text(result.bm25_rank),
            rank_text(result.knn_rank),
            result.hit.path,
            result.hit.line,
        );
    }
}

fn rank_text(rank: Option<usize>) -> String {
    match rank {
        Some(rank) => rank.to_string(),
        None => "-".to_string(),
    }
}
