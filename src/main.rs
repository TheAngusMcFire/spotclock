extern crate librespot;
extern crate tokio_core;
extern crate tokio_io;
extern crate futures;
extern crate rand;
use rand::thread_rng;
use rand::seq::SliceRandom;

use std::env;
use std::{thread, time};
use tokio_core::reactor::Core;

use librespot::core::authentication::Credentials;
use librespot::core::config::SessionConfig;
use librespot::core::session::Session;
use librespot::core::spotify_id::SpotifyId;
use librespot::metadata::{Metadata, Track, Playlist};

use librespot::playback::audio_backend;
use librespot::playback::player::Player;
use librespot::playback::config::PlayerConfig;
use librespot::playback::mixer;
use librespot::playback::mixer::MixerConfig;
use librespot::playback::mixer::Mixer;
use librespot::playback::mixer::AudioFilter;


extern 
{
    fn init();
    fn init_port(port : u8, in_out : u8 );
    fn set_port(port : u8, state : u8 );
    fn nano_sleep(mu_sec : u64);
    fn gen_sig();
}

fn write_pulse()
{
    loop
    {
        unsafe{ set_port(7, 1);}
        unsafe{nano_sleep(8);}
        unsafe{ set_port(7, 0);}
        unsafe{nano_sleep(16);}
    }
}

fn get_audio_filter_by_fixed_volume(vol : u16) -> Option<Box<dyn AudioFilter + Send>>
{
    let st : Option<String> = None;
    let mixer : fn(Option<MixerConfig>) -> Box<dyn Mixer> = mixer::find(st).expect("Invalid mixer");

    let mixer_config = MixerConfig {
        card: String::from("default"),
        mixer: String::from("PCM"),
        index: 0
    };

    let mixer_test = (mixer)(Some(mixer_config));
    let audio_filter = mixer_test.get_audio_filter();
    mixer_test.set_volume(vol);
    

    unsafe
    {
        init();
        init_port(7, 1);
        gen_sig(); 
    }

    //write_pulse();

    return audio_filter;
}

fn main() 
{
    let args: Vec<_> = env::args().collect();

    if args.len() != 5 
    {
        eprintln!("Usage: {} USERNAME PASSWORD PLAYLIST VOLUME", args[0]);
        return;
    }

    //get user and password
    let username = args[1].to_owned();
    let password = args[2].to_owned();
    let volume_raw = args[4].parse::<i32>().unwrap_or(20);

    if volume_raw > 100 || volume_raw < 0 {eprintln!("Volume needs to be between 0-100%"); return;}
    let volume = (volume_raw * 655) as u16;

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

    let audio_filter = get_audio_filter_by_fixed_volume(volume);
    //get playlist and player instance
    let (player, _test) = Player::new(player_config, session.clone(), audio_filter, move || (backend)(None));

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