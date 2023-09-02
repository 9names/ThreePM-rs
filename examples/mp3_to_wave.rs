//! Demo to convert from mp3 to raw audio to validate decoding
//!
//! To verify: run
//! ```cargo run --bin mp3toraw --features="byte-slice-cast"```
//! then
//! ```sox -t raw -r 44100 -b 16 -c 2 -L -e signed-integer audio_raw.bin audio_raw.wav```
//! finally
//! ```mplayer audio_raw.wav```
//! and compare to
//! ```mplayer gs-16b-2c-44100hz.mp3```

static MP3: &[u8] = include_bytes!("../gs-16b-2c-44100hz.mp3");
use byte_slice_cast::AsByteSlice;
use picomp3lib_rs::Mp3;
use std::{fs::File, io::Write, path::Path};
use hound;

fn main() {
    println!("mp3-wav start");
    let mut mp3dec = Mp3::new();
    let mut mp3_slice = &MP3[0..];
    let mut bytes_left = mp3_slice.len() as i32;
    let start = Mp3::find_sync_word(&mp3_slice);
    bytes_left -= start;
    println!("mp3ptr {:?}", mp3_slice.as_ptr(),);
    println!("start of mp3 audio data: {}", start);

    // Update our MP3 pointer to skip past the id3 tags
    mp3_slice = &mp3_slice[start as usize..];

    let mut frame = mp3dec.get_next_frame_info(mp3_slice).unwrap();

    println!("info: {:?}", frame);

    let spec = hound::WavSpec {
        channels: frame.nChans as u16,
        sample_rate: frame.samprate as u32,
        bits_per_sample: frame.bitsPerSample as u16,
        sample_format: hound::SampleFormat::Int,
    };

    let path: &Path = "audio.wav".as_ref();

    let mut writer = hound::WavWriter::create(path, spec).unwrap();

    let mut newlen = bytes_left as i32;
    println!("mp3 len: {:?}", newlen);
    // todo: work out what a sensible buffer length is
    // check decode_len for an idea. decode_len is in bytes
    const BUFF_LEN: usize = 2304;
    let mut buf = [0i16; BUFF_LEN];

    let mut file = File::create("audio_raw.bin").unwrap();
    while newlen > 0 {
        // println!("{:?}, {}", mp3_slice.as_ptr(), newlen);
        newlen = mp3dec.decode(&mp3_slice, newlen, &mut buf).unwrap();
        mp3_slice = &mp3_slice[mp3_slice.len() - (newlen as usize)..];

        // get info about the last frame decoded
        frame = mp3dec.get_last_frame_info();
        if frame.outputSamps <= BUFF_LEN as i32 {
            for sample in &buf[0..(frame.outputSamps as usize)] {
                writer.write_sample(*sample).unwrap();
            }
        } else {
            println!(
                "Decoded frame size {} exceeds buffer size. Assume frame is corrupted",
                frame.outputSamps
            );
        }
    }
    writer.finalize().unwrap();
    drop(mp3dec);
    println!("Should be free now");
}
