use std::collections::HashSet;
use std::{error::Error, env};
use std::fs::File;

use csv::{Trim, ReaderBuilder};

use gotta_fill_em_all::artist::Artist;
use gotta_fill_em_all::song::Song;
use gotta_fill_em_all::output_record::OutputRecord;

use log::{info, warn};

use scraper::{Html, Selector};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize the logger
    pretty_env_logger::formatted_builder()
        .filter(None, log::LevelFilter::Info)
        .init();

    // Get the CSV reader
    let file_path = get_arg(1)?;
    let file = File::open(file_path)?;
    let mut reader = ReaderBuilder::new()
        .trim(Trim::All)
        .has_headers(true)
        .comment(Some(b'#'))
        .from_reader(file);

    // Get a CSV writer
    let mut writer = csv::Writer::from_writer(std::io::stdout());

    // Get the Genius API token
    let token = get_arg(2)?;

    // Get a request client
    let client = reqwest::Client::new();

    // Keep track of which songs has been checked
    let mut checked = HashSet::new();

    // Go through each line in the CSV
    for result in reader.deserialize() {
        let artist: Artist = result?;
        info!("Looking at artist: {}", artist.name);

        // Get every song for the artist
        let mut next_page: Option<u64> = Some(1);
        let mut songs: Vec<Song> = Vec::new();

        // Go through each result page
        while next_page != None {
            let response: serde_json::Value = client
                .get(format!("https://api.genius.com/artists/{}/songs?per_page=50&page={}", artist.id, next_page.unwrap()))
                .bearer_auth(&token)
                .send().await?
                .json().await?;

            // Get the songs for the current page into a vector
            let mut page_songs: Vec<Song> = response
                .get("response").expect("Response field not found")
                .get("songs")
                .map(|s| {
                    serde_json::from_value(s.clone())
                }).expect("Songs field not found")?;

            // Append the songs for the current page
            songs.append(&mut page_songs);

            // Update the next page
            next_page = response
                .get("response").unwrap()
                .get("next_page")
                .map_or_else(|| None, |n| {
                    if n.is_null() {
                        None
                    } else {
                        Some(n.as_u64().unwrap())
                    }
                });
        }

        // Get a CSS selector for the lyrics
        let lyrics_selector = Selector::parse("div[data-lyrics-container=true]").unwrap();

        // Check each song's lyrics
        for song in songs {
            // Check whether the song was already seen
            if checked.contains(&song.id) {
                continue;
            }

            // Scrape the web page for the song
            info!("Looking at song {}", song.full_title);
            let song_page = reqwest::get(&song.url).await?.text().await?;
            let song_page = Html::parse_document(&song_page);

            // Check the lyrics for a hole
            for lyrics in song_page.select(&lyrics_selector) {
                if lyrics.inner_html().contains("?]") {
                    warn!("{} contains hole", song.full_title);

                    let primary_artist = song.primary_artist.get("name").unwrap().as_str().unwrap().to_string();
                    let record = OutputRecord { primary_artist, title: song.title, id: song.id };
                    writer.serialize(record)?;

                    break;
                }
            }

            checked.insert(song.id);
        }
    }

    // Flush CSV buffer to file
    writer.flush()?;
    Ok(())
}

/// Get the nth command line argument
fn get_arg(index: usize) -> Result<String, Box<dyn Error>> {
    env::args().nth(index)
        .ok_or_else(|| From::from(format!("less than {index} arguments provided")))
}
