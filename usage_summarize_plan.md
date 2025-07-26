We are going to add a slash command "/usage summarize" that will query the connected AI model to provide a funny equivalent amount for one of the UsageStatistics fields

### Phase 1: Add the "summarize" subcommand to UsageSubcommand

File: /Users/aidanvd/Documents/amazon-q-developer-cli/crates/chat-cli/src/cli/chat/cli/usage.rs

Action: Add the following subcommand to the `UsageSubcommand` enum:
```rust
/// Ask the AI model to provide a funny equivalent for one of your usage statistics
Summarize,
```

### Phase 2: Implement the execute_summarize method

In the same file: /Users/aidanvd/Documents/amazon-q-developer-cli/crates/chat-cli/src/cli/chat/cli/usage.rs

Action: Add a new method `execute_summarize` that will:
1. Get the current day's usage statistics using `session.conversation.usage_summary(os)`
2. Select one of the four fields (dollars, watthours, co2, water) randomly
3. Create a prompt asking the AI model for a funny equivalent amount
4. Send this prompt to the connected AI model and display the response

### Phase 3: Update the execute method to handle the new subcommand

In the same file: /Users/aidanvd/Documents/amazon-q-developer-cli/crates/chat-cli/src/cli/chat/cli/usage.rs

Action: Update the `execute` method to include:
```rust
Some(UsageSubcommand::Summarize) => self.execute_summarize(os, session).await,
```

### Phase 4: Update the subcommand_name method

In the same file: /Users/aidanvd/Documents/amazon-q-developer-cli/crates/chat-cli/src/cli/chat/cli/usage.rs

Action: Update the `subcommand_name` method to include:
```rust
Some(UsageSubcommand::Summarize) => Some("summarize"),
```

### Phase 5: Implementation Details for execute_summarize

The `execute_summarize` method should:

1. Get today's usage statistics:
```rust
let usage_summary = session.conversation.usage_summary(os);
```

2. Select the most interesting field (highest value or random selection):
```rust
// Find the field with the highest value or select randomly
let (field_name, field_value, field_unit) = if usage_summary.dollars > 0.0 {
    ("cost", usage_summary.dollars as f64, "dollars")
} else if usage_summary.watthours > 0.0 {
    ("energy", usage_summary.watthours as f64, "watt-hours")
} else if usage_summary.co2 > 0.0 {
    ("CO2 emissions", usage_summary.co2 as f64, "grams")
} else if usage_summary.water > 0.0 {
    ("water usage", usage_summary.water as f64, "milliliters")
} else {
    // No usage data available
    return Ok(ChatState::PromptUser { skip_printing_tools: true });
};
```

3. Create a humorous prompt for the AI:
```rust
let prompt = format!(
    "I've used {:.2} {} worth of {} today with Amazon Q. Can you give me a funny, creative, and relatable equivalent amount? For example, 'That's like buying X cups of coffee' or 'That's equivalent to Y bananas' or something similarly amusing and easy to visualize. Keep it light and fun!",
    field_value, field_unit, field_name
);
```

4. Send the prompt to the AI model and handle the response by creating a new chat state that processes this as a regular user input.

### Phase 6: Integration with existing chat flow

The method should return a `ChatState::HandleInput` with the generated prompt, allowing the existing chat infrastructure to process the AI query and display the response naturally within the conversation flow.
