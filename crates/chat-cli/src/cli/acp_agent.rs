//! A simple ACP agent server for educational purposes.
//!
//! The agent communicates with clients over stdio. To run it with logging:
//!
//! ```bash
//! RUST_LOG=info cargo run --example agent
//! ```
//!
//! To connect it to the example client from this crate:
//!
//! ```bash
//! cargo build --example agent && cargo run --example client -- target/debug/examples/agent
//! ```

use std::{cell::Cell, process::ExitCode};

use agent_client_protocol::{
    self as acp, AuthMethod, AuthMethodId, Client, ContentBlock, PromptCapabilities, SessionNotification,
};
use tokio::sync::{mpsc, oneshot};
use tokio_util::compat::{TokioAsyncReadCompatExt as _, TokioAsyncWriteCompatExt as _};

use crate::{
    api_client::model::{ConversationState, UserInputMessage},
    cli::chat::cli::model::get_available_models,
    os::Os,
};

struct ExampleAgent {
    os: Os,
    session_update_tx: mpsc::UnboundedSender<(acp::SessionNotification, oneshot::Sender<()>)>,
    next_session_id: Cell<u64>,
}

impl ExampleAgent {
    fn new(os: Os, session_update_tx: mpsc::UnboundedSender<(acp::SessionNotification, oneshot::Sender<()>)>) -> Self {
        Self {
            os,
            session_update_tx,
            next_session_id: Cell::new(0),
        }
    }
}

impl acp::Agent for ExampleAgent {
    async fn initialize(&self, arguments: acp::InitializeRequest) -> Result<acp::InitializeResponse, acp::Error> {
        // log::info!("Received initialize request {arguments:?}");
        Ok(acp::InitializeResponse {
            protocol_version: acp::V1,
            agent_capabilities: acp::AgentCapabilities {
                load_session: true,
                prompt_capabilities: PromptCapabilities {
                    image: true,
                    audio: false,
                    embedded_context: false,
                },
            },
            auth_methods: vec![
                AuthMethod {
                    id: AuthMethodId("BuilderId".into()),
                    name: "Builder ID".into(),
                    description: None,
                },
                AuthMethod {
                    id: AuthMethodId("IdentityCenter".into()),
                    name: "Identity Center".into(),
                    description: None,
                },
            ],
        })
    }

    async fn authenticate(&self, arguments: acp::AuthenticateRequest) -> Result<(), acp::Error> {
        // log::info!("Received authenticate request {arguments:?}");
        Ok(())
    }

    async fn new_session(&self, arguments: acp::NewSessionRequest) -> Result<acp::NewSessionResponse, acp::Error> {
        // log::info!("Received new session request {arguments:?}");
        let session_id = self.next_session_id.get();
        self.next_session_id.set(session_id + 1);
        Ok(acp::NewSessionResponse {
            session_id: acp::SessionId(session_id.to_string().into()),
        })
    }

    async fn load_session(&self, arguments: acp::LoadSessionRequest) -> Result<(), acp::Error> {
        // log::info!("Received load session request {arguments:?}");
        Err(acp::Error::method_not_found())
    }

    async fn prompt(&self, arguments: acp::PromptRequest) -> Result<acp::PromptResponse, acp::Error> {
        // log::info!("Received prompt request {arguments:?}");

        let conversation_id = Some(uuid::Uuid::new_v4().to_string());
        let content = match arguments.prompt.first().ok_or(acp::Error::internal_error())? {
            ContentBlock::Text(text_content) => text_content.text.clone(),
            ContentBlock::Image(image_content) => "image".into(),
            ContentBlock::Audio(audio_content) => "audio".into(),
            ContentBlock::ResourceLink(resource_link) => "resource link".into(),
            ContentBlock::Resource(embedded_resource) => "embedded resource".into(),
        };
        let model_id = Some(
            get_available_models(&self.os)
                .await
                .map_err(|_| acp::Error::internal_error())?
                .1
                .model_id,
        );
        let user_input_message = UserInputMessage {
            content,
            user_input_message_context: None,
            user_intent: None,
            images: None,
            model_id,
        };
        let conversation = ConversationState {
            conversation_id,
            user_input_message,
            history: None,
        };
        SendMessageStream::send_message(&self.os.client, conversation)
        let a = match self.os.client.send_message(conversation).await.unwrap().recv().await {
            Ok(Some(content)) => format!("{:?}", content),
            Ok(None) => "ok none".into(),
            Err(_) => "err".into(),
            _ => "unkown".into(),
        };

        for content in ["Client sent: ".into()].into_iter().chain(vec![a]) {
            let content = ContentBlock::Text(agent_client_protocol::TextContent {
                annotations: None,
                text: content,
            });
            let (tx, rx) = oneshot::channel();
            self.session_update_tx
                .send((
                    SessionNotification {
                        session_id: arguments.session_id.clone(),
                        update: acp::SessionUpdate::AgentMessageChunk { content },
                    },
                    tx,
                ))
                .map_err(|_| acp::Error::internal_error())?;
            rx.await.map_err(|_| acp::Error::internal_error())?;
        }
        Ok(acp::PromptResponse {
            stop_reason: acp::StopReason::EndTurn,
        })
    }

    async fn cancel(&self, args: acp::CancelNotification) -> Result<(), acp::Error> {
        // log::info!("Received cancel request {args:?}");
        Ok(())
    }
}

pub(crate) async fn agent(os: &mut Os) -> Result<std::process::ExitCode, eyre::Error> {
    let outgoing = tokio::io::stdout().compat_write();
    let incoming = tokio::io::stdin().compat();

    // let (agents, _) = Agents::load(os, None, true, &mut std::io::stderr(), true).await;
    // let model = agents.active_idx;

    // let mcp_enabled = os.client.is_mcp_enabled();
    // let (prompt_request_sender, prompt_request_receiver) = tokio::sync::broadcast::channel::<PromptQuery>(5);
    // let (prompt_response_sender, prompt_response_receiver) =
    //     tokio::sync::broadcast::channel::<PromptQueryResult>(5);
    // let mut tool_manager = ToolManagerBuilder::default()
    //     .prompt_query_result_sender(prompt_response_sender)
    //     .prompt_query_receiver(prompt_request_receiver)
    //     .prompt_query_sender(prompt_request_sender.clone())
    //     .prompt_query_result_receiver(prompt_response_receiver.resubscribe())
    //     .conversation_id(&conversation_id)
    //     .agent(agents.get_active().cloned().unwrap_or_default())
    //     .build(os, Box::new(std::io::stderr()), false)
    //     .await?;
    // let tool_config = tool_manager.load_tools(os, &mut std::io::stderr()).await?;
    // let (models, default_model_opt) = get_available_models(os).await?;

    // let model_id: Option<String> = if let Some(saved) = os.database.settings.get_string(Setting::ChatDefaultModel) {
    //             find_model(&models, &saved)
    //                 .map(|m| m.model_id.clone())
    //                 .or(Some(default_model_opt.model_id.clone()))
    //         } else {
    //             Some(default_model_opt.model_id.clone())
    //         };
    // // self.conversation = Some(
    // let conversation_state = ConversationState::new(
    //     &conversation_id,
    //     agents,
    //     tool_config,
    //     tool_manager,
    //     model_id,
    //     os,
    //     mcp_enabled,
    // )
    // .await;

    // The AgentSideConnection will spawn futures onto our Tokio runtime.
    // LocalSet and spawn_local are used because the futures from the
    // agent-client-protocol crate are not Send.
    let local_set = tokio::task::LocalSet::new();
    local_set
        .run_until(async move {
            let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
            // Start up the ExampleAgent connected to stdio.
            let (conn, handle_io) =
                acp::AgentSideConnection::new(ExampleAgent::new(os.to_owned(), tx), outgoing, incoming, |fut| {
                    tokio::task::spawn_local(fut);
                });
            // Kick off a background task to send the ExampleAgent's session notifications to the client.
            tokio::task::spawn_local(async move {
                while let Some((session_notification, tx)) = rx.recv().await {
                    let result = conn.session_notification(session_notification).await;
                    if let Err(e) = result {
                        // log::error!("{e}");
                        break;
                    }
                    tx.send(()).ok();
                }
            });
            // Run until stdin/stdout are closed.
            handle_io.await
        })
        .await
        .map(|_| ExitCode::SUCCESS)
        .map_err(|err| eyre::Error::msg(err.to_string()))
}
