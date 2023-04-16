mod convert;
mod envconfig;
mod spotify;

use chrono::{DateTime, Duration, Local};
use rspotify::AuthCodeSpotify;

#[tokio::main]
async fn main() {
    let vb_api_key = envconfig::get::<String>("vestaboard_api_key");
    let vb_api_secret = envconfig::get::<String>("vestaboard_api_secret");

    let openweather_key = envconfig::get::<String>("openweather_key");
    let openweather_location = envconfig::get::<String>("openweather_location");
    let openweather_units = envconfig::get::<String>("openweather_units");

    let spotify = spotify::login_to_spotify().await;

    let board: &mut Vec<Vec<i32>> = &mut vec![vec![0; 22]; 6];

    let mut last_time = String::new();
    let mut last_spotify_check: DateTime<Local> = Default::default();
    let mut last_weather_check: DateTime<Local> = Default::default();

    loop {
        let current_time = Local::now().format("%H:%M").to_string();
        if current_time != last_time {
            update_time_and_day(board);
            last_time = current_time;
        }

        if duration_passed(last_spotify_check, Duration::seconds(5)) {
            update_spotify_information(board, &spotify).await;
            last_spotify_check = Local::now();
        }

        if duration_passed(last_weather_check, Duration::minutes(5)) {
            update_weather_information(
                board,
                &openweather_location,
                &openweather_units,
                &openweather_key,
            )
            .await;
            last_weather_check = Local::now();
        }

        let client = vestalia::Vestaboard::new(vb_api_key.to_string(), vb_api_secret.to_string());

        let _ = client.characters(board.clone()).await;
    }
}

fn duration_passed(last_check: DateTime<Local>, duration: Duration) -> bool {
    let now: DateTime<Local> = Local::now();
    now - last_check > duration
}

fn update_time_and_day(board: &mut Vec<Vec<i32>>) {
    let now = Local::now();

    let time = now.format("%H:%M").to_string();
    for (i, c) in time.chars().enumerate() {
        board[0][i] = convert::char_to_int(c);
    }

    let day_str = now.format("%A").to_string().to_lowercase();
    let day_str_len = day_str.chars().count();
    let offset = 11 - (day_str_len) / 2;
    for (i, c) in day_str.chars().enumerate() {
        board[0][offset + i] = convert::char_to_int(c);
    }
}

async fn update_spotify_information(board: &mut Vec<Vec<i32>>, spotify: &AuthCodeSpotify) {
    board[2] = vec![0; 22];
    board[3] = vec![0; 22];
    board[4] = vec![0; 22];

    let currently_playing_opt = spotify::get_currently_playing_song(spotify).await;
    if currently_playing_opt.is_some() {
        let song = currently_playing_opt.unwrap();

        let now_playing_line =
            vestalia::format::convert_line("now playing:".to_string(), "center").unwrap();
        board[2] = now_playing_line;

        let song_line = vestalia::format::convert_line(song.name, "center").unwrap();
        board[3] = song_line;

        let artist_line = vestalia::format::convert_line(song.artist, "center").unwrap();
        board[4] = artist_line;
    }
}

async fn update_weather_information(
    board: &mut Vec<Vec<i32>>,
    location: &str,
    units: &str,
    key: &str,
) {
    let response = openweathermap::weather(location, units, "en", key).await;

    if let Ok(weather) = response {
        let temp_str = format!("{:.0}°", weather.main.temp).trim().to_string();
        let temp_str_len = temp_str.chars().count();
        for (i, c) in temp_str.chars().enumerate() {
            board[0][22 - temp_str_len + i] = convert::char_to_int(c);
        }
    } else {
        println!("Error: {:?}", response);
    }
}
