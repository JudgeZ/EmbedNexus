//! Embedding orchestration placeholders.

use blake3::Hasher;
use ingestion_sanitization::SanitizedChunk;
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct EmbeddingConfig {
    pub encoder_id: String,
    pub dimensions: usize,
}

impl EmbeddingConfig {
    pub fn new(encoder_id: String, dimensions: usize) -> Self {
        Self {
            encoder_id,
            dimensions,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EmbeddingBatch {
    pub encoder_id: String,
    pub vectors: Vec<Vec<f32>>,
    pub compression_fingerprint: String,
}

#[derive(Debug, Error)]
pub enum EmbeddingError {
    #[error("embedding dimensions must be non-zero")]
    InvalidDimensions,
}

#[derive(Debug, Clone)]
pub struct EmbeddingGenerator {
    config: EmbeddingConfig,
}

impl EmbeddingGenerator {
    pub fn new(config: EmbeddingConfig) -> Self {
        Self { config }
    }

    pub fn encode(&self, chunks: &[SanitizedChunk]) -> Result<EmbeddingBatch, EmbeddingError> {
        if self.config.dimensions == 0 {
            return Err(EmbeddingError::InvalidDimensions);
        }
        let mut vectors = Vec::with_capacity(chunks.len());
        let mut fingerprint_hasher = Hasher::new();
        for chunk in chunks {
            let vector = self.vector_for_chunk(chunk);
            fingerprint_hasher.update(chunk.plan_id.as_bytes());
            fingerprint_hasher.update(chunk.scrubbed_payload.as_bytes());
            vectors.push(vector);
        }
        let fingerprint = fingerprint_hasher.finalize().to_hex().to_string();
        Ok(EmbeddingBatch {
            encoder_id: self.config.encoder_id.clone(),
            vectors,
            compression_fingerprint: format!("comp:{}", fingerprint),
        })
    }

    fn vector_for_chunk(&self, chunk: &SanitizedChunk) -> Vec<f32> {
        let mut hasher = Hasher::new();
        hasher.update(chunk.plan_id.as_bytes());
        hasher.update(chunk.scrubbed_payload.as_bytes());
        let hash = hasher.finalize();
        let bytes = hash.as_bytes();
        let mut vector = Vec::with_capacity(self.config.dimensions);
        let mut idx = 0usize;
        while vector.len() < self.config.dimensions {
            let b1 = bytes[idx % bytes.len()] as u16;
            let b2 = bytes[(idx + 1) % bytes.len()] as u16;
            let combined = (b1 << 8) | b2;
            vector.push(combined as f32 / u16::MAX as f32);
            idx += 2;
        }
        vector
    }
}
