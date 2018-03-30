extern crate reqwest;
extern crate select;
extern crate csv;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use select::document::Document;
use select::predicate::{Predicate, Attr, Class, Name};
use std::error::Error;

// @todo remove this after we write to file
use std::io;

/// URL of TWR archive page
const TWR_ARCHIVE_URL: &str = "https://this-week-in-rust.org/blog/archives/index.html";

/// Defines an issue that consist of title and URL
#[derive(Debug)]
struct Issue {
    title: String,
    url: String,
}

/// Defines an article that consists of title and URL
#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
struct Article {
    title: String,
    url: String,
}

fn main() {
    // download archive page
    let archive_page = match download_url(TWR_ARCHIVE_URL) {
        Ok(page) => page,
        Err(err) => {
            println!("Unable to download {}, reason: {}", TWR_ARCHIVE_URL, err.description());
            return;
        }
    };

    // get issues from archive page
    let issues = get_issues(archive_page);
    if issues.is_empty() {
        println!("No issues found");
        return;
    }

    for issue in issues {
        // download issue page
        let issue_page = match download_url(issue.url.as_str()) {
            Ok(page) => page,
            Err(err) => {
                println!("Unable to issue {}, reason: {}", issue.title, err.description());
                continue;
            }
        };

        // get articles from issue page
        let articles = get_articles(issue_page);
        println!("{} ({} articles)", issue.title, articles.len());
        let _csv_result = save_to_csv(articles);
    }
}

/// Downloads HTML string of the given URL
///
/// # Arguments
///
/// * `url` - A string slice that holds the URL
fn download_url(url: &str) -> Result<String, reqwest::Error> {
    let page = reqwest::get(url)?.text()?;
    Ok(page)
}

/// Returns vector of TWR issues from the archive page URL
///
/// # Arguments
///
/// * `archive_url` - A string that holds HTML of the archive page
fn get_issues(archive_page: String) -> Vec<Issue> {
    let mut issues = Vec::new();

    let dom = Document::from(archive_page.as_str());
    for issue_node in dom.find(Class("col-sm-8").descendant(Name("a"))) {
        issues.push(Issue {
            title: issue_node.text(),
            url: issue_node.attr("href").unwrap().to_owned(),
        });
    }

    issues
}

/// Returns vector of articles from the given issue page URL
///
/// # Arguments
///
/// * `issue_url` - A string that holds HTML of the issue page
fn get_articles(issue_page: String) -> Vec<Article> {
    let mut articles = Vec::new();

    let dom = Document::from(issue_page.as_str());

    // ignore issue that doesnt have the section we after
    if dom.find(Attr("id", "news-blog-posts")).count() == 0
        && dom.find(Attr("id", "blog-posts")).count() == 0 {
        return articles;
    }

    // collect the articles
    for news_blog_posts in dom.find(Name("ul")).take(1) {
        for post in news_blog_posts.children() {
            for link in post.find(Name("a")) {
                articles.push(Article {
                    title: link.text(),
                    url: link.attr("href").unwrap().to_owned(),
                });
            }
        }
    }

    articles
}

fn save_to_csv(articles: Vec<Article>) -> Result<(), Box<Error>> {
    let mut wtr = csv::Writer::from_writer(io::stdout());

    for article in articles {
        wtr.serialize(article)?;
    }

    wtr.flush()?;
    Ok(())
}


