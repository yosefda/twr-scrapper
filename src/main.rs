extern crate reqwest;
extern crate select;

use select::document::Document;
use select::predicate::{Predicate, Attr, Class, Name};

const TWR_ARCHIVE_URL: &str = "https://this-week-in-rust.org/blog/archives/index.html";

fn main() {
    // download archive page
    let archive_page = reqwest::get(TWR_ARCHIVE_URL)
        .unwrap()
        .text()
        .unwrap();

    // get list of issues and crawl them
    let archive_doc = Document::from(archive_page.as_str());
    for issue_node in archive_doc.find(Class("col-sm-8").descendant(Name("a"))) {
        let issue_url = issue_node.attr("href").unwrap();
        let issue_page = reqwest::get(issue_url)
            .unwrap()
            .text()
            .unwrap();
        let issue_doc = Document::from(issue_page.as_str());

        println!("{}", issue_node.text());

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

