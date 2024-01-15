use godot::engine::{audio_stream_wav, AudioStreamWav};
use godot::prelude::*;
use std::io::Cursor;
use wav;

/// Generate a Godot AudioStreamWav from the given 8-bit WAV data
pub fn sound_from_bytes(data: Vec<u8>) -> Result<Gd<AudioStreamWav>, &'static str> {
    let mut file = Cursor::new(data);
    let (_, data) = wav::read(&mut file).map_err(|_| "Invalid WAV data!")?;
    match data {
        wav::BitDepth::Eight(d) => {
            let mut wav = AudioStreamWav::new_gd();
            wav.set_format(audio_stream_wav::Format::FORMAT_8_BITS);
            wav.set_data(PackedByteArray::from(&d[..]));
            Ok(wav)
        }
        // FIXME: I am not sure how - if it is possible - to load 16-bit data into Godot WAV
        // wav::BitDepth::Sixteen(d) => {
        //     wav.set_format(audio_stream_wav::Format::FORMAT_16_BITS);
        //     wav.set_data(PackedByteArray::from(&d[..]));
        // }
        _ => Err("Unsupported WAV format"),
    }
}
