//use rodio::decoder::DecoderError;
use rodio::{Decoder, OutputStreamHandle, Sink};
use std::fs::File;
use std::io::BufReader;

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
}

impl<'a> MusicPlayer<'a> {
    pub fn new(starting_item: ItemTag, output_stream_handle: &'a OutputStreamHandle) -> Self {
        let sink = Sink::try_new(&output_stream_handle).unwrap();

        let file = BufReader::new(File::open(starting_item.path.clone()).unwrap());

        let source = Decoder::new(file).unwrap();
        sink.append(source);

        let mp = MusicPlayer {
            output_stream_handle,
            playing_sink: sink,
            currently_playing: starting_item,
        };

        mp.pause();
        return mp;
    }

    pub fn pause(self: &Self) -> () {
        self.playing_sink.pause();
    }

    pub fn play(self: &Self) -> () {
        self.playing_sink.play();
        println!("playing");
    }

    // TODO: set these to return results
    pub fn change_now_playing(self: &mut Self, item: ItemTag) -> Result<(), MusicPlayerError> {
        println!("\n\n switching now playing to: {}", item.path.clone());
        let file = File::open(item.path.clone());

        if file.is_err() {
            return Err(MusicPlayerError::IOError);
        }

        let reader = BufReader::new(file.unwrap());

        let source = Decoder::new(reader);

        if source.is_err() {
            return Err(MusicPlayerError::DecoderError);
        }

        self.playing_sink.stop();
        self.playing_sink = Sink::try_new(self.output_stream_handle).unwrap();
        self.playing_sink.append(source.unwrap());
        Ok(())
    }
}
