extern crate reqwest;
extern crate scraper;

use std::path::Path;
use scraper::{Html, Selector};
use github_gql as gh;
use gh::client::Github;
use gh::query::Query;
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::File;
use std::io::{BufWriter, Write};
use chrono::{Utc};

const URL: &str = "https://themes.gohugo.io/";

#[derive(Debug, Serialize, Deserialize, Clone, Eq, Ord, PartialEq, PartialOrd)]
#[serde(rename_all = "camelCase")]
struct Stargazers{
    total_count: i32
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Forks{
    total_count: i32
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Repository{
    name: String,
    url: String,
    updated_at: String,
    forks: Forks,
    stargazers: Stargazers
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Data{
    repository: Repository
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Response{
    data: Data
}

#[derive(Debug, Serialize, Clone)]
struct Repo{
    repository: Repository,
    tags: Vec<String>
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{

    dotenv().ok();
    let mut g = Github::new(std::env::var("GITHUB_ACCESS_TOKEN").unwrap()).unwrap();

    let mut resp = reqwest::get(URL)
        .await?
        .text()
        .await?;

    let doc = Html::parse_document(&resp);
    let themes = Selector::parse("body > main > div > div > div.w-100.w-80-l.ph0 > div > section > a").unwrap();

    let mut f = BufWriter::new(fs::File::create("README.md").unwrap());
    let now = Utc::now();
    let title = format!("# hugo_stars\nUpdated at {}\n\n", now);
    f.write(title.as_bytes())?;
    f.write(b"|Name|Stars|Forks|Tags|UpdatedAt|\n----|----|----|----|----\n")?;

    let mut repos: Vec<Repo> = Vec::new();
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
        let mut tags: Vec<String> = Vec::new();
        for tag_elem in page.select(&tags_selector){
            let tag = tag_elem.text().collect::<Vec<_>>();
            if tag.len() != 0{
                tags.push(tag[0].to_string());
            }
        }
        let path = Path::new(&git_link);
        let owner = path.parent().unwrap().file_stem().unwrap().to_str().unwrap();
        let name = path.file_stem().unwrap().to_str().unwrap();
        let query = format!(r#"query {{ repository(name: "{}", owner: "{}"){{name,stargazers{{totalCount}},forks{{totalCount}},updatedAt,url}}}}"#, name, owner);
        println!("{}", query);
        let resp = match g.query::<Response>(
            &Query::new_raw(query)
        ){
            Ok((_, _, resp)) => resp.unwrap(),
            Err(_) => continue
        };
        let repo: Repo = Repo{
            repository: resp.data.repository,
            tags: tags
        };
        repos.push(repo);
    }
    repos.sort_by(|a, b| b.repository.stargazers.total_count.cmp(&a.repository.stargazers.total_count));

    for repo in &repos{
        let row: &str = &format!("|[{}]({})|{}|{}|{}|{}|\n", repo.repository.name, repo.repository.url, repo.repository.stargazers.total_count, repo.repository.forks.total_count, repo.tags.join(","), repo.repository.updated_at);
        f.write(row.as_bytes())?;
    }

    let json_file = File::create("themes.json")?;
    serde_json::to_writer(&json_file, &repos)?;

    Ok(())
}

