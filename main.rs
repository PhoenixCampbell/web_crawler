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
    match crawler::crawl(&args.url, args.depth).await {
        Ok(links) => {
            info!("Crawled {} links in {} seconds.", links.len(), start.elapsed().as_secs_f64());
        }
        Err(e) => {
            error!("Error crawling: {}", e);
        }
    }
    Ok(())
}
