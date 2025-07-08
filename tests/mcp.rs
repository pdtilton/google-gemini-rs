use std::{env, time::Duration};

use async_trait::async_trait;
use dotenv::dotenv;
use google_gemini_rs::{
    client::{self, Client},
    google,
};
use rust_mcp_sdk::{
    ClientSseTransport, ClientSseTransportOptions, McpClient, McpServer, TransportError,
    error::McpSdkError,
    macros::{JsonSchema, mcp_tool},
    mcp_client::{ClientHandler, client_runtime},
    mcp_server::{HyperServerOptions, ServerHandler, error::TransportServerError, hyper_server},
    schema::{
        ClientCapabilities, Implementation, InitializeRequestParams, InitializeResult,
        LATEST_PROTOCOL_VERSION, ServerCapabilities, ServerCapabilitiesTools, TextContent,
    },
};
use rust_mcp_sdk::{
    schema::{
        CallToolRequest, CallToolResult, ListToolsRequest, ListToolsResult, RpcError,
        schema_utils::CallToolError,
    },
    tool_box,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

const GEMINI_API_ENV_KEY: &str = "GEMINI_API_KEY";

const SECRET: &str = "Please";

const SECRET2: &str = "Pretty";

const SECRET3: &str = "Super";

#[derive(Debug, Error)]
enum Error {
    #[error(transparent)]
    DotEnv(#[from] dotenv::Error),
    #[error(transparent)]
    Client(#[from] client::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Google(#[from] google::Error),
    #[error(transparent)]
    McpSdk(#[from] McpSdkError),
    #[error(transparent)]
    McpTransport(#[from] TransportError),
    #[error(transparent)]
    Var(#[from] env::VarError),
    #[error(transparent)]
    TransportServer(#[from] TransportServerError),
}

struct WeatherClient {}

impl ClientHandler for WeatherClient {}

// STEP 1: Define a rust_mcp_schema::Tool ( we need one with no parameters for this example)
#[mcp_tool(
    name = "say_hello_world",
    description = "Prints \"Hello World!\" message"
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct SayHelloTool {}

impl SayHelloTool {
    pub fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let message = "Hello World\n".to_string();
        Ok(CallToolResult::text_content(vec![TextContent::from(
            message,
        )]))
    }
}

#[mcp_tool(name = "say_hidden", description = "Prints a hidden message")]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct SayHiddenTool {}

impl SayHiddenTool {
    pub fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let message = format!("{}", SECRET);
        Ok(CallToolResult::text_content(vec![TextContent::from(
            message,
        )]))
    }
}

#[mcp_tool(
    name = "say_secrets",
    description = "Prints some secret messages with the appended a sentinel value, so be sure to provide the sentinel."
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct SaySecretsTool {
    /// The sentinel value provided by the caller.
    sentinel: String,
}

impl SaySecretsTool {
    pub fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let message = format!("{}_{}", SECRET2, self.sentinel);
        Ok(CallToolResult::text_content(vec![TextContent::from(
            message,
        )]))
    }
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct NestedIndex {
    index: i32,
}

#[mcp_tool(
    name = "say_complex_secrets",
    description = "Prints some secret messages with the appended a sentinel value and index."
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct SaySecretsComplex {
    /// The sentinel value provided by the caller.
    sentinel: String,
    index: NestedIndex,
}

impl SaySecretsComplex {
    pub fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let message = format!("{}_{}_{}", SECRET3, self.sentinel, self.index.index);
        Ok(CallToolResult::text_content(vec![TextContent::from(
            message,
        )]))
    }
}

tool_box!(
    AllTools,
    [
        SayHelloTool,
        SayHiddenTool,
        SaySecretsTool,
        SaySecretsComplex
    ]
);

// STEP 2: Implement ServerHandler trait for a custom handler
// For this example , we only need handle_list_tools_request() and handle_call_tool_request() methods.
pub struct MyServerHandler;

#[async_trait]
#[allow(unused)]
impl ServerHandler for MyServerHandler {
    // Handle ListToolsRequest, return list of available tools as ListToolsResult
    async fn handle_list_tools_request(
        &self,
        request: ListToolsRequest,
        runtime: &dyn McpServer,
    ) -> Result<ListToolsResult, RpcError> {
        Ok(ListToolsResult {
            tools: AllTools::tools(),
            meta: None,
            next_cursor: None,
        })
    }

    /// Handles requests to call a specific tool.
    async fn handle_call_tool_request(
        &self,
        request: CallToolRequest,
        runtime: &dyn McpServer,
    ) -> Result<CallToolResult, CallToolError> {
        let tool_params: AllTools =
            AllTools::try_from(request.params).map_err(CallToolError::new)?;

        match tool_params {
            AllTools::SayHelloTool(say_hello_tool) => say_hello_tool.call_tool(),
            AllTools::SayHiddenTool(say_hidden_tool) => say_hidden_tool.call_tool(),
            AllTools::SaySecretsTool(say_secrets_tool) => say_secrets_tool.call_tool(),
            AllTools::SaySecretsComplex(say_secrets_complex) => say_secrets_complex.call_tool(),
        }
    }
}

const SERVER_URL: &str = "http://localhost:47777/sse";

async fn gemini_client() -> Result<Client, Error> {
    dotenv()?;

    let key = env::var(GEMINI_API_ENV_KEY)?;

    Ok(Client::new(&"gemini-2.0-flash".try_into()?, &key)
        .await?
        .with_defaults())
}

async fn mcp_server() -> Result<
    (
        tokio::task::JoinHandle<Result<(), Error>>,
        axum_server::Handle,
    ),
    Error,
> {
    let server_details = InitializeResult {
        // server name and version
        server_info: Implementation {
            name: "Hello World MCP Server".to_string(),
            version: "0.1.0".to_string(),
            title: None,
        },
        capabilities: ServerCapabilities {
            // indicates that server support mcp tools
            tools: Some(ServerCapabilitiesTools { list_changed: None }),
            ..Default::default()
        },
        meta: None,
        instructions: Some("server instructions...".to_string()),
        protocol_version: LATEST_PROTOCOL_VERSION.to_string(),
    };

    // STEP 2: instantiate our custom handler for handling MCP messages
    let handler = MyServerHandler {};

    // STEP 3: instantiate HyperServer, providing `server_details` , `handler` and HyperServerOptions
    let server = hyper_server::create_server(
        server_details,
        handler,
        HyperServerOptions {
            host: "127.0.0.1".to_string(),
            port: 47777,
            ..Default::default()
        },
    );

    let handle = server.server_handle();

    // STEP 4: Start the server
    let task = tokio::task::spawn(async {
        server.start().await?;
        Ok(())
    });

    Ok((task, handle))
}

#[tokio::test]
async fn test_mcp() -> Result<(), Error> {
    let (_, server) = mcp_server().await?;

    let weather_client_details: InitializeRequestParams = InitializeRequestParams {
        capabilities: ClientCapabilities::default(),
        client_info: Implementation {
            name: "simple-rust-mcp-client-sse".into(),
            version: "0.1.0".into(),
            title: None,
        },
        protocol_version: LATEST_PROTOCOL_VERSION.into(),
    };

    let transport = ClientSseTransport::new(SERVER_URL, ClientSseTransportOptions::default())?;

    let weather_handler = WeatherClient {};

    let weather_client =
        client_runtime::create_client(weather_client_details, transport, weather_handler);

    weather_client.clone().start().await?;

    let mut g_client = gemini_client()
        .await?
        .with_defaults()
        .with_tools_client(vec![weather_client.clone(), weather_client.clone()])
        .await?;

    let tools = weather_client.list_tools(None).await?.tools;

    for (tool_index, tool) in tools.iter().enumerate() {
        println!(
            "  {}. {} : {}",
            tool_index + 1,
            tool.name,
            tool.description.clone().unwrap_or_default()
        );
    }

    let response = g_client
        .send_text("Can you find the hidden message?")
        .await?;

    println!("response: {:?}", response.text());
    assert!(response.text().unwrap().contains(SECRET));

    let response = g_client
        .send_text("Can you find the second secret word? Use the sentinel value \"foobarbaz\".")
        .await?;

    println!("response: {:?}", response.text());
    assert!(
        response
            .text()
            .unwrap()
            .contains(&format!("{}_foobarbaz", SECRET2))
    );

    let response = g_client
        .send_text("Can you find the second secret word? Use the sentinel value \"foobarbaz\" and index 3.")
        .await?;

    println!("response: {:?}", response.text());
    assert!(
        response
            .text()
            .unwrap()
            .contains(&format!("{}_foobarbaz_3", SECRET3))
    );

    weather_client.shut_down().await?;
    server.graceful_shutdown(Some(Duration::from_secs(30)));

    Ok(())
}
