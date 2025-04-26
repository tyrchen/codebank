# CodeBank MCP Server

This is a Model Context Protocol (MCP) server for CodeBank, allowing AI agents to interact with code bases and generate code bank summaries.

## Installation

Make sure you have Rust and Cargo installed, then build the project:

```bash
cargo build --release
```

The binary will be in `target/release/cb_mcp`.

## Usage

### Command Line Interface

There are several ways to use the CodeBank MCP:

#### Generate a Code Bank

Generate a code bank from source code with different strategies:

```bash
# Default strategy (includes all code)
cb_mcp generate ./src default

# Summary strategy (only public interfaces will be included)
cb_mcp generate ./src summary

# No-tests strategy (all code except test cases will be included)
cb_mcp generate ./src no-tests
```

#### Generate and Save to File

Generate a code bank and save it to a specific file:

```bash
cb_mcp generate-file ./src default ./src/code.md
```

### Running as an MCP Server

#### STDIO Mode

Run as an MCP server in STDIO mode:

```bash
cb_mcp stdio
```

This allows AI agents to interact with the server through standard input/output.

#### SSE Mode

Run as an MCP server in SSE (Server-Sent Events) mode on a specific port:

```bash
cb_mcp sse --port 3000
```

This starts a web server that AI agents can interact with.

## Server API

When running in server mode, the following tools are available to AI agents:

- `mcp_generate`: Generate code bank from source code
- `mcp_generate_file`: Generate code bank and save to file

## Strategies

CodeBank supports different generation strategies:

- `default`: Includes all code from the specified path
- `summary`: Includes only public interfaces, function signatures without bodies
- `no-tests`: Includes all code except test cases
