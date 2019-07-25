extern crate reqwest;
extern crate select;
extern crate url;

use url::Url;
use select::document::Document;
use select::predicate::Name;
use std::collections::HashSet;
use std::env;
use std::error::Error;
use std::io::Read;
use std::time::Instant;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    // begin timing
    let start = Instant::now();
    // Set the url to follow
    let website = &args[1];
    println!("{}", website.to_string() + " being crawled");

    // crawl the allowed pages
    println!("Parsing URL");
    let mut url = Url::parse(website)?;

    // robots.txt checks
    println!("Checking robots.txt");

    let mut all_site_urls = HashSet::<Url>::new();
    let mut site_urls = HashSet::<Url>::new();
    robots_parser(&mut url, &mut all_site_urls)?;

    println!("Getting base URL");
    let base_url = Url::parse(website)?;

    println!("Reading document");
    let mut res = reqwest::get(base_url.as_ref())?;
    let text = res.text().expect("response text");
    let document = Document::from_read(::std::io::Cursor::new(text.into_bytes())).expect("Document from_read");
    // let document = Document::from_read(res)?;

    println!("Getting all links");
    let mut loop_cnt = 0;
    match site_walker(&mut all_site_urls, &mut site_urls, website, &document, &mut loop_cnt){
        Ok(_) => println!("All links gathered"),
        Err(err) => println!("Encountered an error while crawling: {}", err),
    };

    // populate blocked urls into all_site_urls

    println!("Crawled in {} seconds", start.elapsed().as_secs());
    for item in site_urls {
        println!("{}", item);
    }
    // generate JSON output
    //generate_json()
    Ok(())
}

fn robots_parser(url: &mut Url, all_site_urls: &mut HashSet::<Url>) -> Result<(), Box<dyn Error>> {
    // If there is a robots.txt that disallows web crawling we should stop crawling to be in line
    // with internet standards

    url.set_path("/robots.txt");

    let mut res = reqwest::get(url.as_ref())?;
    let base_parser = Url::options().base_url(Some(&url));
    let mut link;
    let mut body = String::new();
    res.read_to_string(&mut body)?;

    // println!("Body:\n{}", body);
    let mut this_bot = false;

    // iterate over file
    for line in body.lines() {
        line.to_string().retain(|c| c != ' ');
        if line.contains("User-agent") {
            let agent = line.split(":").last().unwrap().trim_start();
            if agent == "*" {
                this_bot = true;
            }
            else {
                this_bot = false;
            }
        }
        if line.contains("Disallow") && this_bot {
            let path = line.split(":").last().unwrap().trim_start();
            if path == "/" {
                // we cannot crawl this site
                link = base_parser.parse(path);
                all_site_urls.insert(link.unwrap());
                break;
            }
            else {
                link = base_parser.parse(path);
                all_site_urls.insert(link.unwrap());
            }
        }
    }

    Ok(())
}

fn site_walker(all_site_urls: &mut HashSet::<Url>, site_urls: &mut HashSet::<Url>, website: &str, document: &Document, loop_cnt: &mut i32) -> Result<(), Box<dyn Error>> {

    *loop_cnt += 1;
    let url = Url::parse(website)?;
    let base_url = Url::parse(website)?;
    let mut item_short = url;
    let base_parser = Url::options().base_url(Some(&base_url));

    let links: HashSet<Url> = document
        .find(Name("a"))
        .filter_map(|n| n.attr("href"))
        .filter_map(|link| base_parser.parse(link).ok())
        .collect();

    for item in links{
        if item.as_str().contains(website) && !all_site_urls.contains(&item){
            let mut res = reqwest::get(item.as_ref())?;
            let text = res.text().expect("response text");
            let document = Document::from_read(::std::io::Cursor::new(text.into_bytes())).expect("Document from_read");
            // strip queries etc
            item_short.set_path(item.path());
            site_urls.insert(item_short.clone());
            // store full
            all_site_urls.insert(item);

            match site_walker(all_site_urls, site_urls, website, &document, loop_cnt){
                Ok(_) => print!("."),
                Err(err) => println!("\nEncountered an issue while crawling: {}", err),
            };
        }
    }
    Ok(())
}

//fn generate_json () {
    // outputs a JSON compliant file of all crawled pages
//}
