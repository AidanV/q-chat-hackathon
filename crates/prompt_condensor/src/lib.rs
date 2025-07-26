pub mod algorithms;
pub mod metrics;
pub mod validation;

#[cfg(test)]
mod integration_test;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors that can occur during prompt compression
#[derive(Error, Debug)]
pub enum CompressionError {
    #[error("Compression failed: {0}")]
    CompressionFailed(String),
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Semantic similarity too low: {similarity:.2}% (minimum: {minimum:.2}%)")]
    SemanticSimilarityTooLow { similarity: f32, minimum: f32 },
}

/// Configuration for compression strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionConfig {
    /// Minimum semantic similarity threshold (0-100%)
    pub min_similarity_threshold: f32,
    /// Compression strategy to use
    pub strategy: CompressionStrategy,
    /// Whether to enable aggressive compression
    pub aggressive: bool,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            min_similarity_threshold: 85.0,
            strategy: CompressionStrategy::Balanced,
            aggressive: false,
        }
    }
}

/// Available compression strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompressionStrategy {
    /// Conservative compression - prioritizes semantic preservation
    Conservative,
    /// Balanced compression - good mix of compression and preservation
    Balanced,
    /// Aggressive compression - maximizes token reduction
    Aggressive,
}

/// Result of a compression operation
#[derive(Debug, Clone)]
pub struct CompressionResult {
    /// The compressed prompt
    pub compressed_text: String,
    /// Original token count
    pub original_tokens: usize,
    /// Compressed token count
    pub compressed_tokens: usize,
    /// Compression ratio (0-100%)
    pub compression_ratio: f32,
    /// Semantic similarity score (0-100%)
    pub similarity_score: f32,
    /// Energy savings estimation
    pub energy_savings: EnergySavings,
}

/// Energy savings from compression
#[derive(Debug, Clone)]
pub struct EnergySavings {
    /// CO2 savings in grams
    pub co2_grams: f32,
    /// Energy savings in watt-hours
    pub watthours: f32,
    /// Water savings in mL
    pub water_ml: f32,
    /// Cost savings in dollars
    pub dollars: f32,
}

impl CompressionResult {
    pub fn new(
        _original_text: &str,
        compressed_text: String,
        original_tokens: usize,
        compressed_tokens: usize,
        similarity_score: f32,
    ) -> Self {
        let compression_ratio = if original_tokens > 0 && compressed_tokens <= original_tokens {
            ((original_tokens - compressed_tokens) as f32 / original_tokens as f32) * 100.0
        } else {
            0.0
        };

        let tokens_saved = if original_tokens >= compressed_tokens {
            (original_tokens - compressed_tokens) as f32
        } else {
            0.0
        };
        let energy_savings = EnergySavings::from_tokens_saved(tokens_saved);

        Self {
            compressed_text,
            original_tokens,
            compressed_tokens,
            compression_ratio,
            similarity_score,
            energy_savings,
        }
    }
}

impl EnergySavings {
    pub fn from_tokens_saved(tokens_saved: f32) -> Self {
        // Using the same constants as in conversation.rs
        let dollars_per_token = 15.0 / 1_000_000.0;
        let watthours_per_token = 0.001;
        let co2_per_token = 0.4 * watthours_per_token;
        let water_per_token = 1.8 * watthours_per_token;

        Self {
            dollars: dollars_per_token * tokens_saved,
            watthours: watthours_per_token * tokens_saved,
            co2_grams: co2_per_token * tokens_saved,
            water_ml: water_per_token * tokens_saved,
        }
    }
}

/// Main trait for compression algorithms
pub trait PromptCompressor {
    /// Compress a prompt with the given configuration
    fn compress(
        &self,
        prompt: &str,
        config: &CompressionConfig,
    ) -> Result<CompressionResult, CompressionError>;

    /// Get the name of this compression algorithm
    fn name(&self) -> &'static str;

    /// Get the priority of this compressor (lower = higher priority)
    fn priority(&self) -> u8;
}

/// Main compression pipeline that orchestrates multiple compression strategies
pub struct CompressionPipeline {
    compressors: Vec<Box<dyn PromptCompressor + Send + Sync>>,
    config: CompressionConfig,
}

impl CompressionPipeline {
    pub fn new(config: CompressionConfig) -> Self {
        let mut compressors: Vec<Box<dyn PromptCompressor + Send + Sync>> = vec![
            Box::new(algorithms::text_preprocessing::TextPreprocessor::new()),
            Box::new(algorithms::semantic_compression::SemanticCompressor::new()),
        ];

        // Sort by priority (lower number = higher priority)
        compressors.sort_by_key(|c| c.priority());

        Self { compressors, config }
    }

    /// Compress a prompt using the configured pipeline
    pub fn compress(&self, prompt: &str) -> Result<CompressionResult, CompressionError> {
        if prompt.trim().is_empty() {
            return Err(CompressionError::InvalidInput("Empty prompt".to_string()));
        }

        let original_tokens = metrics::count_tokens(prompt);
        let mut current_text = prompt.to_string();
        let mut total_similarity = 100.0;

        // Apply each compressor in priority order
        for compressor in &self.compressors {
            match compressor.compress(&current_text, &self.config) {
                Ok(result) => {
                    current_text = result.compressed_text;
                    // Track cumulative similarity degradation
                    total_similarity = (total_similarity * result.similarity_score) / 100.0;
                    
                    tracing::debug!(
                        "Applied {} compression: {} -> {} tokens ({}% reduction, {}% similarity)",
                        compressor.name(),
                        result.original_tokens,
                        result.compressed_tokens,
                        result.compression_ratio,
                        result.similarity_score
                    );
                }
                Err(e) => {
                    tracing::warn!("Compression failed with {}: {}", compressor.name(), e);
                    // Continue with other compressors
                }
            }
        }

        let compressed_tokens = metrics::count_tokens(&current_text);
        let final_result = CompressionResult::new(
            prompt,
            current_text,
            original_tokens,
            compressed_tokens,
            total_similarity,
        );

        // Check if similarity is acceptable
        if final_result.similarity_score < self.config.min_similarity_threshold {
            return Err(CompressionError::SemanticSimilarityTooLow {
                similarity: final_result.similarity_score,
                minimum: self.config.min_similarity_threshold,
            });
        }

        Ok(final_result)
    }

    /// Update the compression configuration
    pub fn update_config(&mut self, config: CompressionConfig) {
        self.config = config;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compression_pipeline_creation() {
        let config = CompressionConfig::default();
        let pipeline = CompressionPipeline::new(config);
        assert_eq!(pipeline.compressors.len(), 2);
    }

    #[test]
    fn test_energy_savings_calculation() {
        let savings = EnergySavings::from_tokens_saved(100.0);
        assert!(savings.co2_grams > 0.0);
        assert!(savings.watthours > 0.0);
        assert!(savings.water_ml > 0.0);
        assert!(savings.dollars > 0.0);
    }

    #[test]
    fn test_compression_result_creation() {
        let result = CompressionResult::new(
            "original text",
            "compressed".to_string(),
            100,
            80,
            95.0,
        );
        assert_eq!(result.compression_ratio, 20.0);
        assert_eq!(result.original_tokens, 100);
        assert_eq!(result.compressed_tokens, 80);
    }
}
