use clap::Parser;
use error_chain::error_chain;
use log::{error, info};
use std::time::Instant;
use env_logger;

mod crawler;

#[derive(Parser, Debug)]
#[clap(Author, version, about, long_about=None)]
struct Args{
    #[clap(short, long)]
    url: String,

    #[clap(short, ong, default_value_t=2)]
    depth: u32,

    #[clap(short, long, multiple = true)]
    allowed_domains: Option<Vec<String>>,

    #[clap(short, long, multiple = true)]
    keywords: Vec<String>,

    #[clap(short, long, default_value_t=4)]
    concurrency: usize,
}
error_chain!{
    foreign_links{
        Io(std::io::Error);
        HttpRequest(reqwest::Error);
    }
}

#[tokio::main]
async fn main()->Result<()> {
    env_logger::init();
    let args = Args::parse();
    
    info!("Crawling {} at depth {}...", &args.url, args.depth);
    info!("Allowed domains: {:?}", args.allowed_domains);
    info!("Keywords: {:?}", args.keywords);
    info!("Concurrency: {}", args.concurrency);

    let start = Instant::now();

    match crawler::crawl(&args.url, args.depth, args.allowed_domains, args.keywords, args.concurrency).await {
        Ok(results) => {
            info!("Crawling completed successfully");
            info!("Total pages crawled: {}", results.len());
            info!("Top 10 results:");
            for (url, score) in results.iter().take(10) {
                info!("URL: {}, Score: {}", url, score);
            }
        }
        Err(e) => {
            error!("Error during crawling: {}", e);
        }
    }

    let duration = start.elapsed();
    info!("Crawling took {:?}", duration);

    Ok(())
}
