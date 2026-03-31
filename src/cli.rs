use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(
    name = "ferris-search",
    version,
    about = "A blazing-fast multi-engine web search tool & MCP server with local document indexing, written in Rust."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Search the web using one or more engines
    Search {
        /// Search query
        query: String,

        /// Search engines to use (can be specified multiple times, or comma-separated)
        #[arg(short, long = "engine", value_delimiter = ',')]
        engine: Vec<String>,

        /// Maximum results per engine (1-50)
        #[arg(short, long, default_value_t = 10)]
        limit: u32,

        /// Output format
        #[arg(short, long, default_value = "text")]
        format: OutputFormat,
    },

    /// Fetch and extract content from a URL
    Fetch {
        /// URL to fetch
        url: String,

        /// Maximum characters of content to return
        #[arg(short, long, default_value_t = 30000)]
        max_chars: u32,

        /// Output format
        #[arg(short, long, default_value = "text")]
        format: OutputFormat,
    },

    /// List all supported and allowed search engines
    ListEngines {
        /// Output format
        #[arg(short, long, default_value = "text")]
        format: OutputFormat,
    },

    /// Show the current effective configuration
    ShowConfig {
        /// Output format
        #[arg(short, long, default_value = "text")]
        format: OutputFormat,
    },

    /// Start the MCP server (stdio transport)
    Mcp,

    /// Build a full-text index from local documents
    IndexLocal {
        /// Directories or files to index (can be specified multiple times)
        #[arg(short, long = "path", required = true)]
        path: Vec<String>,

        /// Directory to store the index (default: from LOCAL_DOCS_INDEX_PATH or ./.ferris-index)
        #[arg(long)]
        index_path: Option<String>,

        /// Output format
        #[arg(short, long, default_value = "text")]
        format: OutputFormat,
    },

    /// Search the local document index
    SearchLocal {
        /// Search query
        query: String,

        /// Directory of the index (default: from LOCAL_DOCS_INDEX_PATH or ./.ferris-index)
        #[arg(long)]
        index_path: Option<String>,

        /// Maximum results to return (1-50)
        #[arg(short, long, default_value_t = 10)]
        limit: u32,

        /// Output format
        #[arg(short, long, default_value = "text")]
        format: OutputFormat,
    },
}

#[derive(Clone, ValueEnum)]
pub enum OutputFormat {
    Text,
    Json,
}
