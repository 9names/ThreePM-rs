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
use hound;
use picomp3lib_rs::mp3::{DecodeErr, Mp3};
use std::{fmt, fs::File, io::Write, path::Path};

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

    /// How much data is stored in the buffer
    pub fn used(&self) -> usize {
        self.buff_end - self.buff_start
    }

    /// How much free space is in the buffer
    pub fn available(&self) -> usize {
        BUFF_SZ - self.used()
    }

    /// How much contiguous free space there is at the end of the buffer
    pub fn tail_free(&self) -> usize {
        BUFF_SZ - self.buff_end
    }

    /// Shuffle all bytes along so that start of buffer == start of data
    pub fn remove_unused(&mut self) {
        if self.buff_start != 0 {
            self.mp3_byte_buffer
                .copy_within((self.buff_start)..(self.buff_end), 0);
            let used = self.used().clone();
            self.buff_start = 0;
            self.buff_end = used;
        }
    }

    /// Using the provided Chunks iterator, load more data into the buffer
    pub fn load_more(&mut self, loader: &mut Chunks<'_, u8>) -> bool {
        let mut loaded_some = false;
        // if we need more contiguous space, shuffle the data to the start
        if self.tail_free() < CHUNK_SZ {
            self.remove_unused();
        }
        while self.available() >= CHUNK_SZ {
            if let Some(d) = loader.next() {
                let newend = self.buff_end + d.len();
                self.mp3_byte_buffer[(self.buff_end)..(newend)].copy_from_slice(d);
                self.buff_end = newend;
                loaded_some = true;
            } else {
                return loaded_some;
            }
        }
        loaded_some
    }

    /// Using the provided slice, load more data into the buffer.
    /// Returns the number of bytes consumed
    pub fn load_slice(&mut self, data: &[u8]) -> usize {
        if self.tail_free() < CHUNK_SZ {
            self.remove_unused();
        }
        let loadsize = usize::min(self.tail_free(), data.len());
        let newend = self.buff_end + loadsize;
        self.mp3_byte_buffer[(self.buff_end)..(newend)].copy_from_slice(data);
        self.buff_end = newend;

        loadsize
    }

    /// Increment our "start pointer". use this as you consume slices from the start
    pub fn increment_start(&mut self, increment: usize) {
        self.buff_start += increment;
    }

    /// Return a slice over the remaining data in the buffer
    pub fn borrow_slice(&self) -> &[u8] {
        &self.mp3_byte_buffer[self.buff_start..self.buff_end]
    }

    /// Return a slice over the remaining data in the buffer and update the indexes
    /// this should be safe, since the &mut is active as long as the slice is borrowed
    pub fn take_slice(&mut self) -> &[u8] {
        let start = self.buff_start.clone();
        let end = self.buff_end.clone();
        self.buff_start = 0;
        self.buff_end = 0;

        &self.mp3_byte_buffer[start..end]
    }

    /// Return a slice over some of the data and update the indexes
    /// this should be safe, since the &mut is active as long as the slice is borrowed
    /// if you request more data than is present, you get an error
    pub fn take_subslice(&mut self, slice_size: usize) -> Result<&[u8], ()> {
        if slice_size <= self.used() {
            let start = self.buff_start.clone();
            let end = start + slice_size;
            // update the start of data index to be beyond what we returned
            self.buff_start = end;

            Ok(&self.mp3_byte_buffer[start..end])
        } else {
            Err(())
        }
    }
}

fn main() {
    println!("load_chunks_wave start");
    let mut mp3dec = Mp3::new();
    let mp3_loader = &mut MP3.chunks(CHUNK_SZ);

    let mut buffer = Buffer::new();

    while buffer.available() >= CHUNK_SZ && buffer.load_more(mp3_loader) {}

    // find the first sync word so we can skip over headers to our mp3 data
    let start = Mp3::find_sync_word(buffer.borrow_slice());
    if start >= 0 {
        let start_usize = start as usize;
        println!("Start: {}", start_usize);
        println!("increment start");
        buffer.increment_start(start_usize);
    }

    // the first frame of mp3 data can be used to determine the audio format
    let mut frame = mp3dec.get_next_frame_info(buffer.borrow_slice()).unwrap();
    println!("info: {:?}", frame);

    let mut buffered_data_len = buffer.used() as i32;
    const BUFF_LEN: usize = 2304;
    let mut buf = [0i16; BUFF_LEN];

    // Set our Wave metadata based on mp3 audio format
    let spec = hound::WavSpec {
        channels: frame.nChans as u16,
        sample_rate: frame.samprate as u32,
        bits_per_sample: frame.bitsPerSample as u16,
        sample_format: hound::SampleFormat::Int,
    };

    let path: &Path = "audio.wav".as_ref();

    let mut writer = hound::WavWriter::create(path, spec).unwrap();
    let mut inc = 0;

    while buffered_data_len > 0 {
        inc += 1;
        // if the buffer has space for another chunk of data from our source, load it
        if buffer.available() >= CHUNK_SZ {
            if !buffer.load_more(mp3_loader) {
                println!("Ran into end of file while loading new mp3 data. We still have {buffered_data_len} bytes of mp3 data in the buffer");
            }
        }
        buffered_data_len = buffer.used() as i32;
        let oldlen = buffered_data_len;
        match mp3dec.decode(buffer.borrow_slice(), buffered_data_len, &mut buf) {
            Ok(newlen) => {
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
                let consumed = oldlen as usize - newlen as usize;
                buffer.increment_start(consumed);
            }
            Err(e) => {
                if e == DecodeErr::InDataUnderflow {
                    print!("mp3 decoder reports data underflow. attempting to loading more from file... ");
                    if !buffer.load_more(mp3_loader) {
                        println!("there was no more data, breaking out of decode loop");
                        buffered_data_len = 0;
                    } else {
                        println!("found more data. continuing decode");
                    }
                }
            }
        }
    }
    writer.flush().unwrap();
}
