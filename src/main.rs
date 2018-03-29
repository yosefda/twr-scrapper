extern crate reqwest;
extern crate select;

use select::document::Document;
use select::predicate::{Predicate, Attr, Class, Name};

/// URL of TWR archive page
const TWR_ARCHIVE_URL: &str = "https://this-week-in-rust.org/blog/archives/index.html";

/// Defines an issue that consist of title and URL
#[derive(Debug)]
struct Issue {
    title: String,
    url: String,
}

/// Defines an article that consists of title and URL
#[derive(Debug)]
struct Article {
    title: String,
    url: String,
}

fn main() {
    let issues = get_issues(TWR_ARCHIVE_URL);

    for issue in issues {
        println!("{}", issue.title);

        let articles = get_articles(issue.url.as_str());
        for article in articles {
            println!("\t{} --> {}", article.title, article.url);
        }
    }
}

/// Downloads HTML string of the given URL
///
/// # Arguments
///
/// * `url` - A string slice that holds the URL
fn download_url(url: &str) -> String {
    reqwest::get(url)
        .unwrap()
        .text()
        .unwrap()
}


/// Returns vector of TWR issues from the archive page URL
///
/// # Arguments
///
/// * `archive_url` - A string slice that holds the URL of archive page
fn get_issues(archive_url: &str) -> Vec<Issue> {
    let mut issues = Vec::new();

    // parse archive page to get urls of previous issues
    let archive_page = download_url(archive_url);
    let archive_doc = Document::from(archive_page.as_str());
    for issue_node in archive_doc.find(Class("col-sm-8").descendant(Name("a"))) {
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
/// * `issue_url` - A string slice that holds the URL of an issue page
fn get_articles(issue_url: &str) -> Vec<Article> {
    // parse page to get list of article entries
    let issue_page = download_url(issue_url);
    let issue_doc = Document::from(issue_page.as_str());

    // parsing of issue doc in separate function
    parse_issue_doc(issue_doc)
}

/// Returns vector of articles from the given issue document
///
/// # Arguments
///
/// * `issue_doc` - An issue page Document
fn parse_issue_doc(issue_doc: Document) -> Vec<Article> {
    let mut articles = Vec::new();

    // ignore issue that doesnt have the section we after
    if issue_doc.find(Attr("id", "news-blog-posts")).count() == 0
            && issue_doc.find(Attr("id", "blog-posts")).count() == 0 {
            return articles;
    }

    // collect the articles
    for news_blog_posts in issue_doc.find(Name("ul")).take(1) {
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

