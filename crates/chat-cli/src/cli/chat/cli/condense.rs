use clap::Subcommand;
use crossterm::{
    execute,
    style::{Color, Print, SetForegroundColor},
};

use crate::cli::chat::{ChatError, ChatSession, ChatState};

#[derive(Debug, PartialEq, Subcommand)]
pub enum CondenseSubcommand {
    /// Enable automatic prompt compression to reduce token usage and energy consumption
    Enable {
        /// Compression strategy: conservative, balanced, or aggressive
        #[arg(long, default_value = "balanced")]
        strategy: String,
        /// Minimum similarity threshold (0-100%)
        #[arg(long, default_value = "75.0")]
        min_similarity: f32,
    },
    /// Disable automatic prompt compression
    Disable,
    /// Show current compression status and statistics
    Status,
}

impl CondenseSubcommand {
    pub async fn execute(self, session: &mut ChatSession) -> Result<ChatState, ChatError> {
        match self {
            CondenseSubcommand::Enable {
                strategy,
                min_similarity,
            } => {
                // Parse and validate strategy
                let compression_strategy = match strategy.to_lowercase().as_str() {
                    "conservative" => prompt_condensor::CompressionStrategy::Conservative,
                    "balanced" => prompt_condensor::CompressionStrategy::Balanced,
                    "aggressive" => prompt_condensor::CompressionStrategy::Aggressive,
                    _ => {
                        execute!(
                            session.stderr,
                            SetForegroundColor(Color::Red),
                            Print("❌ Invalid strategy. Use: conservative, balanced, or aggressive\n"),
                            SetForegroundColor(Color::Reset)
                        )?;
                        return Ok(ChatState::PromptUser {
                            skip_printing_tools: true,
                        });
                    },
                };

                // Validate similarity threshold
                if min_similarity < 0.0 || min_similarity > 100.0 {
                    execute!(
                        session.stderr,
                        SetForegroundColor(Color::Red),
                        Print("❌ Similarity threshold must be between 0 and 100\n"),
                        SetForegroundColor(Color::Reset)
                    )?;
                    return Ok(ChatState::PromptUser {
                        skip_printing_tools: true,
                    });
                }

                // Update compression configuration
                let config = prompt_condensor::CompressionConfig {
                    min_similarity_threshold: min_similarity,
                    strategy: compression_strategy.clone(),
                    aggressive: matches!(compression_strategy, prompt_condensor::CompressionStrategy::Aggressive),
                };

                session.update_compression_config(config);
                session.set_compression_enabled(true);

                execute!(
                    session.stderr,
                    SetForegroundColor(Color::Green),
                    Print("✅ Prompt compression enabled - saving energy with every message!\n"),
                    SetForegroundColor(Color::Reset)
                )?;

                execute!(
                    session.stderr,
                    SetForegroundColor(Color::DarkGrey),
                    Print(format!(
                        "📊 Strategy: {} | Min similarity: {:.1}%\n",
                        strategy, min_similarity
                    )),
                    Print(
                        "Your prompts will be automatically compressed before sending to reduce token usage and CO2 emissions.\n"
                    ),
                    SetForegroundColor(Color::Reset)
                )?;
            },
            CondenseSubcommand::Disable => {
                session.set_compression_enabled(false);

                execute!(
                    session.stderr,
                    SetForegroundColor(Color::Yellow),
                    Print("⚠️  Prompt compression disabled\n"),
                    SetForegroundColor(Color::Reset)
                )?;

                execute!(
                    session.stderr,
                    SetForegroundColor(Color::DarkGrey),
                    Print("Your prompts will be sent without compression.\n"),
                    SetForegroundColor(Color::Reset)
                )?;
            },
            CondenseSubcommand::Status => {
                if session.is_compression_enabled() {
                    let stats = session.get_compression_stats();
                    let total_prompts = stats.total_prompts;
                    let total_original_tokens = stats.total_original_tokens;
                    let total_compressed_tokens = stats.total_compressed_tokens;
                    let total_co2_saved = stats.total_co2_saved;
                    let total_energy_saved = stats.total_energy_saved;
                    let total_cost_saved = stats.total_cost_saved;

                    let total_reduction = if total_original_tokens > 0 {
                        ((total_original_tokens - total_compressed_tokens) as f32 / total_original_tokens as f32)
                            * 100.0
                    } else {
                        0.0
                    };

                    execute!(
                        session.stderr,
                        SetForegroundColor(Color::Green),
                        Print("✅ Compression: ENABLED\n"),
                        SetForegroundColor(Color::DarkGrey),
                        Print("┌─ Session Statistics ─────────────────────────────────────────────\n"),
                        Print(format!("│ 📊 Prompts compressed: {}\n", total_prompts)),
                        Print(format!(
                            "│ 🔢 Tokens: {} → {} ({:.1}% reduction)\n",
                            total_original_tokens, total_compressed_tokens, total_reduction
                        )),
                        Print(format!("│ 💚 CO2 saved: {:.2}g\n", total_co2_saved)),
                        Print(format!("│ ⚡ Energy saved: {:.3}Wh\n", total_energy_saved)),
                        Print(format!("│ 💰 Cost saved: ${:.4}\n", total_cost_saved)),
                        Print("└─────────────────────────────────────────────────────────────────\n"),
                        Print("💡 Use '/condense disable' to turn off compression.\n"),
                        SetForegroundColor(Color::Reset)
                    )?;
                } else {
                    execute!(
                        session.stderr,
                        SetForegroundColor(Color::Yellow),
                        Print("⚠️  Compression: DISABLED\n"),
                        SetForegroundColor(Color::DarkGrey),
                        Print("💡 Use '/condense enable' to start saving energy with compressed prompts.\n"),
                        SetForegroundColor(Color::Reset)
                    )?;
                }
            },
        }

        Ok(ChatState::PromptUser {
            skip_printing_tools: true,
        })
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Enable { .. } => "enable",
            Self::Disable => "disable",
            Self::Status => "status",
        }
    }
}
