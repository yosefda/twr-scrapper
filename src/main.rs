extern crate reqwest;
extern crate select;

use select::document::Document;
use select::predicate::{Predicate, Attr, Class, Name};

const TWR_ARCHIVE_URL: &str = "https://this-week-in-rust.org/blog/archives/index.html";

#[derive(Debug)]
struct Issue {
    title: String,
    url: String,
}

#[derive(Debug)]
struct Article {
    title: String,
    url: String,
}

fn main() {
    let issues = get_issues(TWR_ARCHIVE_URL);

    for issue in issues {
        println!("{}", issue.title);

        // @todo refactor these lines using get_articles()
        let issue_page = download_url(issue.url.as_str());
        let issue_doc = Document::from(issue_page.as_str());

        // only crawl issue with News and Blog Posts section
        // @todo add ability to add more sections e.g. notable links
        if issue_doc.find(Attr("id", "news-blog-posts")).count() == 0
            && issue_doc.find(Attr("id", "blog-posts")).count() == 0 {
            continue;
        }

        for news_blog_posts in issue_doc.find(Name("ul")).take(1) {
            for post in news_blog_posts.children() {
                for link in post.find(Name("a")) {
                    println!("\t{} --> {}", link.text(), link.attr("href").unwrap());
                }
            }
        }
        print!("\n");
    }
}

fn download_url(url: &str) -> String {
    reqwest::get(url)
        .unwrap()
        .text()
        .unwrap()
}


fn get_issues(archive_url: &str) -> Vec<Issue> {
    let mut issues = Vec::new();

    // parse archive page to get urls of previous issues
    let archive_page = download_url(TWR_ARCHIVE_URL);
    let archive_doc = Document::from(archive_page.as_str());
    for issue_node in archive_doc.find(Class("col-sm-8").descendant(Name("a"))) {
        issues.push(Issue {
            title: issue_node.text(),
            url: issue_node.attr("href").unwrap().to_owned(),
        });
    }

    issues
}

// @todo finish this function
fn get_articles(issue_url: &str) -> Vec<Article> {
    let mut article_entries = Vec::new();

    // parse page to get list of article entries
    let issue_page = download_url(issue_url);
    let issue_doc = Document::from(issue_page.as_str());

    // @todo put parsing of issue doc in separate function

    article_entries
}

