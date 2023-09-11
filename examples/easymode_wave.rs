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

static MP3: &[u8] = include_bytes!("../gs-16b-2c-44100hz.mp3");
use byte_slice_cast::AsByteSlice;
use core::slice::Chunks;
use hound;
use picomp3lib_rs::mp3::{DecodeErr, Mp3};
use std::{fmt, fs::File, io::Write, path::Path};

// Substitute here the size of
const CHUNK_SZ: usize = 512;
const BUFF_LEN: usize = 2304;
use picomp3lib_rs::easy_mode;

fn main() {
    println!("easymode decode start!");
    let mut easy = easy_mode::EasyMode::new();
    let mp3_loader = &mut MP3.chunks(CHUNK_SZ);

    // fill up the mp3 decoder's buffer before starting the decode
    while easy.buffer_free() >= CHUNK_SZ {
        if let Some(mp3data) = mp3_loader.next() {
            easy.add_data(mp3data);
        } else {
            panic!("Out of data!");
        }
    }
    let frame = easy.mp3_info().expect("Could not find MP3 frame in buffer");
    let mut buf = [0i16; BUFF_LEN];

    // Set our Wave metadata based on mp3 audio format
    let spec = hound::WavSpec {
        channels: frame.nChans as u16,
        sample_rate: frame.samprate as u32,
        bits_per_sample: frame.bitsPerSample as u16,
        sample_format: hound::SampleFormat::Int,
    };

    let path: &Path = "audio.wav".as_ref();

    let mut wave_file = hound::WavWriter::create(path, spec).unwrap();

    while easy.buffer_used() > 0 {
        // if the buffer has space for another chunk of data from our source, load it
        if easy.buffer_free() >= CHUNK_SZ {
            if let Some(mp3data) = mp3_loader.next() {
                // no need to check how much was added, we know that it's large enough to fit
                easy.add_data(mp3data);
            }
        }

        match easy.decode(&mut buf) {
            Ok(decoded_samples) => {
                for sample in &buf[0..decoded_samples] {
                    wave_file.write_sample(*sample).unwrap();
                }
            }
            Err(_) => {
                break;
            }
        }
    }
    println!("successful decode. finalising wave file");
    wave_file.flush().unwrap();
}
