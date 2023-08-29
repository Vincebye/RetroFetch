
use chrono::{Datelike, Local};
use clap::{Arg, Command};
use reqwest::{blocking::Client, header, Proxy};
use url::Url;
use regex::Regex;
//use std::fs;
use std::fs::File;
use std::io::{prelude::*, BufReader};

mod logg;
use logg::ColorLogger;
fn url_to_domain(url: &str) -> String {
    match Url::parse(url) {
        Ok(parsed_url) => parsed_url.host_str().unwrap_or("").to_string(),
        Err(_) => "".to_string(),
    }
}
fn generate_filename(target: &str, ttype: &str) -> String {
    let now = Local::now();
    let year = now.year();
    let month = now.month();
    let day = now.day();
    format!("{}_{}_{}_{:02}_{:02}", target, ttype, year, month, day)
}
fn find_replace_fuzz_parameters(url_str: &str, fuzz_value: &str)->String  {
    let re = Regex::new(r"(\?|&)([^=&]+)=([^&]*)").unwrap();

    let modified_url = re.replace_all(url_str, format!("$1$2={}", fuzz_value).as_str());

    modified_url.into()
}
fn analyse_urls(urls: Vec<&str>) -> Vec<String> {
    let blacklist = vec![
        ".jpg", ".jpeg", ".png", ".gif", ".pdf", ".svg", ".json", ".css", ".js", ".webp", ".woff",
        ".woff2", ".eot", ".ttf", ".otf", ".mp4", ".txt",";",
    ];
    let mut filted_urls = Vec::new();
    for url in urls {
        let mut is_blacklisted = false;
        for suffix in &blacklist {
            if url.ends_with(suffix) {
                is_blacklisted = true;
                break;
            }
        }
        if !is_blacklisted {
            let fuzz_url = find_replace_fuzz_parameters(url, "FUZZ");
            filted_urls.push(fuzz_url)
        }
    }
    filted_urls
}

 fn write_data_to_file(data:Vec<String>,filename:String)->std::io::Result<()>{
    
    let mut file=File::create(filename)?;
    for line in data{
        file.write_all(line.as_bytes())?;
        file.write(b"\n")?;
    }
    Ok(())
}

fn single_wayback(url:&str,client:Client)-> Result<(), Box<dyn std::error::Error>>{
    let domain = url_to_domain(url);
    let _wayback_uri = format!(
        "https://web.archive.org/cdx/search/cdx?url={}/*&output=txt&collapse=urlkey&fl=original&page=/",
        domain,
    );
    log::info!("Prepare to request {}",_wayback_uri);
    
    let body = client
        .get(_wayback_uri)
        .header(header::USER_AGENT, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.36")
        .header(header::ACCEPT_LANGUAGE, "en-US,en;q=0.5")
        .send()?
        .text()?;
    log::info!("The {} is done",domain);
    let mut urls: Vec<&str> = Vec::new();
    for line in body.split('\n') {
        urls.push(line);
    }

    let filter_urls = analyse_urls(urls);
    let number=filter_urls.capacity();
    log::info!("The data size:{}",number);
    let filename=generate_filename(&domain, "Fuzz");
    let _=write_data_to_file(filter_urls, filename);
    Ok(())
}

static LOGGER: ColorLogger = ColorLogger;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(log::LevelFilter::Info);
    log::info!("Init the logger");
    let matches = Command::new("RetroHistory")
        .arg(
            Arg::new("url")
                .short('u')
                .long("url")
                .value_name("URL")
                .required(false)
                .help("The target you need to find"),
        )
        .arg(
            Arg::new("proxy")
                .short('p')
                .long("proxy")
                .value_name("PROXY")
                .required(false)
                .help("The address of proxy"),
        )
        .arg(
            Arg::new("list")
            .short('l')
            .long("list")
            .value_name("LIST")
            .required(false)
            .help("The address of list to scan"),
        )
        .get_matches();
    let proxy = matches.contains_id("proxy");

    let client = if proxy {
        let proxy_scheme = matches.get_one::<String>("proxy").unwrap();
        let proxy = Proxy::all(proxy_scheme)?;
        Client::builder().proxy(proxy).build()?
    } else {
        Client::new()
    };
    if matches.contains_id("url"){
        let url = matches.get_one::<String>("url").unwrap();
        single_wayback(url, client)?;
    }
    else if matches.contains_id("list") {
        let targets_filename=matches.get_one::<String>("list").unwrap();
        let file=File::open(targets_filename).expect("Failed to open");
        let reader=BufReader::new(file);
        for line in reader.lines(){
            if let Ok(line)=line{
                single_wayback(&line, client.clone())?;
            }
        }
        
    }
   
   
    Ok(())
}
