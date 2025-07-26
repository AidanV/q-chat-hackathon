# Implementation Guide: `/usage offset` Command

## Overview
Add a new `/usage offset` command that provides personalized suggestions for offsetting the user's daily CO2 emissions from Amazon Q usage. This command will follow similar patterns to `/usage summarize` but focus on actionable environmental offset recommendations.

## Implementation Steps

### 1. Command Structure Setup
- **Location**: Add to existing usage command module (likely in `crates/chat_cli/src/commands/usage.rs` or similar)
- **Command**: `/usage offset`

### 2. Reuse Existing UsageStatistics
- Use the same `UsageStatistics` struct that `/usage summarize` uses
- No need for new data structures - leverage existing usage tracking

### 3. Simple Prompt Generation
```rust
fn generate_offset_prompt(stats: &UsageStatistics) -> String {
    format!(
        "Based on my Amazon Q usage statistics: {},
        please provide 3-5 creative, actionable, and realistic suggestions for offsetting
        my environmental impact today. Make suggestions:
        - Specific and measurable
        - Achievable within a day
        - Relatable to a developer's lifestyle
        - Include both digital and physical actions
        - Show approximate CO2 offset amounts when relevant

        Format as a friendly, encouraging list with emojis.",
        serde_json::to_string_pretty(stats).unwrap_or_default()
    )
}
```

### 4. Command Implementation

#### Step 4.1: Add Command Handler
```rust
// In usage command module - mirror the summarize command pattern
pub async fn handle_offset_command(
    context: &CommandContext,
    args: &OffsetArgs,
) -> Result<(), CliError> {
    // 1. Get existing usage statistics (same as summarize)
    let stats = get_usage_statistics()?;

    // 2. Generate offset suggestions prompt
    let prompt = generate_offset_prompt(&stats);

    // 3. Send to Q for personalized suggestions (same as summarize)
    let response = send_chat_request(context, &prompt).await?;

    // 4. Display response (same as summarize)
    println!("{}", response);

    Ok(())
}
```

#### Step 4.2: Add to Command Parser
```rust
// In main command parsing logic
match usage_subcommand {
    "summary" => handle_summary_command(context, args).await,
    "graph" => handle_graph_command(context, args).await,
    "offset" => handle_offset_command(context, args).await,
    _ => show_usage_help(),
}
```

### 8. Implementation Checklist
- [ ] Add command parsing logic for "offset"
- [ ] Create simple prompt generation function
- [ ] Reuse existing UsageStatistics and display pattern

## Notes
- **Keep it simple**: Mirror the `/usage summarize` implementation exactly
- **Reuse existing code**: No new data structures or complex logic needed
- **Same display pattern**: Just print the AI response directly like summarize does
