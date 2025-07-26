# Amazon Q CLI Prompt Condensor - Live Compression Feature Spec

## Overview
Build a real-time prompt compression feature that reduces token count while preserving semantic meaning, demonstrating energy savings through visual feedback.

## Feature Requirements

### Core Functionality
- **Real-time compression**: Process prompts as user types with <500ms latency
- **Token count reduction**: Target 20-40% reduction without semantic loss
- **Energy impact visualization**: Show estimated CO2 savings in real-time
- **Semantic preservation**: Maintain prompt intent and effectiveness

### User Interface
- **Split-pane editor**: Original prompt (left) → Compressed prompt (right)
- **Live metrics dashboard**: Token count, compression ratio, estimated energy savings
- **Compression quality indicator**: Semantic similarity score (0-100%)
- **One-click copy**: Easy export of compressed prompts

## Implementation Steps

### Phase 1: Core Infrastructure
1. **Set up compression engine**
   - Create `prompt_condensor` crate in `/crates/`
   - Implement trait-based compression pipeline
   - Add configuration for compression strategies

2. **Token counting integration**
   - Integrate tiktoken or similar tokenizer
   - Create before/after token counting utilities
   - Add energy estimation calculations (tokens → CO2)

3. **Slash command integration**
   ```bash
   /condense enable    # Enable auto-compression for all prompts
   /condense disable   # Disable compression
   ```
   - Integrate with existing chat command parser
   - Add compression toggle to user session state
   - Show compression metrics in chat interface before sending

### Phase 2: Compression Algorithms
4. **Text preprocessing**
   - Remove redundant whitespace and formatting
   - Normalize punctuation and capitalization
   - Strip unnecessary articles/prepositions

5. **Semantic compression**
   - Implement abbreviation replacement dictionary
   - Add synonym substitution with shorter alternatives
   - Create phrase-to-concise mapping ("in order to" → "to")

6. **Advanced compression**
   - Integrate sentence-transformers for semantic deduplication
   - Add dependency parsing for context-aware word removal
   - Implement instruction template optimization

### Phase 3: Live Interface
7. **Real-time processing pipeline**
   - Debounced input handling (300ms delay)
   - Async compression processing
   - Progressive enhancement (fast → slow algorithms)

8. **Interactive terminal UI**
   - Use `ratatui` for split-pane interface
   - Real-time metrics display
   - Syntax highlighting for prompts

9. **Visual feedback system**
   - Token count animations
   - Compression ratio progress bars
   - Energy savings counter with CO2 units
   - You can use existing `get_usage_stats_from_tokens` method to find impact

### Phase 4: Quality Assurance
10. **Semantic similarity validation**
    - Implement embedding-based similarity scoring
    - Add quality threshold warnings
    - Create rollback mechanism for poor compressions

11. **Compression strategy selection**
    - Auto-detect prompt type (code, natural language, structured)
    - Apply appropriate compression techniques
    - Allow manual strategy override

### Phase 5: Integration & Polish
12. **Amazon Q CLI integration**
    - Add condensor as slash command functionality
    - Integrate with existing usage tracking
    - Connect to sustainability metrics

13. **Performance optimization**
    - Cache compression results
    - Optimize for common prompt patterns
    - Add compression presets (aggressive, balanced, conservative)

14. **Demo-ready features**
    - Export compression reports
    - Batch processing mode
    - Integration with `q usage` commands

## Technical Architecture

### File Structure
```
crates/prompt_condensor/
├── src/
│   ├── lib.rs                 # Main compression traits
│   ├── algorithms/            # Compression implementations
│   │   ├── text_preprocessing.rs
│   │   ├── semantic_compression.rs
│   │   └── template_optimization.rs
│   ├── metrics/               # Token counting & energy calc
│   ├── ui/                    # Terminal interface
│   └── validation/            # Quality assurance
└── tests/
```

### Key Dependencies
- `tiktoken` - Token counting
- `sentence-transformers` (Python binding) - Semantic analysis
- `ratatui` - Terminal UI
- `tokio` - Async processing
- `serde` - Configuration management

## Success Metrics
- **Compression ratio**: 20-40% token reduction
- **Processing speed**: <500ms for typical prompts
- **Semantic preservation**: >85% similarity score
- **Energy savings**: Measurable CO2 reduction calculations

## Demo Flow
1. User types `/condense enable` in Amazon Q chat
2. System shows: "✅ Prompt compression enabled - saving energy with every message!"
3. User types their next prompt
4. Before sending, system shows compression preview in other pane:
   ```
   Original: 156 tokens | Compressed: 98 tokens | 37% reduction | ~2.1g CO2 saved
   [Preview compressed prompt here]
   ```
5. When user submits the prompt energy savings added to their sustainability metrics