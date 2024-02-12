use godot::engine::{audio_stream_wav, AudioStreamWav};
use godot::prelude::*;
use std::io::Cursor;
use wav;

/// Generate a Godot AudioStreamWav from the given 8-bit WAV data
pub fn sound_from_bytes(data: Vec<u8>) -> Result<Gd<AudioStreamWav>, String> {
    let mut file = Cursor::new(data);
    let (header, data) = wav::read(&mut file).map_err(|_| "Invalid WAV data!")?;
    match data {
        wav::BitDepth::Eight(mut d) => {
            let mut wav = AudioStreamWav::new_gd();
            // https://docs.godotengine.org/en/stable/classes/class_audiostreamwav.html#class-audiostreamwav-property-data
            for byte in &mut d {
                *byte -= 128;
            }
            wav.set_format(audio_stream_wav::Format::FORMAT_8_BITS);
            wav.set_data(PackedByteArray::from(&d[..]));
            wav.set_mix_rate(header.sampling_rate as i32);
            Ok(wav)
        },
        wav::BitDepth::Sixteen(d) => {
            let mut wav = AudioStreamWav::new_gd();
            let d: Vec<u8> = d.iter().flat_map(|b|b.to_le_bytes()).collect();
            wav.set_format(audio_stream_wav::Format::FORMAT_16_BITS);
            wav.set_data(PackedByteArray::from(&d[..]));
            wav.set_mix_rate(header.sampling_rate as i32);
            Ok(wav)
        }
        _ => Err(format!("Unsupported WAV format: {}", header.audio_format)),
    }
}
