//! mem1 MCP server: exposes the mem1 memory store as MCP tools over stdio.
//!
//! Each tool wraps a transport-agnostic `handlers::*_svc` function from
//! mem1-server, so the MCP surface and the HTTP API share one retrieval/write
//! pipeline (extraction, embedding, RRF, rerank). Tool input structs derive
//! `JsonSchema` for automatic MCP `inputSchema` generation; they are converted
//! into the server's serde DTOs before calling the service layer.

use std::collections::HashMap;
use std::sync::Arc;

use mem1_server::api::dto::{
    AddMemoryRequest, CreateSessionRequest, ListMemoriesQuery, SearchRequest, UpdateMemoryRequest,
};
use mem1_server::api::handlers;
use mem1_server::app_state::AppState;

use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{
    CallToolResult, Content, ErrorData, Implementation, ServerCapabilities, ServerInfo,
};
use rmcp::{tool, tool_handler, tool_router, ServerHandler};
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Clone)]
pub struct Mem1Mcp {
    state: Arc<AppState>,
    // Used by the #[tool_router]/#[tool_handler] macro expansion.
    #[allow(dead_code)]
    tool_router: ToolRouter<Self>,
}

// ---- tool input schemas (agent-facing) ----

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AddArgs {
    /// Identifier for the user/owner whose memory this belongs to.
    pub user_id: String,
    /// The text to remember. Either `content` or `messages` must be provided.
    #[serde(default)]
    pub content: Option<String>,
    /// Conversation turns to extract memories from (alternative to `content`).
    #[serde(default)]
    pub messages: Option<Vec<ChatMessage>>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ChatMessage {
    /// Role of the speaker, e.g. "user" or "assistant".
    pub role: String,
    /// Message text.
    pub content: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SearchArgs {
    /// User/owner whose memories to search.
    pub user_id: String,
    /// The natural-language query.
    pub query: String,
    /// Max number of memories to return (default 10).
    #[serde(default)]
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListArgs {
    /// User/owner whose memories to list.
    pub user_id: String,
    /// Max number of memories to return (default 10).
    #[serde(default)]
    pub limit: Option<u32>,
    /// Number of memories to skip (pagination).
    #[serde(default)]
    pub offset: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetArgs {
    /// Memory id.
    pub id: String,
    /// User/owner the memory belongs to.
    pub user_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateArgs {
    /// Memory id to update.
    pub id: String,
    /// User/owner the memory belongs to.
    pub user_id: String,
    /// New content (omit to only change metadata).
    #[serde(default)]
    pub content: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DeleteArgs {
    /// Memory id to delete.
    pub id: String,
    /// User/owner the memory belongs to.
    pub user_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct UserScopeArgs {
    /// User/owner identifier.
    pub user_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateSessionArgs {
    /// User/owner the session belongs to.
    pub user_id: String,
    /// Session id (= run_id). Generated if omitted.
    #[serde(default)]
    pub id: Option<String>,
    /// Optional human-readable name.
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SessionRefArgs {
    /// Session id.
    pub id: String,
    /// User/owner the session belongs to.
    pub user_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DeleteSessionArgs {
    /// Session id.
    pub id: String,
    /// User/owner the session belongs to.
    pub user_id: String,
    /// When true, also delete the memories belonging to this session.
    #[serde(default)]
    pub cascade: bool,
}

fn ok_json<T: serde::Serialize>(value: &T) -> Result<CallToolResult, ErrorData> {
    let text = serde_json::to_string_pretty(value)
        .map_err(|e| ErrorData::internal_error(format!("serialize: {e}"), None))?;
    Ok(CallToolResult::success(vec![Content::text(text)]))
}

fn to_mcp_err(e: mem1_server::Error) -> ErrorData {
    ErrorData::internal_error(e.to_string(), None)
}

#[tool_router]
impl Mem1Mcp {
    pub fn new(state: Arc<AppState>) -> Self {
        Self {
            state,
            tool_router: Self::tool_router(),
        }
    }

    #[tool(
        description = "Store a memory for a user. Provide either `content` (a string) or `messages` (conversation turns). Returns the stored memory facts."
    )]
    async fn add_memory(
        &self,
        Parameters(args): Parameters<AddArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let req = if let Some(messages) = args.messages {
            AddMemoryRequest::ByMessages {
                user_id: args.user_id,
                messages: messages
                    .into_iter()
                    .map(|m| mem1_server::api::dto::Message {
                        role: m.role,
                        content: m.content,
                    })
                    .collect(),
                metadata: HashMap::new(),
            }
        } else {
            AddMemoryRequest::ByContent {
                user_id: args.user_id,
                content: args.content.unwrap_or_default(),
                metadata: HashMap::new(),
            }
        };
        let out = handlers::add_memory_svc(&self.state, req)
            .await
            .map_err(to_mcp_err)?;
        ok_json(&out)
    }

    #[tool(
        description = "Search a user's memories by natural-language query. Returns matching memories plus an assembled `formatted_context` string ready to inject into a prompt."
    )]
    async fn search_memory(
        &self,
        Parameters(args): Parameters<SearchArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let req = SearchRequest {
            user_id: Some(args.user_id),
            query: args.query,
            limit: args.limit.unwrap_or(10),
            scope: None,
            memory_type: None,
            filters: HashMap::new(),
        };
        let out = handlers::search_memories_svc(&self.state, req)
            .await
            .map_err(to_mcp_err)?;
        ok_json(&out)
    }

    #[tool(description = "List a user's stored memories with pagination.")]
    async fn list_memories(
        &self,
        Parameters(args): Parameters<ListArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let q = ListMemoriesQuery {
            user_id: args.user_id,
            limit: args.limit.unwrap_or(10),
            offset: args.offset.unwrap_or(0),
            scope: None,
            memory_type: None,
            agent_id: None,
            run_id: None,
        };
        let out = handlers::list_memories_svc(&self.state, q)
            .await
            .map_err(to_mcp_err)?;
        ok_json(&out)
    }

    #[tool(description = "Fetch a single memory by id.")]
    async fn get_memory(
        &self,
        Parameters(args): Parameters<GetArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let out = handlers::get_memory_svc(&self.state, &args.id, &args.user_id)
            .await
            .map_err(to_mcp_err)?;
        ok_json(&out)
    }

    #[tool(
        description = "Update a memory's content (re-embeds it). Provide the memory id and the new content."
    )]
    async fn update_memory(
        &self,
        Parameters(args): Parameters<UpdateArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let req = UpdateMemoryRequest {
            user_id: args.user_id,
            content: args.content,
            metadata: HashMap::new(),
        };
        let out = handlers::update_memory_svc(&self.state, &args.id, req)
            .await
            .map_err(to_mcp_err)?;
        ok_json(&out)
    }

    #[tool(description = "Delete a single memory by id.")]
    async fn delete_memory(
        &self,
        Parameters(args): Parameters<DeleteArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        handlers::delete_memory_svc(&self.state, &args.id, &args.user_id)
            .await
            .map_err(to_mcp_err)?;
        ok_json(&serde_json::json!({"deleted": args.id}))
    }

    #[tool(description = "Show the change history (add/update/delete) of a single memory.")]
    async fn memory_history(
        &self,
        Parameters(args): Parameters<GetArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let out = handlers::memory_history_svc(&self.state, &args.id, &args.user_id)
            .await
            .map_err(to_mcp_err)?;
        ok_json(&out)
    }

    #[tool(description = "List all user ids that have stored memories.")]
    async fn list_users(&self) -> Result<CallToolResult, ErrorData> {
        let out = handlers::list_users_svc(&self.state)
            .await
            .map_err(to_mcp_err)?;
        ok_json(&out)
    }

    #[tool(description = "Delete ALL memories for a single user. Destructive.")]
    async fn delete_all_memories(
        &self,
        Parameters(args): Parameters<UserScopeArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let q = ListMemoriesQuery {
            user_id: args.user_id,
            limit: 0,
            offset: 0,
            scope: None,
            memory_type: None,
            agent_id: None,
            run_id: None,
        };
        let out = handlers::delete_all_memories_svc(&self.state, q)
            .await
            .map_err(to_mcp_err)?;
        ok_json(&out)
    }

    #[tool(
        description = "Create a session (an optional group for memories sharing a run_id). Pass a stable id, or omit to auto-generate one."
    )]
    async fn create_session(
        &self,
        Parameters(args): Parameters<CreateSessionArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let req = CreateSessionRequest {
            user_id: args.user_id,
            id: args.id,
            name: args.name,
            metadata: std::collections::HashMap::new(),
        };
        let out = handlers::create_session_svc(&self.state, req)
            .await
            .map_err(to_mcp_err)?;
        ok_json(&out)
    }

    #[tool(description = "List a user's sessions.")]
    async fn list_sessions(
        &self,
        Parameters(args): Parameters<UserScopeArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let out = handlers::list_sessions_svc(&self.state, &args.user_id)
            .await
            .map_err(to_mcp_err)?;
        ok_json(&out)
    }

    #[tool(description = "Fetch a single session by id.")]
    async fn get_session(
        &self,
        Parameters(args): Parameters<SessionRefArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let out = handlers::get_session_svc(&self.state, &args.id, &args.user_id)
            .await
            .map_err(to_mcp_err)?;
        ok_json(&out)
    }

    #[tool(
        description = "Delete a session. With cascade=true, also delete the memories in that session; otherwise the memories are kept."
    )]
    async fn delete_session(
        &self,
        Parameters(args): Parameters<DeleteSessionArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let out = handlers::delete_session_svc(&self.state, &args.id, args.user_id, args.cascade)
            .await
            .map_err(to_mcp_err)?;
        ok_json(&out)
    }
}

#[tool_handler]
impl ServerHandler for Mem1Mcp {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_instructions(
                "mem1 memory backend. Use add_memory to store durable facts about a user, \
                 and search_memory to retrieve relevant memories (returns formatted_context \
                 ready to inject into a prompt). Always pass a stable user_id.",
            )
            .with_server_info(Implementation::new("mem1-mcp", env!("CARGO_PKG_VERSION")))
    }
}
