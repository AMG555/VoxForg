//! Model Context Protocol (MCP) server implementation for VoxForg.
//!
//! Exposes VoxForg speech synthesis, voice cloning, audio transcription, model
//! catalog, and DAG pipeline capabilities directly to AI agents (Cursor, Claude Desktop)
//! via standard JSON-RPC 2.0 over standard input and output streams.

pub mod protocol;
pub mod security;
pub mod server;
pub mod tools;

pub use protocol::{
    JsonRpcError, JsonRpcRequest, JsonRpcResponse, McpPrompt, McpResource, McpResourceContent,
    McpTool, ToolCallResult,
};
pub use server::McpServer;
pub use tools::get_mcp_tools;
