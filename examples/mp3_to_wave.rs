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

// We're using a fake file here to make docs happy without bundling an MP3 in this project
#[cfg(doc)]
static MP3: &[u8] = &[0u8; 512];
#[cfg(not(doc))]
static MP3: &[u8] = include_bytes!("../gs-16b-2c-44100hz.mp3");
use hound;
use std::path::Path;
use threepm::mp3::Mp3;

fn main() {
    println!("mp3_to_wave start");
    let mut mp3dec = Mp3::new();
    let mut mp3_slice = &MP3[0..];
    let mut bytes_left = mp3_slice.len() as i32;

    // find the first sync word so we can skip over headers to our mp3 data
    let start = Mp3::find_sync_word(&mp3_slice);
    bytes_left -= start;
    println!("mp3ptr {:?}", mp3_slice.as_ptr(),);
    println!("start of mp3 audio data: {}", start);

    // Update our MP3 pointer to skip past the id3 tags
    mp3_slice = &mp3_slice[start as usize..];

    // the first frame of mp3 data can be used to determine the audio format
    let mut frame = mp3dec.get_next_frame_info(mp3_slice).unwrap();
    println!("info: {:?}", frame);

    // Set our Wave metadata based on mp3 audio format
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

    // mpeg1 layer3 uses 1152 samples per frame.
    // each sample is 2 bytes long, so we need 2304 bytes to store 1 frame of data
    const BUFF_LEN: usize = 2304;
    let mut buf = [0i16; BUFF_LEN];

    while newlen > 0 {
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
