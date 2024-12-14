use error_chain::error_chain;
use log::{debug, error};
use reqwest::Client;
use scraper::{Html, Selector};
use std::collections::{HashSet, VecDeque};
use url::Url;

error_chain!{
    foreign_links{
        ReqwestError(reqwest::Error);
        Url(url::ParseError);
    }
}

pub async fn crawl(start_url: &str, depth: u32) -> Result<Vec<String>> {
    let client=Client::new();
    let mut visited=HashSet::new();
    let mut queue=VecDeque::new();
    let mut results=Vec::new();
    
    queue.push_back((start_url.to_string(), 0));

    while let Some((url,depth))=queue.pop_front() {
        if depth > max_depth{
            continue;
        }
        if visited.contains(&url){
            continue;
        }
        debug!("Crawling {} (depth: {})", url, depth);

        match fetch_url(&client, &url).await{
            Ok(html)=>{
                visited.insert(url.clone());
                results.push(url.clone());

                let links=extract_links(&html, &url);
                for link in links{
                    queue.push_back((link, depth+1));
                }
            }
            Err(e)=>{
                error!("Error fetching {}: {}", url, e);
            }
        }
    }
    Ok(results)
}

async fn fetch_url(client: &Client, url: &str) -> Result<String> {
    let response=client.get(url).send().await?.test().await?;
    ok(response)
}

fn extract_links(html: &str, base_url: &str) -> Vec<String> {
    let document=Html::parse_document(html);
    let selector=Selector::parse("a").unwrap();
    let base_url=Url::parse(base_url).unwrap();

    document
        .select(&selector)
        .filter_map(|element| {
            element.value().attr("href").and_then(|href| {
                base_url.join(href).ok().map(|url| url.to_string())
            })
        })
        .collect()
}