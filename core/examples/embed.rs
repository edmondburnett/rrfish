use rrfish_core::embed::Embedder;

#[tokio::main]
async fn main() {
    let base_url = std::env::args().nth(1).unwrap_or("http://lain:8081".to_string());
    let embedder = Embedder::new(&base_url);
    let texts = vec![
        "How do I restart PipeWire?".to_string(),
        "systemctl --user restart pipewire pipewire-pulse".to_string(),
        "A recipe for sourdough bread".to_string(),
    ];
    let embeddings = embedder.embed(&texts).await.expect("embedding request failed");
    println!("{} embeddings, {} dimensions", embeddings.len(), embeddings[0].len());
    println!("question vs answer: {:.3}", cosine(&embeddings[0], &embeddings[1]));
    println!("question vs bread:  {:.3}", cosine(&embeddings[0], &embeddings[2]));
}

fn cosine(a: &[f32], b: &[f32]) -> f32 {
    let mut dot = 0.0;
    let mut norm_a = 0.0;
    let mut norm_b = 0.0;
    for (x, y) in a.iter().zip(b) {
        dot += x * y;
        norm_a += x * x;
        norm_b += y * y;
    }
    dot / (norm_a.sqrt() * norm_b.sqrt())
}
