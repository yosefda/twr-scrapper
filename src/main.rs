#![recursion_limit = "1024"]

extern crate reqwest;
extern crate select;
extern crate csv;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate clap;
extern crate rayon;
#[macro_use]
extern crate error_chain;

use select::document::Document;
use select::predicate::{Predicate, Attr, Class, Name};
use std::error::Error;
use clap::App;
use rayon::prelude::*;
use std::sync::{Mutex, Arc};

mod errors {
    error_chain!{}
}

use errors::*;

/// URL of TWR archive page
const TWR_ARCHIVE_URL: &str = "https://this-week-in-rust.org/blog/archives/index.html";

/// Defines an issue that consist of title and URL
#[derive(Debug)]
struct Issue {
    title: String,
    url: String,
}

/// Defines an article that consists of title and URL
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "PascalCase")]
struct Article {
    title: String,
    url: String,
}

fn main() {
    if let Err(ref e) = run() {
        println!("error: {}", e);
        for e in e.iter().skip(1) {
            println!("caused by: {}", e);
        }
        if let Some(backtrace) = e.backtrace() {
            println!("backtrace: {:?}", backtrace);
        }
        std::process::exit(1);
    }
}

/// Runs the TWR scrapper
fn run() -> Result<()> {
    // parse cmdline args
    let cli_yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(cli_yaml).get_matches();
    let csv_output = matches.value_of("output").unwrap();

    // download archive page
    let archive_page = download_url(TWR_ARCHIVE_URL)
        .chain_err(|| format!("failed to download archive page {}", TWR_ARCHIVE_URL))?;

    // get issues from archive page
    let issues = get_issues(archive_page);
    if issues.is_empty() {
        println!("No issues found");
        return Ok(());
    }

    // run in parallel collecting articles from every issues
    let articles: Arc<Mutex<Vec<Article>>> = Arc::new(Mutex::new(Vec::new()));
    issues.par_iter().for_each(|issue| {
        // download issue page
        let issue_page = match download_url(issue.url.as_str()) {
            Ok(page) => page,
            Err(err) => {
                println!("Unable to download issue {}, reason: {}", issue.title, err.description());
                return;
            }
        };

        // get articles from issue page
        let issue_articles = get_articles(issue_page);
        println!("Processing {} with {} articles", issue.title, issue_articles.len());
        articles.lock().expect("failed to acquire lock for articles").extend(issue_articles);
    });

    // write to csv
    let _csv_result = save_to_csv(
        articles.lock()
            .expect("failed to acquire lock for articles").to_vec(), csv_output)?;

    Ok(())
}


/// Downloads HTML string of the given URL
///
/// # Arguments
///
/// * `url` - A string slice that holds the URL
fn download_url(url: &str) -> Result<String> {
    let mut response = reqwest::get(url).chain_err(|| "failed GET request")?;
    let page = response.text().chain_err(|| "failed to decode the response")?;
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

/// Saves articles to the given csv file
///
/// # Arguments
///
/// * `articles` - A vector that holds list of articles
/// * `csv_output` - A string slice that holds the path to output csv
fn save_to_csv(articles: Vec<Article>, csv_output: &str) -> Result<()> {
    let mut wtr = csv::Writer::from_path(csv_output)
        .chain_err(|| "failed to created csv writer")?;

    for article in articles {
        wtr.serialize(article).chain_err(|| "failed to serialise article data")?;
    }

    wtr.flush().chain_err(|| "failed to flush csv")?;
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::io::prelude::*;
    use std::fs::File;
    use std::path::Path;
    use std::env;

    fn load_fixture(relative_fixture_path: &str) -> String {
        let project_root_dir = env::current_dir().unwrap();
        let fixtures_root_dir = project_root_dir.join(Path::new("tests/fixtures"));
        let fixture_file_path = fixtures_root_dir.join(Path::new(relative_fixture_path));

        let mut fixture = File::open(fixture_file_path).expect("Unable to open fixture");
        let mut contents = String::new();

        fixture.read_to_string(&mut contents).expect("Unable to read the file");
        contents
    }

    #[test]
    fn test_get_issues() {
        let contents = load_fixture("archive_page.html");

        let issues = get_issues(contents);
        assert_eq!(issues.len(), 230);
        assert_eq!(issues[0].title, "This Week in Rust 227".to_owned());
        assert_eq!(issues[0].url, "https://this-week-in-rust.org/blog/2018/03/27/this-week-in-rust-227/".to_owned());
    }

    #[test]
    fn test_get_articles_from_issue_without_articles() {
        let contents = load_fixture("issue_without_articles.html");

        let articles = get_articles(contents);
        assert!(articles.is_empty());
    }

    #[test]
    fn test_get_articles_from_issue_with_articles() {
        let contents = load_fixture("issue_with_articles.html");

        let articles = get_articles(contents);
        assert!(!articles.is_empty());
        assert_eq!(articles.len(), 15);
        assert_eq!(articles[0].title, "Async/Await VI: 6 weeks of great progress".to_owned());
        assert_eq!(articles[0].url, "https://boats.gitlab.io/blog/post/2018-03-20-async-vi/".to_owned());
    }
}


