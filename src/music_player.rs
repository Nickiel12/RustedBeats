//use rodio::decoder::DecoderError;
use rodio::{Decoder, OutputStreamHandle, Sink, Source};
use std::fs::File;
use std::io::BufReader;
use std::time::{Duration, Instant};
use log::warn;

use crate::message_types::ItemTag;

#[derive(Debug)]
pub enum MusicPlayerError {
    DecoderError,
    IOError,
}

pub struct MusicPlayer<'a> {
    output_stream_handle: &'a OutputStreamHandle,
    playing_sink: rodio::Sink,
    currently_playing: ItemTag,

    current_track_length: Duration,
    started_playing: Instant,
    paused_length: Duration,
}

impl<'a> MusicPlayer<'a> {
    pub fn new(starting_item: ItemTag, output_stream_handle: &'a OutputStreamHandle) -> Self {
        let sink = Sink::try_new(&output_stream_handle).unwrap();

        let file = BufReader::new(File::open(starting_item.path.clone()).unwrap());

        let source = Decoder::new(file).unwrap();

        println!("{:?}", source.total_duration());

        let tmp_length;
        match source.total_duration() {
            None => tmp_length = Duration::from_millis(0),
            Some(length) => tmp_length = length,
        };

        sink.append(source);

        let mut mp = MusicPlayer {
            output_stream_handle,
            playing_sink: sink,
            currently_playing: starting_item,

            current_track_length: Duration::from_millis(0),
            started_playing: Instant::now(),
            paused_length: Duration::from_millis(0),
        };


        mp.current_track_length = tmp_length;

        mp.started_playing = Instant::now();
        mp.pause();
        return mp;
    }

    /// Check if `MediaPlayer` is paused
    pub fn is_paused(&self) -> bool {
        return self.playing_sink.is_paused();
    }

    /// Pause the playback of what is currently playing
    pub fn pause(&mut self) {
        self.paused_length = self.started_playing.elapsed();
        self.playing_sink.pause();
    }

    /// Resume playing what is in the `MediaPlayer`
    pub fn play(&mut self) {
        self.playing_sink.play();
        self.started_playing = Instant::now();
        println!("playing");
    }

    // TODO: set these to return results
    pub fn change_now_playing(&mut self, item: ItemTag) -> Result<(), MusicPlayerError> {
        println!("\n\n switching now playing to: {}", item.path.clone());
        let file = File::open(item.path.clone());

        if file.is_err() {
            return Err(MusicPlayerError::IOError);
        }

        let reader = BufReader::new(file.unwrap());

        let source = Decoder::new(reader);

        match source {
            Err(_err) => return Err(MusicPlayerError::DecoderError),
            Ok(src) => {
                match src.total_duration() {
                    None => self.current_track_length = Duration::from_millis(0),
                    Some(length) => self.current_track_length = length,
                };

                self.playing_sink.stop();
                self.playing_sink = Sink::try_new(self.output_stream_handle).unwrap();
                self.playing_sink.append(src);

                self.started_playing = Instant::now();
                return Ok(())
            }
        }
    }

    /// Get the song's current position (time wise)
    pub fn get_played_time(&self) -> Duration {
        if self.is_paused() {return self.paused_length;}
        else {return self.started_playing.elapsed();}
    }

    /// Get the song's length
    pub fn get_track_length(&self) -> Duration {
        return self.current_track_length;
    }
}
