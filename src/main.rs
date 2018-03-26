extern crate reqwest;
extern crate select;

use std::io::Read;
use select::document::Document;
use select::predicate::{Predicate, Attr, Class, Name, Element};

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
        println!("{}", issue_node.text());
        let issue_url = issue_node.attr("href").unwrap();
        let issue_page = reqwest::get(issue_url)
            .unwrap()
            .text()
            .unwrap();
        let issue_doc = Document::from(issue_page.as_str());

        // only crawl issue with News & Blog Posts section
        for news_blog_posts in issue_doc.find(Name("ul")).take(1) {
            for post in news_blog_posts.children() {
                match post.first_child() {
                    Some(_) => if post.first_child().unwrap().is(Element) {
                        let link = post.first_child().unwrap();
                        println!("\t{} --> {}", link.text(), link.attr("href").unwrap());
                    },
                    _ => print!(""),
                }
            }
        }
    }
}

