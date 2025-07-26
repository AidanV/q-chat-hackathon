We are going to add a slash command "/usage graph" that will display the usage data for the past month as an aesthetically pleasing graph

### Phase 1: Add the needed Dependencies

File: /Users/aidanvd/Documents/amazon-q-developer-cli/crates/chat-cli/Cargo.toml

Action: Add the following dependencies (these are the tools we will use to create this graph):
toml
ratatui = "0.24"
plotters = "0.3"
plotters-backend = "0.3"

### Phase 2: Add the "/usage graph"

Add the "/usage graph" command

### Phase 3: Create Graph Module using ratatui and plotters

In this File: /Users/aidanvd/Documents/amazon-q-developer-cli/crates/chat-cli/src/database/mod.rs
Use the method:
get_last_month_usage(&self) -> Result<Vec<(u32, UsageStatistics)>, DatabaseError>
to get the data for the graph