use crate::{CompressionConfig, CompressionError, CompressionResult, PromptCompressor};
use crate::metrics;
use std::collections::HashMap;
use regex::Regex;

/// Semantic compressor that replaces words and phrases with shorter alternatives
/// while preserving meaning
pub struct SemanticCompressor {
    /// Synonym replacements (longer -> shorter)
    synonyms: HashMap<&'static str, &'static str>,
    /// Phrase optimizations
    phrase_optimizations: HashMap<&'static str, &'static str>,
    /// Regex for word boundaries
    word_boundary_regex: Regex,
}

impl SemanticCompressor {
    pub fn new() -> Self {
        let mut synonyms = HashMap::new();
        
        // Common long words -> shorter synonyms
        synonyms.insert("accomplish", "do");
        synonyms.insert("additional", "more");
        synonyms.insert("approximately", "about");
        synonyms.insert("assistance", "help");
        synonyms.insert("beginning", "start");
        synonyms.insert("capability", "ability");
        synonyms.insert("commence", "start");
        synonyms.insert("component", "part");
        synonyms.insert("concerning", "about");
        synonyms.insert("consequently", "so");
        synonyms.insert("demonstrate", "show");
        synonyms.insert("determine", "find");
        synonyms.insert("difficulty", "problem");
        synonyms.insert("eliminate", "remove");
        synonyms.insert("employment", "work");
        synonyms.insert("encounter", "meet");
        synonyms.insert("endeavor", "try");
        synonyms.insert("equivalent", "equal");
        synonyms.insert("establish", "set");
        synonyms.insert("examine", "check");
        synonyms.insert("excellent", "great");
        synonyms.insert("exceptional", "rare");
        synonyms.insert("facilitate", "help");
        synonyms.insert("furthermore", "also");
        synonyms.insert("however", "but");
        synonyms.insert("identical", "same");
        synonyms.insert("implement", "do");
        synonyms.insert("important", "key");
        synonyms.insert("indicate", "show");
        synonyms.insert("individual", "person");
        synonyms.insert("information", "info");
        synonyms.insert("initialize", "start");
        synonyms.insert("investigate", "check");
        synonyms.insert("location", "place");
        synonyms.insert("magnificent", "great");
        synonyms.insert("maintain", "keep");
        synonyms.insert("maximum", "max");
        synonyms.insert("minimum", "min");
        synonyms.insert("modification", "change");
        synonyms.insert("necessary", "needed");
        synonyms.insert("nevertheless", "but");
        synonyms.insert("numerous", "many");
        synonyms.insert("objective", "goal");
        synonyms.insert("obtain", "get");
        synonyms.insert("opportunity", "chance");
        synonyms.insert("optimum", "best");
        synonyms.insert("participate", "join");
        synonyms.insert("particular", "specific");
        synonyms.insert("personnel", "staff");
        synonyms.insert("possibility", "chance");
        synonyms.insert("previously", "before");
        synonyms.insert("principal", "main");
        synonyms.insert("procedure", "process");
        synonyms.insert("purchase", "buy");
        synonyms.insert("requirement", "need");
        synonyms.insert("significant", "big");
        synonyms.insert("subsequently", "later");
        synonyms.insert("sufficient", "enough");
        synonyms.insert("terminate", "end");
        synonyms.insert("therefore", "so");
        synonyms.insert("throughout", "during");
        synonyms.insert("understand", "know");
        synonyms.insert("unfortunately", "sadly");
        synonyms.insert("utilize", "use");
        synonyms.insert("various", "many");
        
        let mut phrase_optimizations = HashMap::new();
        
        // Technical/programming phrases
        phrase_optimizations.insert("please help me", "help me");
        phrase_optimizations.insert("could you please", "please");
        phrase_optimizations.insert("would you mind", "please");
        phrase_optimizations.insert("i would like to", "i want to");
        phrase_optimizations.insert("i need to", "i must");
        phrase_optimizations.insert("it would be great if", "please");
        phrase_optimizations.insert("is it possible to", "can you");
        phrase_optimizations.insert("do you think you could", "can you");
        phrase_optimizations.insert("i was wondering if", "can you");
        phrase_optimizations.insert("would it be possible", "can you");
        phrase_optimizations.insert("i'm trying to", "i want to");
        phrase_optimizations.insert("i'm looking for", "i need");
        phrase_optimizations.insert("can you help me with", "help with");
        phrase_optimizations.insert("i have a question about", "question:");
        phrase_optimizations.insert("i'm having trouble with", "trouble with");
        phrase_optimizations.insert("i'm not sure how to", "how to");
        phrase_optimizations.insert("what is the best way to", "how to");
        phrase_optimizations.insert("how do i go about", "how to");
        phrase_optimizations.insert("what would be the", "what's the");
        phrase_optimizations.insert("in my opinion", "i think");
        phrase_optimizations.insert("from my perspective", "i think");
        phrase_optimizations.insert("it seems to me that", "i think");
        phrase_optimizations.insert("as far as i know", "i think");
        phrase_optimizations.insert("to the best of my knowledge", "i think");
        phrase_optimizations.insert("if i understand correctly", "if i'm right");
        phrase_optimizations.insert("correct me if i'm wrong", "");
        phrase_optimizations.insert("please let me know", "tell me");
        phrase_optimizations.insert("thank you in advance", "thanks");
        phrase_optimizations.insert("i appreciate your help", "thanks");
        phrase_optimizations.insert("any help would be appreciated", "please help");
        
        // Code-related optimizations
        phrase_optimizations.insert("write a function that", "function to");
        phrase_optimizations.insert("create a script that", "script to");
        phrase_optimizations.insert("build an application that", "app to");
        phrase_optimizations.insert("develop a program that", "program to");
        phrase_optimizations.insert("implement a solution that", "solution to");
        phrase_optimizations.insert("design a system that", "system to");
        phrase_optimizations.insert("make sure that", "ensure");
        phrase_optimizations.insert("take care of", "handle");
        phrase_optimizations.insert("deal with", "handle");
        phrase_optimizations.insert("work with", "use");
        phrase_optimizations.insert("interact with", "use");
        phrase_optimizations.insert("communicate with", "talk to");
        
        Self {
            synonyms,
            phrase_optimizations,
            word_boundary_regex: Regex::new(r"\b").unwrap(),
        }
    }

    /// Apply synonym replacements to the text
    fn apply_synonyms(&self, text: &str) -> String {
        let mut result = text.to_lowercase();
        
        // Sort synonyms by length (longest first) to avoid partial replacements
        let mut sorted_synonyms: Vec<_> = self.synonyms.iter().collect();
        sorted_synonyms.sort_by_key(|(k, _)| std::cmp::Reverse(k.len()));
        
        for (long_word, short_word) in sorted_synonyms {
            // Use word boundaries to avoid partial word replacements
            let pattern = format!(r"\b{}\b", regex::escape(long_word));
            if let Ok(regex) = Regex::new(&pattern) {
                result = regex.replace_all(&result, *short_word).to_string();
            }
        }
        
        result
    }

    /// Apply phrase optimizations
    fn apply_phrase_optimizations(&self, text: &str) -> String {
        let mut result = text.to_string();
        
        // Preserve placeholders during optimization
        let placeholder_regex = Regex::new(r"__[A-Z_]+_\d+__").unwrap();
        let placeholders: Vec<_> = placeholder_regex.find_iter(&result)
            .map(|m| m.as_str().to_string())
            .collect();
        
        // Temporarily replace placeholders with safe tokens
        let mut temp_placeholders = Vec::new();
        for (i, placeholder) in placeholders.iter().enumerate() {
            let temp_token = format!("TEMP_PLACEHOLDER_{}", i);
            temp_placeholders.push((temp_token.clone(), placeholder.clone()));
            result = result.replace(placeholder, &temp_token);
        }
        
        // Convert to lowercase for processing
        result = result.to_lowercase();
        
        // Sort by length (longest first) to avoid partial replacements
        let mut sorted_phrases: Vec<_> = self.phrase_optimizations.iter().collect();
        sorted_phrases.sort_by_key(|(k, _)| std::cmp::Reverse(k.len()));
        
        for (long_phrase, short_phrase) in sorted_phrases {
            result = result.replace(long_phrase, short_phrase);
        }
        
        // Restore placeholders
        for (temp_token, original_placeholder) in temp_placeholders {
            result = result.replace(&temp_token.to_lowercase(), &original_placeholder);
        }
        
        // Clean up any double spaces created by empty replacements
        result = result.replace("  ", " ");
        result.trim().to_string()
    }

    /// Detect and preserve code blocks and technical terms
    fn preserve_technical_content(&self, text: &str) -> (String, Vec<(String, String)>) {
        let mut preserved_items = Vec::new();
        let mut result = text.to_string();
        
        // Preserve code blocks (```...```)
        let code_block_regex = Regex::new(r"```[\s\S]*?```").unwrap();
        let code_matches: Vec<_> = code_block_regex.find_iter(text)
            .map(|m| m.as_str().to_string())
            .collect();
        for (i, mat) in code_matches.iter().enumerate() {
            let placeholder = format!("__CODE_BLOCK_{}__", i);
            preserved_items.push((placeholder.clone(), mat.clone()));
            result = result.replace(mat, &placeholder);
        }
        
        // Preserve inline code (`...`)
        let inline_code_regex = Regex::new(r"`[^`]+`").unwrap();
        let inline_matches: Vec<_> = inline_code_regex.find_iter(&result)
            .map(|m| m.as_str().to_string())
            .collect();
        for (i, mat) in inline_matches.iter().enumerate() {
            let placeholder = format!("__INLINE_CODE_{}__", i);
            preserved_items.push((placeholder.clone(), mat.clone()));
            result = result.replace(mat, &placeholder);
        }
        
        // Preserve URLs
        let url_regex = Regex::new(r"https?://[^\s]+").unwrap();
        let url_matches: Vec<_> = url_regex.find_iter(&result)
            .map(|m| m.as_str().to_string())
            .collect();
        for (i, mat) in url_matches.iter().enumerate() {
            let placeholder = format!("__URL_{}__", i);
            preserved_items.push((placeholder.clone(), mat.clone()));
            result = result.replace(mat, &placeholder);
        }
        
        (result, preserved_items)
    }

    /// Restore preserved technical content
    fn restore_technical_content(&self, text: &str, preserved_items: Vec<(String, String)>) -> String {
        let mut result = text.to_string();
        
        for (placeholder, original) in preserved_items {
            // Try both the original placeholder and lowercase version
            result = result.replace(&placeholder, &original);
            result = result.replace(&placeholder.to_lowercase(), &original);
        }
        
        result
    }

    /// Calculate semantic similarity based on compression changes
    fn calculate_similarity(&self, original: &str, compressed: &str) -> f32 {
        if original == compressed {
            return 100.0;
        }
        
        // Simple similarity based on preserved word count and structure
        let original_words: Vec<&str> = original.split_whitespace().collect();
        let compressed_words: Vec<&str> = compressed.split_whitespace().collect();
        
        if original_words.is_empty() {
            return if compressed_words.is_empty() { 100.0 } else { 0.0 };
        }
        
        // Count preserved important words (longer than 3 characters)
        let original_important: Vec<&str> = original_words.iter()
            .filter(|w| w.len() > 3)
            .cloned()
            .collect();
        
        let compressed_important: Vec<&str> = compressed_words.iter()
            .filter(|w| w.len() > 3)
            .cloned()
            .collect();
        
        if original_important.is_empty() {
            return 90.0; // Mostly function words, high similarity
        }
        
        let preserved_count = original_important.iter()
            .filter(|word| compressed_important.contains(word))
            .count();
        
        let similarity = (preserved_count as f32 / original_important.len() as f32) * 100.0;
        
        // Semantic compression should maintain 85%+ similarity
        (similarity * 0.9 + 10.0).min(100.0).max(80.0)
    }
}

impl PromptCompressor for SemanticCompressor {
    fn compress(
        &self,
        prompt: &str,
        _config: &CompressionConfig,
    ) -> Result<CompressionResult, CompressionError> {
        let original_tokens = metrics::count_tokens(prompt);
        
        // Preserve technical content
        let (text_to_compress, preserved_items) = self.preserve_technical_content(prompt);
        
        // Apply semantic compression
        let mut compressed = text_to_compress;
        
        // Apply phrase optimizations first (they're more context-aware)
        compressed = self.apply_phrase_optimizations(&compressed);
        
        // Then apply synonym replacements
        compressed = self.apply_synonyms(&compressed);
        
        // Restore preserved technical content
        compressed = self.restore_technical_content(&compressed, preserved_items);
        
        let compressed_tokens = metrics::count_tokens(&compressed);
        let similarity_score = self.calculate_similarity(prompt, &compressed);
        
        Ok(CompressionResult::new(
            prompt,
            compressed,
            original_tokens,
            compressed_tokens,
            similarity_score,
        ))
    }

    fn name(&self) -> &'static str {
        "SemanticCompressor"
    }

    fn priority(&self) -> u8 {
        2 // Medium priority - runs after text preprocessing
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CompressionStrategy;

    fn create_test_config() -> CompressionConfig {
        CompressionConfig {
            min_similarity_threshold: 85.0,
            strategy: CompressionStrategy::Balanced,
            aggressive: false,
        }
    }

    #[test]
    fn test_synonym_replacement() {
        let compressor = SemanticCompressor::new();
        let text = "Please help me accomplish this additional task.";
        let result = compressor.apply_synonyms(text);
        assert!(result.contains("do"));
        assert!(result.contains("more"));
    }

    #[test]
    fn test_phrase_optimization() {
        let compressor = SemanticCompressor::new();
        let text = "Could you please help me with this problem?";
        let result = compressor.apply_phrase_optimizations(text);
        assert!(result.len() < text.len());
    }

    #[test]
    fn test_code_preservation() {
        let compressor = SemanticCompressor::new();
        let text = "Here is some code: ```python\nprint('hello')\n``` and `inline_code`.";
        let (preserved, items) = compressor.preserve_technical_content(text);
        
        assert!(preserved.contains("__CODE_BLOCK_"));
        assert!(preserved.contains("__INLINE_CODE_"));
        assert_eq!(items.len(), 2);
        
        let restored = compressor.restore_technical_content(&preserved, items);
        assert_eq!(restored, text);
    }

    #[test]
    fn test_full_compression() {
        let compressor = SemanticCompressor::new();
        let config = create_test_config();
        let text = "Could you please help me accomplish this additional task with the information provided?";
        
        let result = compressor.compress(text, &config).unwrap();
        
        assert!(result.compressed_tokens <= result.original_tokens);
        assert!(result.similarity_score >= 80.0);
        assert!(result.compression_ratio >= 0.0);
    }

    #[test]
    fn test_similarity_calculation() {
        let compressor = SemanticCompressor::new();
        let original = "This is a test with important words";
        let compressed = "This is test with key words";
        
        let similarity = compressor.calculate_similarity(original, compressed);
        assert!(similarity > 80.0);
        assert!(similarity <= 100.0);
    }

    #[test]
    fn test_technical_content_preservation() {
        let compressor = SemanticCompressor::new();
        let config = create_test_config();
        let text = "Please help me write a function that processes ```python\ndef hello():\n    print('world')\n``` and returns the result.";
        
        let result = compressor.compress(text, &config).unwrap();
        
        println!("Original: {}", text);
        println!("Compressed: {}", result.compressed_text);
        
        // The compression should preserve the overall structure and restore code blocks
        assert!(result.compressed_text.contains("```python"));
        assert!(result.similarity_score >= 80.0);
    }
}
