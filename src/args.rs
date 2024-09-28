use clap::Parser;

/// A web crawler application.

/// This program crawls the web starting from a given URL and can be configured for concurrency.
/// It uses two pools: 
/// 1. For sending HTTP requests (scorpions) and 
/// 2. For parsing HTML and finding new URLs to crawl. 
 
///
/// # Parameters
/// - `target`: The starting URL for the crawl.
/// - `connections`: Number of concurrent HTTP request tasks.
/// - `parsers`: Number of concurrent HTML parsing tasks.
/// - `interval`: Time (in ms) between requests to avoid rate-limiting or blocking.
/// - `file`: Optional file path to save the index.
///
/// Beware: Setting too high a QPS (queries per second) may result in IP blocking.

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]  // Automatically fills metadata from Cargo.toml
pub struct Args {
    /// The target URL to start crawling from.
    #[arg(short = 't', long = "target", default_value = "https://<TARGET_URL>", help = "The starting point for the web crawl.")]
    pub target: String,

    /// Number of connections for sending HTTP requests.
    #[arg(short = 'c', long = "connections", default_value_t = 128, value_parser = clap::value_parser!(u16).range(1..), help = "Number of concurrent connection tasks.")]
    pub connections: u16,

    /// Number of parsers for interpreting HTML and finding links.
    #[arg(short = 'p', long = "parsers", default_value_t = 64, value_parser = clap::value_parser!(u16).range(1..), help = "Number of concurrent parser tasks.")]
    pub parsers: u16,

    /// Time interval between requests to the same domain (in milliseconds).
    #[arg(short = 'i', long = "interval", default_value_t = 5000, value_parser = clap::value_parser!(u64).range(1..), help = "Delay between requests to the same domain (in ms).")]
    pub interval: u64,

    /// The optional file path to write the index to.
    #[arg(short = 'f', long = "file", help = "The file path to save the crawl results (optional).")]
    pub file: Option<String>,
}

