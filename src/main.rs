//extern crate log;
//extern crate env_logger;

extern crate librespot;
//extern crate librespot_metadata;
extern crate tokio_core;
extern crate tokio_io;
extern crate futures;
extern crate rand;
use rand::thread_rng;
use rand::seq::SliceRandom;

use std::env;
use tokio_core::reactor::Core;

use librespot::core::authentication::Credentials;
use librespot::core::config::SessionConfig;
use librespot::core::session::Session;
use librespot::core::spotify_id::SpotifyId;
use librespot::metadata::{Metadata, Track, Playlist};
//use librespot::metadata::


use librespot::playback::audio_backend;
use librespot::playback::player::Player;
use librespot::playback::config::PlayerConfig;

fn main() 
{
    let args: Vec<_> = env::args().collect();

    if args.len() != 4 
    {
        println!("Usage: {} USERNAME PASSWORD PLAYLIST", args[0]);
    }

    //get user and password
    let username = args[1].to_owned();
    let password = args[2].to_owned();

    //get playlist handle
    let uri_split = args[3].split(":");
    let uri_parts: Vec<&str> = uri_split.collect();
    let plist_uri = SpotifyId::from_base62(uri_parts[2]).unwrap();

    //get all the spotify stuff
    let credentials = Credentials::with_password(username, password);
    let session_config = SessionConfig::default();
    let player_config = PlayerConfig::default();
    let backend = audio_backend::find(None).unwrap();

    //get the tokio stuff
    let mut core = Core::new().unwrap();
    let handle = core.handle();

    //get spotify session
    let session = core
        .run(Session::connect(session_config, credentials, None, handle))
        .unwrap();

    //get playlist and player instance
    let (player, _) = Player::new(player_config, session.clone(), None, move || (backend)(None));
    let plist = core.run(Playlist::get(&session, plist_uri)).unwrap();

    //shuffle playlist
    let mut tracks = plist.tracks;
    tracks.shuffle(&mut thread_rng());

    for track_id in tracks 
    {
        let plist_track = core.run(Track::get(&session, track_id)).unwrap();
        println!("now playing: {} ", plist_track.name);
        core.run(player.load(track_id, true, 0)).unwrap();
    }
}