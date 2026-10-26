use std::path::Path;

use rrfish_core::embed::Embedder;
use rrfish_core::index::{IndexDoc, embed_text, index_docs};
use rrfish_core::note::read_note;
use rrfish_core::opensearch::OpenSearch;
use rrfish_core::walk::discover;

const BATCH_SIZE: usize = 32;

#[tokio::main]
async fn main() {
    let root = std::env::args().nth(1).expect("usage: index <notes-dir>");
    let embedder = Embedder::new("http://lain:8081");
    let opensearch = OpenSearch::new("http://lain:9200", "notes");

    let docs = collect_docs(Path::new(&root));
    println!("{} chunks to index", docs.len());

    opensearch.recreate_index().await.expect("recreate index");
    for (number, batch) in docs.chunks(BATCH_SIZE).enumerate() {
        let texts: Vec<String> = batch.iter().map(embed_text).collect();
        let embeddings = embedder.embed(&texts).await.expect("embed batch");
        let response = opensearch.bulk_index(batch, &embeddings).await.expect("bulk request");
        if response.errors {
            eprintln!("batch {number}: some documents failed to index");
        }
        println!("indexed {}/{}", number * BATCH_SIZE + batch.len(), docs.len());
    }
    opensearch.refresh().await.expect("refresh");
}

/// Every chunk of every note under `root`, ready to index.
fn collect_docs(root: &Path) -> Vec<IndexDoc> {
    let mut docs = Vec::new();
    for file in discover(root) {
        match read_note(&file) {
            Ok(note) => docs.extend(index_docs(&file, &note)),
            Err(error) => eprintln!("skipping {}: {error}", file.rel_path.display()),
        }
    }
    docs
}
