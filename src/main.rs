use clap::Parser;
use error_chain::error_chain;
use log::{error, info};
use std::time::Instant;

mod crawler;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    url: String,

    #[clap(short, long, default_value_t = 2)]
    depth: u32,
}

error_chain! {
    foreign_links {
        Io(std::io::Error);
        HttpRequest(reqwest::Error);
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let args = Args::parse();
    
    info!("Starting web crawler");
    let start_time = Instant::now();

    match crawler::crawl(&args.url, args.depth).await {
        Ok(results) => {
            info!("Crawling completed successfully");
            info!("Total pages crawled: {}", results.len());
            // TODO: Process and display results
        }
        Err(e) => {
            error!("Error during crawling: {}", e);
        }
    }

    let duration = start_time.elapsed();
    info!("Crawling took {:?}", duration);

    Ok(())
}