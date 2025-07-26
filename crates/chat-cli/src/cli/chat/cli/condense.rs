use clap::Subcommand;
use crossterm::{
    execute,
    style::{Color, Print, SetForegroundColor},
};

use crate::cli::chat::{ChatError, ChatSession, ChatState};

#[derive(Debug, PartialEq, Subcommand)]
pub enum CondenseSubcommand {
    /// Enable automatic prompt compression to reduce token usage and energy consumption
    Enable,
    /// Disable automatic prompt compression
    Disable,
}

impl CondenseSubcommand {
    pub async fn execute(self, session: &mut ChatSession) -> Result<ChatState, ChatError> {
        match self {
            CondenseSubcommand::Enable => {
                // Set compression enabled in session state
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
                    Print("Your prompts will be automatically compressed before sending to reduce token usage and CO2 emissions.\n"),
                    SetForegroundColor(Color::Reset)
                )?;
            }
            CondenseSubcommand::Disable => {
                // Set compression disabled in session state
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
            }
        }

        Ok(ChatState::PromptUser {
            skip_printing_tools: true,
        })
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Enable => "enable",
            Self::Disable => "disable",
        }
    }
}
