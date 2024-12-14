use clap::Parser;
use error_chain::error_chain;
use log::{error, info};
use std::time::Instant;

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

    let start = Instant::now();
    match crawler::crawl(&args.url, args.depth, args.allowed_domains, args.keywords).await {
        Ok(results) => {
            info!("Crawling completed successfully");
            info!("Total pages crawled: {}", results.len());
            for (url, score) in results.iter().take(10) {
                println!("URL: {}, Score: {}", url, score);
            }
        }
        Err(e) => {
            error!("Error during crawling: {}", e);
        }
    }

    let duration = start_time.elapsed();
    info!("Crawling took {:?}", duration);

    Ok(())
}
