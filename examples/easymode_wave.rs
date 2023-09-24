//! Demo using the higher-level mp3 decoder "easymode"
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
use std::path::Path;

/// Size of our fake "sector" to simulate loading data off of a disk
const CHUNK_SZ: usize = 512;

/// The length of our audio output buffer
/// This is correct for MPEG-1 Layer 3, MPEG-2 Layer 3 is smaller so should be fine
const BUFF_LEN: usize = 2304;
use threepm::easy_mode::{EasyMode, EasyModeErr};

fn main() {
    println!("easymode decode start!");
    let mut easy = EasyMode::new();
    let mp3_loader = &mut MP3.chunks(CHUNK_SZ);

    // skip past the id3 tags and anything else up to the first mp3 sync tag
    while !easy.mp3_decode_ready() {
        while easy.buffer_free() >= CHUNK_SZ {
            if let Some(mp3data) = mp3_loader.next() {
                easy.add_data_no_sync(mp3data);
            } else {
                panic!("Out of data!");
            }
        }
    }

    let frame = easy.mp3_info().expect("Could not find MP3 frame in buffer");
    println!("First MP3 frame info: {:?}", frame);
    let mut buf = [0i16; BUFF_LEN];

    // Set our Wave metadata based on mp3 audio format
    let spec = hound::WavSpec {
        channels: frame.nChans as u16,
        sample_rate: frame.samprate as u32,
        bits_per_sample: frame.bitsPerSample as u16,
        sample_format: hound::SampleFormat::Int,
    };

    // Set up our wave file writer
    let path: &Path = "audio.wav".as_ref();
    let mut wave_file = hound::WavWriter::create(path, spec).unwrap();

    loop {
        // if the buffer has space for another chunk of data from our source, load it
        if easy.buffer_free() >= CHUNK_SZ {
            if let Some(mp3data) = mp3_loader.next() {
                // no need to check how much was added, we know that it's large enough to fit
                easy.add_data(mp3data);
            }
        }

        // decode the next chunk of mp3
        match easy.decode(&mut buf) {
            Ok(decoded_samples) => {
                // We successfully decoded! Write this data into a wave file
                for sample in &buf[0..decoded_samples] {
                    wave_file.write_sample(*sample).unwrap();
                }
            }
            Err(e) => {
                // We can recover from data underflow if there's still some more data in our MP3 file
                if e == EasyModeErr::InDataUnderflow {
                    println!("mp3 decoder reports data underflow. attempting to loading more from file... ");
                    if let Some(mp3data) = mp3_loader.next() {
                        // no need to check how much was added, we know that it's large enough to fit
                        easy.add_data(mp3data);
                    } else {
                        println!("there is no more data to add, breaking out of decode loop");
                        break;
                    }
                } else {
                    // Assume all other errors are unrecoverable
                    println!("we hit error {e:?} while decoding, give up on decoding any more");
                    break;
                }
            }
        }
    }
    println!("successful decode. finalising wave file");
    wave_file.flush().unwrap();
}
