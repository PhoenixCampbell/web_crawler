use error_chain::error_chain;
use log::{debug, error};
use reqwest::Client;
use robotstxt::{DefaultMathcer, RobotFileParser};
use scraper::{Html, Selector};
use std::collections::{HashSet, VecDeque};
use std::time::{Duration, Instant};
use tokio::time::sleep;
use url::Url;
use regex::Regex;

error_chain!{
    foreign_links{
        ReqwestError(reqwest::Error);
        Url(url::ParseError);
    }
}

struct CrawlState{
    client: Client,
    visited: Hashset<String>,
    queue: VecDeque<(String, u32)>,
    results: Vec<String>,
    robots_cache: HashMap<String, RobotFileParser>,
    last_request: HashMap<String, Instant>,
    keyword_frequency: HashMap<String, usize>,
}

impl CrawlState{
    fn new(client: Client)->Self{
        CrawlState{
            client,
            visited: HashSet::new(),
            queue: VecDeque::new(),
            results: Vec::new(),
            robots_cache: HashMap::new();
            last_request: HashMap::new(),
            keyword_frequency: HashMap::new();
        }
    }
}

pub async fn crawl(start_url: &str, max_depth: u32, allowed_domains: Option<Vec<String>>, keywords: Vec<String>) -> Result<Vec<(String, usize)>> {
    let client=Client::new();
    let mut state=CrawlState::new(client);
    
    state.queue.push_back((start_url.to_string(), 0));

    while let Some((url,depth))=state.queue.pop_front() {
        if depth > max_depth{
            continue;
        }
        if state.visited.contains(&url){
            continue;
        }
        if !is_allowed_domain(&url, &allowed_domains){
            continue;
        }
        if !can_fetch(&mut state, &url).await{
            continue;
        }
        debug!("Crawling {} (depth: {})", url, depth);

        rate_limit(&mute state, &url).await{
            Ok(html)=>{
                state.visited.insert(url.clone());
                state.results.push(url.clone());

                update_keyword_frequency(&mut state, &html, &keywords);

                let links=extract_links(&html, &url);
                for link in links{
                    state.queue.push_back((link, depth+1));
                }
            }
            Err(e)=>{
                error!("Error fetching {}: {}", url, e);
            }
        }
    }
    Ok(rank_results(&state))
}

async fn fetch_url(client: &Client,url, &str)->Result<String>{
    let response = client.get(url).send().await?.text().await?;
    Ok(response)
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
fn is_allowed_domain(url: &str, allowed_domains: &Option<Vec<String>>) -> bool {
    if let Some(domains) = allowed_domains {
        let url = Url::parse(url).unwrap();
        domains.iter().any(|domain| url.domain().map_or(false, |d| d.ends_with(domain)))
    } else {
        true
    }
}

async fn can_fetch(state: &mut CrawlState, url: &str) -> bool {
    let url = Url::parse(url).unwrap();
    let domain = url.domain().unwrap().to_string();
    let path = url.path();

    if !state.robots_cache.contains_key(&domain) {
        let robots_url = format!("{}://{}/robots.txt", url.scheme(), domain);
        match state.client.get(&robots_url).send().await {
            Ok(response) => {
                let robots_txt = response.text().await.unwrap_or_default();
                let parser = RobotFileParser::new(robots_url);
                parser.parse(&robots_txt);
                state.robots_cache.insert(domain.clone(), parser);
            }
            Err(_) => {
                state.robots_cache.insert(domain.clone(), RobotFileParser::new(robots_url));
            }
        }
    }

    let matcher = DefaultMatcher::default();
    state.robots_cache.get(&domain).map_or(true, |parser| parser.can_fetch("*", path, &matcher))
}

async fn rate_limit(state: &mut CrawlState, url: &str) {
    let domain = Url::parse(url).unwrap().domain().unwrap().to_string();
    let now = Instant::now();

    if let Some(last_request) = state.last_request.get(&domain) {
        let elapsed = now.duration_since(*last_request);
        if elapsed < Duration::from_secs(1) {
            sleep(Duration::from_secs(1) - elapsed).await;
        }
    }

    state.last_request.insert(domain, Instant::now());
}

fn update_keyword_frequency(state: &mut CrawlState, html: &str, keywords: &[String]) {
    let document = Html::parse_document(html);
    let text = document.root_element().text().collect::<String>().to_lowercase();

    for keyword in keywords {
        let count = Regex::new(&format!(r"\b{}\b", keyword.to_lowercase()))
            .unwrap()
            .find_iter(&text)
            .count();
        *state.keyword_frequency.entry(keyword.to_string()).or_insert(0) += count;
    }
}

fn rank_results(state: &CrawlState) -> Vec<(String, usize)> {
    let mut ranked_results: Vec<(String, usize)> = state.results
        .iter()
        .map(|url| {
            let score = state.keyword_frequency.iter()
                .map(|(_, &count)| count)
                .sum();
            (url.clone(), score)
        })
        .collect();

    ranked_results.sort_by(|a, b| b.1.cmp(&a.1));
    ranked_results
}