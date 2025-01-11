use godot::classes::{audio_stream_wav, AudioStreamWav};
use godot::prelude::*;
use std::io::Cursor;
use hound;

/// Generate a Godot AudioStreamWav from the given 8-bit WAV data
pub fn sound_from_bytes(data: Vec<u8>) -> Result<Gd<AudioStreamWav>, String> {
    let mut file = Cursor::new(data);
    let mut reader = hound::WavReader::new(&mut file).map_err(|_| "Invalid WAV data!")?;
    
    match reader.spec().bits_per_sample { 
       8 => {
            let mut wav = AudioStreamWav::new_gd();
            // https://docs.godotengine.org/en/stable/classes/class_audiostreamwav.html#class-audiostreamwav-property-data
            let d: Vec<u8> = reader.samples::<i8>().flat_map(|b|b.unwrap().to_le_bytes()).collect();
            let num_samples = reader.duration() as i32;
            wav.set_format(audio_stream_wav::Format::FORMAT_8_BITS);
            wav.set_data(&PackedByteArray::from(&d[..]));
            wav.set_mix_rate(reader.spec().sample_rate as i32);
            wav.set_loop_end(num_samples);
            Ok(wav)
        },
        16 => {
            let num_samples = reader.duration() as i32;
            let mut wav = AudioStreamWav::new_gd();
            let d: Vec<u8> = reader.samples::<i16>().flat_map(|b|b.unwrap().to_le_bytes()).collect();
            wav.set_format(audio_stream_wav::Format::FORMAT_16_BITS);
            wav.set_data(&PackedByteArray::from(&d[..]));
            wav.set_mix_rate(reader.spec().sample_rate as i32);
            wav.set_loop_end(num_samples);
            Ok(wav)
        }
        _ => Err(format!("Unsupported WAV format")),
    }
}
