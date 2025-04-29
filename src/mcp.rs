use crate::{Bank, BankConfig, BankStrategy, CodeBank};
use anyhow::Result;
use rmcp::{
    Error as McpError, ServerHandler,
    model::{CallToolResult, Content, ErrorCode, ServerCapabilities, ServerInfo},
    schemars, tool,
};
use std::fs;
use std::path::PathBuf;

/// CodeBank MCP server implementation
#[derive(Debug, Clone)]
pub struct CodeBankMcp;

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GenerateRequest {
    #[schemars(description = "Path to source code. Please provide the absolute path.")]
    pub path: String,

    #[schemars(description = "Strategy for generation (default, summary, no-tests)")]
    pub strategy: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GenerateFileRequest {
    #[schemars(description = "Path to source code. Please provide the absolute path.")]
    pub path: String,

    #[schemars(description = "Strategy for generation (default, summary, no-tests)")]
    pub strategy: String,

    #[schemars(description = "Output file path. Please provide the absolute path.")]
    pub output: String,
}

/// Helper function to create an invalid argument error
fn invalid_argument_error(message: impl Into<String>) -> McpError {
    McpError::new(ErrorCode::INVALID_PARAMS, message.into(), None)
}

/// Helper function to create an internal error
fn internal_error(message: impl Into<String>) -> McpError {
    McpError::new(ErrorCode::INTERNAL_ERROR, message.into(), None)
}

#[tool(tool_box)]
impl CodeBankMcp {
    pub fn new() -> Self {
        Self
    }

    /// Parse and validate the strategy parameter
    fn parse_strategy(&self, strategy: &str) -> Result<BankStrategy> {
        match strategy {
            "default" => Ok(BankStrategy::Default),
            "summary" => Ok(BankStrategy::Summary),
            "no-tests" => Ok(BankStrategy::NoTests),
            _ => Err(anyhow::anyhow!(
                "Invalid strategy: {}. Available strategies: default, summary, no-tests",
                strategy
            )),
        }
    }

    #[tool(
        description = "Generate code bank from source files. Helps understand codebase structure, get current code status, summarize code functionality. Useful for code review, onboarding, and maintaining codebase overview."
    )]
    async fn generate(
        &self,
        #[tool(aggr)] req: GenerateRequest,
    ) -> Result<CallToolResult, McpError> {
        let path = PathBuf::from(&req.path);

        // Validate path
        if !path.exists() {
            return Err(invalid_argument_error(format!(
                "Path does not exist: {}",
                path.display()
            )));
        }

        // Parse strategy
        let bank_strategy = match self.parse_strategy(&req.strategy) {
            Ok(strategy) => strategy,
            Err(e) => {
                return Err(invalid_argument_error(e.to_string()));
            }
        };

        // Generate code bank
        let codebank = match CodeBank::try_new() {
            Ok(cb) => cb,
            Err(e) => {
                return Err(internal_error(format!(
                    "Failed to initialize CodeBank: {}",
                    e
                )));
            }
        };

        let config = BankConfig::new(path, bank_strategy, vec![]);
        let content = match codebank.generate(&config) {
            Ok(content) => content,
            Err(e) => {
                return Err(internal_error(format!(
                    "Failed to generate code bank: {}",
                    e
                )));
            }
        };

        Ok(CallToolResult::success(vec![Content::text(content)]))
    }

    #[tool(
        description = "Generate code bank from source files and save to output file. Helps understand codebase structure, get current code status, summarize code functionality. Useful for code review, onboarding, and maintaining codebase overview."
    )]
    async fn generate_file(
        &self,
        #[tool(aggr)] req: GenerateFileRequest,
    ) -> Result<CallToolResult, McpError> {
        let path = PathBuf::from(&req.path);
        let output = PathBuf::from(&req.output);

        // Validate path
        if !path.exists() {
            return Err(invalid_argument_error(format!(
                "Path does not exist: {}",
                path.display()
            )));
        }

        // Parse strategy
        let bank_strategy = match self.parse_strategy(&req.strategy) {
            Ok(strategy) => strategy,
            Err(e) => {
                return Err(invalid_argument_error(e.to_string()));
            }
        };

        // Generate code bank
        let codebank = match CodeBank::try_new() {
            Ok(cb) => cb,
            Err(e) => {
                return Err(internal_error(format!(
                    "Failed to initialize CodeBank: {}",
                    e
                )));
            }
        };

        let config = BankConfig::new(path, bank_strategy, vec![]);
        let content = match codebank.generate(&config) {
            Ok(content) => content,
            Err(e) => {
                return Err(internal_error(format!(
                    "Failed to generate code bank: {}",
                    e
                )));
            }
        };

        // Write to file
        match fs::write(&output, &content) {
            Ok(_) => {}
            Err(e) => {
                return Err(internal_error(format!("Failed to write to file: {}", e)));
            }
        };

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Successfully generated code bank and saved to {}",
            output.display()
        ))]))
    }
}

#[tool(tool_box)]
impl ServerHandler for CodeBankMcp {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "A CodeBank MCP server that allows AI agents to interact with code bank".into(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}

impl Default for CodeBankMcp {
    fn default() -> Self {
        Self::new()
    }
}
