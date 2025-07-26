/// Simple token counting using character-based estimation
/// This matches the approach used in conversation.rs
pub fn count_tokens(text: &str) -> usize {
    let tokens_per_character: f32 = 0.25;
    (tokens_per_character * text.len() as f32) as usize
}

/// More accurate token counting (placeholder for future tiktoken integration)
pub fn count_tokens_accurate(text: &str) -> usize {
    // TODO: Integrate with tiktoken for more accurate counting
    // For now, use the same estimation as the main codebase
    count_tokens(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_counting() {
        let text = "Hello, world!";
        let tokens = count_tokens(text);
        assert!(tokens > 0);
        assert_eq!(tokens, (text.len() as f32 * 0.25) as usize);
    }

    #[test]
    fn test_empty_text() {
        assert_eq!(count_tokens(""), 0);
    }

    #[test]
    fn test_accurate_counting_fallback() {
        let text = "This is a test";
        assert_eq!(count_tokens_accurate(text), count_tokens(text));
    }
}
