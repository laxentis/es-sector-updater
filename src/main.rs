use reqwest::header;
use reqwest::header::HeaderMap;
use reqwest::redirect;
use std::fs;
use std::fs::OpenOptions;
use std::io::copy;
use std::io::Cursor;
use std::fs::File;
use tempfile::Builder;

const VERSION:&str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    // TODO: load the values from config file
    let fir = "EPWW";
    let package_name = "Sector_AIRAC_Update";
    let es_path = "C:\\Users\\dawid\\Documents\\EuroScope";
    // Get latest download link from GNG
    let url = format!("http://files.aero-nav.com/{}", fir);
    println!("ES Sector Update version {}", VERSION);
    println!("Getting sector link");
    let website = reqwest::get(
        url
    ).await?.text().await?;
    let document = scraper::Html::parse_document(&website);
    let link_selector = scraper::Selector::parse("td>a").unwrap();
    let links = document.select(&link_selector).map(|x| x.value().attr("href").unwrap()).filter(|x| is_correct_link(x, package_name, "zip"));
    let file_url = links.last().unwrap();
    println!("Got url: {}", file_url);
    // Create a temporary directory to hold the files
    let tmp_dir = Builder::new().prefix("es-sector-updater-").tempdir()?;
    // Configure the client to download the sector archive
    let redirect_policy = redirect::Policy::custom(|attempt| attempt.stop());
    let mut hdr = HeaderMap::new();
    hdr.insert(header::ACCEPT, header::HeaderValue::from_static("text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8"));
    hdr.insert(header::ACCEPT_ENCODING, header::HeaderValue::from_static("gzip, deflate, br"));
    hdr.insert(header::ACCEPT_LANGUAGE, header::HeaderValue::from_static("en-US;q=0.7,en;q=0.3"));
    hdr.insert(header::CONNECTION, header::HeaderValue::from_static("keep-alive"));
    hdr.insert(header::DNT, header::HeaderValue::from_static("1"));
    hdr.insert(header::HOST, header::HeaderValue::from_static("files.aero-nav.com"));
    hdr.insert(header::REFERER, header::HeaderValue::from_static("http://files.aero-nav.com/"));
    hdr.insert("Sec-Fetch-Dest", header::HeaderValue::from_static("document"));
    hdr.insert("Sec-Fetch-Mode", header::HeaderValue::from_static("navigate"));
    hdr.insert("Sec-Fetch-Site", header::HeaderValue::from_static("cross-site"));
    hdr.insert("Sec-Fetch-User", header::HeaderValue::from_static("?1"));
    hdr.insert(header::UPGRADE_INSECURE_REQUESTS, header::HeaderValue::from_static("1"));
    hdr.insert(header::USER_AGENT, header::HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:108.0) Gecko/20100101 Firefox/108.0"));
    let client = reqwest::Client::builder().redirect(redirect_policy).default_headers(hdr).build()?;
    let response = client.get(file_url).send().await?;
    let file_name = tmp_dir.path().join("sector.zip");
    println!("{:?}", file_name);
    println!("Creating file: {}", file_name.display());
    let mut file = OpenOptions::new().read(true).write(true).create(true).open(file_name.to_owned())?;
    let mut content = Cursor::new(response.bytes().await?);
    copy(&mut content, &mut file)?;
    // File is now downloaded and closed; Time to unzip it
    let mut archive = zip::ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = match file.enclosed_name() {
            Some(path) => tmp_dir.path().join(path.to_owned()),
            None => continue,
        };
        if (*file.name()).ends_with('/') {
            fs::create_dir_all(&outpath).unwrap();
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p).unwrap();
                }
                let mut outfile = File::create(&outpath).unwrap();
                copy(&mut file, &mut outfile).unwrap();
            }
        }
    }
    // Archive is unzipped. No longer needed. Deleting it to make it easier to copy all files.
    drop(archive);
    fs::remove_file(file_name)?;


    Ok(())
}

fn is_correct_link(link: &str, package_name: &str, format: &str) -> bool {
    return link.contains(package_name) && link.ends_with(format)
}