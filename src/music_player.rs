use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use std::fs::File;
use std::io::BufReader;

use crate::message_types::ItemTag;

pub struct MusicPlayer<'a> {
    output_stream_handle: &'a OutputStreamHandle,
    playing_sink: rodio::Sink,
    currently_playing: ItemTag,
}

impl<'a> MusicPlayer<'a> {
    pub fn new (starting_item: ItemTag, output_stream_handle: &'a OutputStreamHandle) -> Self {

        let sink = Sink::try_new(&output_stream_handle).unwrap();

        //println!("filepath: {}", mp.currently_playing.path.clone());
        let file = BufReader::new(File::open(starting_item.path.clone()).unwrap());

        let source = Decoder::new(file).unwrap();
        sink.append(source);

        //mp.playing_sink.play();
        //mp.playing_sink.sleep_until_end();

        //mp.pause();
        let mp = MusicPlayer {
            output_stream_handle,
            playing_sink: sink,
            currently_playing: starting_item,
        };
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
    pub fn change_now_playing(self: &mut Self, item: ItemTag) {
        let source = Decoder::new(BufReader::new(File::open(item.path.clone()).unwrap())).unwrap();
        self.playing_sink.stop();
        self.playing_sink = Sink::try_new(self.output_stream_handle).unwrap();
        self.playing_sink.append(source);
    }
}
