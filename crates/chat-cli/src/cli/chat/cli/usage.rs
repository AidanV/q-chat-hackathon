use clap::{Args, Subcommand};
use crossterm::style::{Attribute, Color};
use crossterm::{execute, queue, style};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    style::{Color as RatatuiColor, Style},
    symbols,
    text::Span,
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType},
};
use std::io;

use crate::cli::chat::consts::CONTEXT_WINDOW_SIZE;
use crate::cli::chat::token_counter::{CharCount, TokenCount};
use crate::cli::chat::{ChatError, ChatSession, ChatState};
use crate::os::Os;

#[derive(Debug, PartialEq, Args)]
pub struct UsageArgs {
    #[command(subcommand)]
    pub command: Option<UsageSubcommand>,
}

#[derive(Debug, PartialEq, Subcommand)]
pub enum UsageSubcommand {
    /// Display usage data for the past month as an aesthetically pleasing graph
    Graph,
}

impl UsageArgs {
    pub async fn execute(self, os: &Os, session: &mut ChatSession) -> Result<ChatState, ChatError> {
        match self.command {
            Some(UsageSubcommand::Graph) => self.execute_graph(os, session).await,
            None => self.execute_default(os, session).await,
        }
    }

    async fn execute_graph(self, os: &Os, session: &mut ChatSession) -> Result<ChatState, ChatError> {
        // Get usage data for the past month
        let usage_data = os
            .database
            .get_last_month_usage()
            .map_err(|e| ChatError::Custom(format!("Failed to get usage data: {}", e).into()))?;

        if usage_data.is_empty() {
            execute!(
                session.stderr,
                style::SetForegroundColor(Color::Yellow),
                style::Print("No usage data available for the past month.\n"),
                style::SetForegroundColor(Color::Reset)
            )?;
            return Ok(ChatState::PromptUser {
                skip_printing_tools: true,
            });
        }

        // Display the graph using ratatui
        self.display_usage_graph(&usage_data, session)?;

        Ok(ChatState::PromptUser {
            skip_printing_tools: true,
        })
    }

    fn display_usage_graph(
        &self,
        usage_data: &[(u32, crate::cli::UsageStatistics)],
        session: &mut ChatSession,
    ) -> Result<(), ChatError> {
        // Prepare data for the chart

        let cost_data: Vec<(f64, f64)> = usage_data.iter().map(|c| (c.0 as f64, c.1.dollars as f64)).collect();
        let max_cost = cost_data
            .iter()
            .map(|c| c.1)
            .max_by(|a, b| a.total_cmp(b))
            .unwrap_or_else(|| 0.);

        // let energy_data: Vec<(f64, f64)> = usage_data.iter().map(|c| (c.0 as f64, c.1.watthours as f64)).collect();
        // let max_energy = energy_data
        //     .iter()
        //     .map(|c| c.1)
        //     .max_by(|a, b| a.total_cmp(b))
        //     .unwrap_or_else(|| 0.);

        // Set up terminal
        let backend = CrosstermBackend::new(io::stderr());
        let mut terminal = Terminal::new(backend)
            .map_err(|e| ChatError::Custom(format!("Failed to create terminal: {}", e).into()))?;

        // Clear screen and enter alternate screen
        execute!(
            session.stderr,
            crossterm::terminal::EnterAlternateScreen,
            crossterm::cursor::Hide
        )?;

        // Enable raw mode
        crossterm::terminal::enable_raw_mode()
            .map_err(|e| ChatError::Custom(format!("Failed to enable raw mode: {}", e).into()))?;

        let result = terminal.draw(|f| {
            let size = f.size();

            // Create the chart
            let datasets = vec![
                Dataset::default()
                    .name("Daily Cost ($)")
                    .marker(symbols::Marker::Braille)
                    .style(Style::default().fg(RatatuiColor::Cyan))
                    .graph_type(GraphType::Line)
                    .data(&cost_data),
                // Dataset::default()
                //     .name("Energy Used (Wh)")
                //     .marker(symbols::Marker::Braille)
                //     .style(Style::default().fg(RatatuiColor::Yellow))
                //     .graph_type(GraphType::Line)
                //     .data(&energy_data),
            ];

            let min_day = usage_data.first().map(|(d, _)| *d as f64).unwrap_or(0.0);
            let max_day = usage_data.last().map(|(d, _)| *d as f64).unwrap_or(30.0);

            let chart = Chart::new(datasets)
                .block(
                    Block::default()
                        .title("Usage Statistics - Past Month")
                        .borders(Borders::ALL)
                        .style(Style::default().fg(RatatuiColor::White)),
                )
                .x_axis(
                    Axis::default()
                        .title("Day")
                        .style(Style::default().fg(RatatuiColor::Gray))
                        .bounds([min_day, max_day])
                        .labels(vec![
                            Span::styled(format!("{}", min_day as u32), Style::default().fg(RatatuiColor::Gray)),
                            Span::styled(format!("{}", max_day as u32), Style::default().fg(RatatuiColor::Gray)),
                        ]),
                )
                .y_axis(
                    Axis::default()
                        .title("Cost ($)")
                        .style(Style::default().fg(RatatuiColor::Gray))
                        .bounds([0.0, max_cost * 1.1])
                        .labels(vec![
                            Span::styled("0.00", Style::default().fg(RatatuiColor::Gray)),
                            Span::styled(format!("{:.2}", max_cost), Style::default().fg(RatatuiColor::Gray)),
                            // Span::styled(format!("{:.2}", max_energy), Style::default().fg(RatatuiColor::Gray)),
                        ]),
                );

            f.render_widget(chart, size);
        });

        // Wait for user input to exit
        if result.is_ok() {
            loop {
                if let Ok(event) = crossterm::event::read() {
                    if let crossterm::event::Event::Key(_) = event {
                        break;
                    }
                }
            }
        }

        // Restore terminal
        crossterm::terminal::disable_raw_mode()
            .map_err(|e| ChatError::Custom(format!("Failed to disable raw mode: {}", e).into()))?;

        execute!(
            session.stderr,
            crossterm::terminal::LeaveAlternateScreen,
            crossterm::cursor::Show
        )?;

        if let Err(e) = result {
            return Err(ChatError::Custom(format!("Failed to draw chart: {}", e).into()));
        }

        Ok(())
    }

    async fn execute_default(self, os: &Os, session: &mut ChatSession) -> Result<ChatState, ChatError> {
        let state = session
            .conversation
            .backend_conversation_state(os, true, &mut session.stderr)
            .await?;

        if !state.dropped_context_files.is_empty() {
            execute!(
                session.stderr,
                style::SetForegroundColor(Color::DarkYellow),
                style::Print("\nSome context files are dropped due to size limit, please run "),
                style::SetForegroundColor(Color::DarkGreen),
                style::Print("/context show "),
                style::SetForegroundColor(Color::DarkYellow),
                style::Print("to learn more.\n"),
                style::SetForegroundColor(style::Color::Reset)
            )?;
        }

        let data = state.calculate_conversation_size();
        let tool_specs_json: String = state
            .tools
            .values()
            .filter_map(|s| serde_json::to_string(s).ok())
            .collect::<Vec<String>>()
            .join("");
        let context_token_count: TokenCount = data.context_messages.into();
        let assistant_token_count: TokenCount = data.assistant_messages.into();
        let user_token_count: TokenCount = data.user_messages.into();
        let tools_char_count: CharCount = tool_specs_json.len().into(); // usize → CharCount
        let tools_token_count: TokenCount = tools_char_count.into(); // CharCount → TokenCount
        let total_token_used: TokenCount =
            (data.context_messages + data.user_messages + data.assistant_messages + tools_char_count).into();
        let window_width = session.terminal_width();
        // set a max width for the progress bar for better aesthetic
        let progress_bar_width = std::cmp::min(window_width, 80);

        let context_width =
            ((context_token_count.value() as f64 / CONTEXT_WINDOW_SIZE as f64) * progress_bar_width as f64) as usize;
        let assistant_width =
            ((assistant_token_count.value() as f64 / CONTEXT_WINDOW_SIZE as f64) * progress_bar_width as f64) as usize;
        let tools_width =
            ((tools_token_count.value() as f64 / CONTEXT_WINDOW_SIZE as f64) * progress_bar_width as f64) as usize;
        let user_width =
            ((user_token_count.value() as f64 / CONTEXT_WINDOW_SIZE as f64) * progress_bar_width as f64) as usize;

        let left_over_width = progress_bar_width
            - std::cmp::min(
                context_width + assistant_width + user_width + tools_width,
                progress_bar_width,
            );

        let is_overflow = (context_width + assistant_width + user_width + tools_width) > progress_bar_width;

        if is_overflow {
            queue!(
                session.stderr,
                style::Print(format!(
                    "\nCurrent context window ({} of {}k tokens used)\n",
                    total_token_used,
                    CONTEXT_WINDOW_SIZE / 1000
                )),
                style::SetForegroundColor(Color::DarkRed),
                style::Print("█".repeat(progress_bar_width)),
                style::SetForegroundColor(Color::Reset),
                style::Print(" "),
                style::Print(format!(
                    "{:.2}%",
                    (total_token_used.value() as f32 / CONTEXT_WINDOW_SIZE as f32) * 100.0
                )),
            )?;
        } else {
            queue!(
                session.stderr,
                style::Print(format!(
                    "\nCurrent context window ({} of {}k tokens used)\n",
                    total_token_used,
                    CONTEXT_WINDOW_SIZE / 1000
                )),
                // Context files
                style::SetForegroundColor(Color::DarkCyan),
                // add a nice visual to mimic "tiny" progress, so the overral progress bar doesn't look too
                // empty
                style::Print("|".repeat(if context_width == 0 && *context_token_count > 0 {
                    1
                } else {
                    0
                })),
                style::Print("█".repeat(context_width)),
                // Tools
                style::SetForegroundColor(Color::DarkRed),
                style::Print("|".repeat(if tools_width == 0 && *tools_token_count > 0 {
                    1
                } else {
                    0
                })),
                style::Print("█".repeat(tools_width)),
                // Assistant responses
                style::SetForegroundColor(Color::Blue),
                style::Print("|".repeat(if assistant_width == 0 && *assistant_token_count > 0 {
                    1
                } else {
                    0
                })),
                style::Print("█".repeat(assistant_width)),
                // User prompts
                style::SetForegroundColor(Color::Magenta),
                style::Print("|".repeat(if user_width == 0 && *user_token_count > 0 { 1 } else { 0 })),
                style::Print("█".repeat(user_width)),
                style::SetForegroundColor(Color::DarkGrey),
                style::Print("█".repeat(left_over_width)),
                style::Print(" "),
                style::SetForegroundColor(Color::Reset),
                style::Print(format!(
                    "{:.2}%",
                    (total_token_used.value() as f32 / CONTEXT_WINDOW_SIZE as f32) * 100.0
                )),
            )?;
        }

        execute!(session.stderr, style::Print("\n\n"))?;

        queue!(
            session.stderr,
            style::SetForegroundColor(Color::DarkCyan),
            style::Print("█ Context files: "),
            style::SetForegroundColor(Color::Reset),
            style::Print(format!(
                "~{} tokens ({:.2}%)\n",
                context_token_count,
                (context_token_count.value() as f32 / CONTEXT_WINDOW_SIZE as f32) * 100.0
            )),
            style::SetForegroundColor(Color::DarkRed),
            style::Print("█ Tools:       "),
            style::SetForegroundColor(Color::Reset),
            style::Print(format!(
                " ~{} tokens ({:.2}%)\n",
                tools_token_count,
                (tools_token_count.value() as f32 / CONTEXT_WINDOW_SIZE as f32) * 100.0
            )),
            style::SetForegroundColor(Color::Blue),
            style::Print("█ Q responses: "),
            style::SetForegroundColor(Color::Reset),
            style::Print(format!(
                "  ~{} tokens ({:.2}%)\n",
                assistant_token_count,
                (assistant_token_count.value() as f32 / CONTEXT_WINDOW_SIZE as f32) * 100.0
            )),
            style::SetForegroundColor(Color::Magenta),
            style::Print("█ Your prompts: "),
            style::SetForegroundColor(Color::Reset),
            style::Print(format!(
                " ~{} tokens ({:.2}%)\n\n\n",
                user_token_count,
                (user_token_count.value() as f32 / CONTEXT_WINDOW_SIZE as f32) * 100.0
            )),
        )?;

        let usage_summary = session.conversation.usage_summary(os);

        queue!(
            session.stderr,
            style::SetAttribute(Attribute::Bold),
            style::Print("Resources used today:\n"),
            style::SetAttribute(Attribute::Reset),
            style::Print(format!("💰 Total cost:  \t${:.2}", usage_summary.dollars)),
            style::Print("\n"),
            style::Print(format!("⚡ Energy used: \t{:.2} Wh", usage_summary.watthours)),
            style::Print("\n"),
            style::Print(format!("🌍 CO2 emited:  \t{:.2} g", usage_summary.co2)),
            style::Print("\n"),
            style::Print(format!("💧 Water used:  \t{:.2} mL", usage_summary.water)),
            style::Print("\n\n"),
        )?;

        queue!(
            session.stderr,
            style::SetAttribute(Attribute::Bold),
            style::Print("\n💡 Pro Tips:\n"),
            style::SetAttribute(Attribute::Reset),
            style::SetForegroundColor(Color::DarkGrey),
            style::Print("Run "),
            style::SetForegroundColor(Color::DarkGreen),
            style::Print("/compact"),
            style::SetForegroundColor(Color::DarkGrey),
            style::Print(" to replace the conversation history with its summary\n"),
            style::Print("Run "),
            style::SetForegroundColor(Color::DarkGreen),
            style::Print("/clear"),
            style::SetForegroundColor(Color::DarkGrey),
            style::Print(" to erase the entire chat history\n"),
            style::Print("Run "),
            style::SetForegroundColor(Color::DarkGreen),
            style::Print("/context show"),
            style::SetForegroundColor(Color::DarkGrey),
            style::Print(" to see tokens per context file\n\n"),
            style::SetForegroundColor(Color::Reset),
        )?;

        Ok(ChatState::PromptUser {
            skip_printing_tools: true,
        })
    }

    pub fn subcommand_name(&self) -> Option<&'static str> {
        match &self.command {
            Some(UsageSubcommand::Graph) => Some("graph"),
            None => None,
        }
    }
}
