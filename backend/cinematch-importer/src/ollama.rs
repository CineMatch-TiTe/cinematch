use anyhow::Result;
use ollama_rs::Ollama;
use ollama_rs::generation::embeddings::request::{EmbeddingsInput, GenerateEmbeddingsRequest};
use std::sync::Arc;

const EMBED_MODEL: &str = "bge-m3";

pub struct OllamaService {
    client: Arc<Ollama>,
}

impl Clone for OllamaService {
    fn clone(&self) -> Self {
        Self {
            client: Arc::clone(&self.client),
        }
    }
}

impl OllamaService {
    pub fn new(host: &str, port: u16) -> Self {
        let url = format!("http://{}", host);
        let client = Ollama::new(url, port);
        Self {
            client: Arc::new(client),
        }
    }

    pub async fn check_service(&self) -> Result<()> {
        self.client
            .list_local_models()
            .await
            .map(|_| ())
            .map_err(|e| anyhow::anyhow!("Failed to connect to Ollama: {}", e))?;

        self.pull_model(EMBED_MODEL).await?;

        Ok(())
    }

    pub async fn pull_model(&self, model_name: &str) -> Result<()> {
        self.client
            .pull_model(model_name.to_string(), false)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to pull model {}: {}", model_name, e))?;

        Ok(())
    }

    /// Generate embeddings for a batch of texts.
    ///
    /// Non-finite values (NaN, ±Inf) are sanitised to prevent JSON
    /// serialisation errors when uploading to Qdrant.
    pub async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        let request = GenerateEmbeddingsRequest::new(
            EMBED_MODEL.to_string(),
            EmbeddingsInput::Multiple(texts.into()),
        );

        let mut response = self
            .client
            .generate_embeddings(request)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to generate batch embeddings: {}", e))?;

        // Sanitize: replace NaN → 0.0, ±Inf → ±1.0
        for embedding in &mut response.embeddings {
            for value in embedding.iter_mut() {
                if !value.is_finite() {
                    *value = if value.is_nan() {
                        0.0
                    } else if *value > 0.0 {
                        1.0
                    } else {
                        -1.0
                    };
                }
            }
        }

        Ok(response.embeddings)
    }
}
