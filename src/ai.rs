use std::sync::LazyLock;

use ollama_rs::{
    Ollama,
    error::OllamaError,
    generation::completion::{GenerationResponseStream, request::GenerationRequest},
};
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio_stream::StreamExt;

const DEF_MODEL: &str = "gemma3n:e4b";

fn ollama_client() -> &'static Ollama {
    static OLLAMA_CLI: LazyLock<Ollama> = LazyLock::new(|| Ollama::default());
    &OLLAMA_CLI
}

// async fn generate_content_once(question: &str) -> Result<String, OllamaError> {
//     let cli = ollama_client();

//     let mut buf = vec![];
//     let mut bw = BufWriter::new(&mut buf);

//     let request = GenerationRequest::new(DEF_MODEL.into(), question.to_string());
//     let mut stream: GenerationResponseStream = cli.generate_stream(request).await?;

//     while let Some(Ok(res)) = stream.next().await {
//         for ele in res {
//             bw.write_all(ele.response.as_bytes()).await?;
//             bw.flush().await?;
//         }
//     }

//     Ok(String::from_utf8_lossy(&buf).to_string())
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[tokio::test]
//     async fn test_generate_content_once() {
//         let q = "我是谁？";
//         let res = generate_content_once(q).await.unwrap();
//         println!("res is\n{}", res)
//     }
// }
