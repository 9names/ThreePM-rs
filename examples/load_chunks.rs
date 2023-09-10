//! Demo loading an mp3 in chunks so we can, for example, stream off SD card
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
use picomp3lib_rs::*;
use std::{fmt, fs::File, io::Write};

const BUFF_SZ: usize = 1024;
const CHUNK_SZ: usize = 512;
#[derive(Debug)]
struct Buffer {
    pub mp3_byte_buffer: [u8; BUFF_SZ],
    pub buff_start: usize,
    pub buff_end: usize,
}

impl fmt::Display for Buffer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "start:{} end:{} used:{} avail:{}",
            self.buff_start,
            self.buff_end,
            self.used(),
            self.available()
        )
    }
}

#[allow(unused)]
impl Buffer {
    pub fn new() -> Self {
        Self {
            mp3_byte_buffer: [0u8; BUFF_SZ],
            buff_start: 0,
            buff_end: 0,
        }
    }

    /// How much data is in the buffer
    pub fn used(&self) -> usize {
        self.buff_end - self.buff_start
    }

    /// How much data is available
    pub fn available(&self) -> usize {
        BUFF_SZ - self.used()
    }

    /// How much space is free at the end of the buffer
    pub fn tail_free(&self) -> usize {
        BUFF_SZ - self.buff_end
    }

    /// Shuffle all bytes along so that start of buffer == start of data
    pub fn remove_unused(&mut self) {
        let used: usize = self.used();
        if self.buff_start != 0 {
            for i in 0..used {
                self.mp3_byte_buffer[i] = self.mp3_byte_buffer[i + self.buff_start];
            }
            self.buff_start = 0;
            self.buff_end = used;
        }
    }

    /// Using the provided iterator, load more data into the buffer
    pub fn load_more(&mut self, loader: &mut Chunks<'_, u8>) {
        self.remove_unused();
        while self.available() >= CHUNK_SZ {
            let newdata = loader.next();
            match newdata {
                Some(d) => {
                    for i in 0..d.len() {
                        self.mp3_byte_buffer[self.buff_end] = d[i];
                        self.buff_end += 1;
                    }
                }
                None => {
                    return;
                }
            }
        }
    }

    /// Increment our "start pointer". use this as you consume slices from the start
    pub fn increment_start(&mut self, increment: usize) {
        self.buff_start += increment;
        self.remove_unused();
    }

    /// Return a slice over the remaining data in the buffer
    pub fn get_slice(&self) -> &[u8] {
        &self.mp3_byte_buffer[self.buff_start..self.buff_end]
    }
}

fn main() {
    println!("Adafruit-mp3 decoding start");
    let mut mp3dec = Mp3::new();
    let mp3_loader = &mut MP3.chunks(CHUNK_SZ);

    let mut buffer = Buffer::new();

    buffer.load_more(mp3_loader);
    println!("buffer: {}", buffer);

    let start = Mp3::find_sync_word(buffer.get_slice());
    if start >= 0 {
        let start_usize = start as usize;
        println!("Start: {}", start_usize);
        println!("increment start");
        buffer.increment_start(start_usize);
    }

    let mut frame = mp3dec.get_next_frame_info(buffer.get_slice()).unwrap();

    println!("info: {:?}", frame);

    let mut newlen = buffer.used() as i32;
    println!("mp3 len: {:?}", newlen);
    // todo: work out what a sensible buffer length is
    // check decode_len for an idea. decode_len is in bytes
    const BUFF_LEN: usize = 2304;
    let mut buf = [0i16; BUFF_LEN];
    // 130bytes/chunk;
    println!("buffer: {}", buffer);
    let mut file = File::create("audio_raw.bin").unwrap();
    'decodeloop: while newlen > 0 {
        println!("buffer: {}", buffer);
        // Add data to our buffer if there is room for more
        if buffer.available() >= CHUNK_SZ {
            println!("Loading more data");
            buffer.load_more(mp3_loader);
        } else {
            println!("good with what we've got");
        }
        println!("{:?}, {}", buffer.get_slice().as_ptr(), newlen);
        let oldlen = newlen;
        match mp3dec.decode(buffer.get_slice(), newlen, &mut buf) {
            Ok(newlen) => {
                println!("buffer: {}", buffer);
                let consumed = oldlen as usize - newlen as usize;
                println!("Consumed: {}", consumed);
                println!("increment start");
                if consumed > buffer.used() {
                    println!("huh. out of data.");
                    let remaining = mp3_loader.count();
                    println!("finished with {} more chunks left...", remaining);

                    break 'decodeloop;
                }
                buffer.increment_start(consumed);
                println!("buffer: {}", buffer);
            }
            Err(e) => {
                if e == picomp3lib_rs::DecodeErr::InDataUnderflow {
                    println!("ran out of data while decoding, loading more from file");
                    buffer.load_more(mp3_loader);
                }
            }
        }

        // get info about the last frame decoded
        frame = mp3dec.get_last_frame_info();
        if frame.outputSamps <= BUFF_LEN as i32 {
            file.write_all((&(buf[0..(frame.outputSamps) as usize])).as_byte_slice())
                .unwrap();
        } else {
            println!(
                "Decoded frame size {} exceeds buffer size. Assume frame is corrupted",
                frame.outputSamps
            );
        }
        newlen = buffer.used() as i32;
    }
    file.flush().unwrap();
    drop(mp3dec);
    println!("Should be free now");
}
