extern crate reqwest;
extern crate scraper;

use scraper::{Html, Selector};

const URL: &str = "https://themes.gohugo.io/";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
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
        for link in page.select(&git_url){
            let git_link = link.value().attr("href").unwrap();
            println!("{:?}", git_link);
        }
        let tags_selector = Selector::parse("body > main > article > div.flex-l.bg-light-gray > div:nth-child(1) > div:nth-child(1) > ul > li.mb2.mt4 > a").unwrap();
        let mut tags: Vec<&str> = Vec::new();
        for tag_elem in page.select(&tags_selector){
            let tag = tag_elem.text().collect::<Vec<_>>();
            tags.push(&tag[0]);
        }
        println!("{:?}", tags);
    }

    Ok(())
}

