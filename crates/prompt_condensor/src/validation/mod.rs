use crate::{CompressionResult, CompressionError};

/// Validates compression results to ensure quality
pub struct CompressionValidator {
    min_similarity_threshold: f32,
}

impl CompressionValidator {
    pub fn new(min_similarity_threshold: f32) -> Self {
        Self {
            min_similarity_threshold,
        }
    }

    /// Validate a compression result
    pub fn validate(&self, result: &CompressionResult) -> Result<(), CompressionError> {
        // Check similarity threshold
        if result.similarity_score < self.min_similarity_threshold {
            return Err(CompressionError::SemanticSimilarityTooLow {
                similarity: result.similarity_score,
                minimum: self.min_similarity_threshold,
            });
        }

        // Check for reasonable compression (not negative)
        if result.compressed_tokens > result.original_tokens {
            return Err(CompressionError::CompressionFailed(
                "Compression resulted in more tokens than original".to_string(),
            ));
        }

        // Check for empty result
        if result.compressed_text.trim().is_empty() {
            return Err(CompressionError::CompressionFailed(
                "Compression resulted in empty text".to_string(),
            ));
        }

        Ok(())
    }

    /// Suggest rollback if compression quality is poor
    pub fn should_rollback(&self, result: &CompressionResult) -> bool {
        // Rollback if similarity is too low or compression ratio is minimal
        result.similarity_score < self.min_similarity_threshold || 
        result.compression_ratio < 5.0 // Less than 5% compression might not be worth it
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CompressionResult;

    #[test]
    fn test_validation_success() {
        let validator = CompressionValidator::new(85.0);
        let result = CompressionResult::new(
            "original text",
            "compressed".to_string(),
            100,
            80,
            90.0,
        );
        
        assert!(validator.validate(&result).is_ok());
        assert!(!validator.should_rollback(&result));
    }

    #[test]
    fn test_validation_low_similarity() {
        let validator = CompressionValidator::new(85.0);
        let result = CompressionResult::new(
            "original text",
            "compressed".to_string(),
            100,
            80,
            70.0, // Below threshold
        );
        
        assert!(validator.validate(&result).is_err());
        assert!(validator.should_rollback(&result));
    }

    #[test]
    fn test_validation_negative_compression() {
        let validator = CompressionValidator::new(85.0);
        let result = CompressionResult::new(
            "original text",
            "much longer compressed text that is worse".to_string(),
            100,
            120, // More tokens than original
            95.0,
        );
        
        assert!(validator.validate(&result).is_err());
    }

    #[test]
    fn test_minimal_compression_rollback() {
        let validator = CompressionValidator::new(85.0);
        let result = CompressionResult::new(
            "original text",
            "original tex".to_string(), // Minimal compression
            100,
            98, // Only 2% compression
            95.0,
        );
        
        assert!(validator.should_rollback(&result));
    }
}
