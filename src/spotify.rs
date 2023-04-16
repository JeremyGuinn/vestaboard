use rspotify::{Credentials, AuthCodeSpotify, OAuth, scopes, prelude::OAuthClient, model::{AdditionalType, Country, Market}};

use crate::envconfig;

pub struct SpotifySong {
    pub name: String,
    pub artist: String,
}

pub async fn get_currently_playing_song(spotify: &AuthCodeSpotify) -> Option<SpotifySong> {
    let result = spotify
        .current_playing(
            Some(Market::Country(Country::UnitedStates)), 
            Option::<Vec<&AdditionalType>>::None
        )
        .await;

    let context_opt = match result {
        Ok(_) => result.unwrap(),
        Err(_) => return None,
    };

    let context = match context_opt {
        Some(_) => context_opt.unwrap(),
        None => return None,
    };

    match context.item.unwrap() {
        rspotify::model::PlayableItem::Track(track) => {
            return Some(SpotifySong {
                name: track.name,
                artist: track.artists[0].name.clone(),
            });
        }
        rspotify::model::PlayableItem::Episode(episode) => {
            return Some(SpotifySong {
                name: episode.name,
                artist: episode.show.name,
            });
        }
    }
}

pub async fn login_to_spotify() -> AuthCodeSpotify {
    let spotify_id = envconfig::get::<String>("spotify_id");
    let spotify_secret = envconfig::get::<String>("spotify_secret");

    let creds = Credentials {
        id: spotify_id,
        secret: Some(spotify_secret)
    };
    
    let oauth = OAuth {
        redirect_uri: "http://localhost:8888/callback".to_string(),
        scopes: scopes!("user-read-currently-playing"),
        ..Default::default()
    };

    let spotify = AuthCodeSpotify::new(creds, oauth);
    let spotify_auth_url = spotify.get_authorize_url(false).unwrap();
    spotify.prompt_for_token(&spotify_auth_url).await.unwrap();

    return spotify;
}