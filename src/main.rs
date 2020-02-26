extern crate reqwest;
extern crate scraper;

use std::path::Path;
use scraper::{Html, Selector};
use github_gql as gh;
use gh::client::Github;
use gh::query::Query;
use serde_json::Value;
use dotenv::dotenv;

const URL: &str = "https://themes.gohugo.io/";

const QUERY: &str = "query { repository(name: \"{}\", owner: \"{}\"){name,stargazers{totalCount},forks{totalCount},updatedAt}}";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{

    dotenv().ok();
    let mut g = Github::new(std::env::var("ACCESS_TOKEN").unwrap()).unwrap();

    let mut resp = reqwest::get(URL)
        .await?
        .text()
        .await?;

    let doc = Html::parse_document(&resp);
    let themes = Selector::parse("body > main > div > div > div.w-100.w-80-l.ph0 > div > section > a").unwrap();

    for theme in doc.select(&themes) {
        let url = theme.value().attr("href").unwrap();
        resp = reqwest::get(url)
            .await?
            .text()
            .await?;
        let page = Html::parse_document(&resp);
        let git_url = Selector::parse("body > main > article > div.flex-l.bg-light-gray > div:nth-child(1) > div:nth-child(2) > div > a:nth-child(1)").unwrap();
        let mut git_link: &str = "";
        for link in page.select(&git_url){
            git_link = link.value().attr("href").unwrap();
        }
        let tags_selector = Selector::parse("body > main > article > div.flex-l.bg-light-gray > div:nth-child(1) > div:nth-child(1) > ul > li.mb2.mt4 > a").unwrap();
        let mut tags: Vec<&str> = Vec::new();
        for tag_elem in page.select(&tags_selector){
            let tag = tag_elem.text().collect::<Vec<_>>();
            tags.push(&tag[0]);
        }
        let path = Path::new(&git_link);
        let owner = path.parent().unwrap().file_stem().unwrap().to_str().unwrap();
        let name = path.file_stem().unwrap().to_str().unwrap();
        println!("{:?}", tags);
        println!("{:?}", owner);
        println!("{:?}", name);
        let query = format!(r#"query {{ repository(name: "{}", owner: "{}"){{name,stargazers{{totalCount}},forks{{totalCount}},updatedAt}}}}"#, name, owner);
        println!("{}", query);
        let (header, status, json) = g.query::<Value>(
            &Query::new_raw(query)
        ).unwrap();
    }

    Ok(())
}

