//! A high-level, user friendly Rust abstraction around `ThreePM`. This should be what you want to use wherever possible.
//! # Usage:
//!
//! ```rust
//! use threepm::easy_mode::{EasyMode, EasyModeErr};
//!
//! // In the real code you could include an MP3 in your program using the following line
//! // static MP3: &[u8] = include_bytes!("../gs-16b-2c-44100hz.mp3");
//! // This will stand in for our real MP3 for now to make doc tests pass.
//! static MP3: &[u8] = &[0u8;512];
//! // Size of our fake "sector" to simulate loading data off of a disk
//! const CHUNK_SZ: usize = 512;
//!
//! fn main() {
//!    // Set up our EasyMode decoder
//!    let mut easy = EasyMode::new();
//!    // Set up the source of our MP3 data
//!    let mp3_loader = &mut MP3.chunks(CHUNK_SZ);
//!    // Set up the buffer for the decoded audio data to be stored in
//!    let mut buf = [0i16; 2304];
//!
//!    // skip past the id3 tags and anything else up to the first mp3 sync tag
//!    'initloop: while !easy.mp3_decode_ready() {
//!        while easy.buffer_free() >= CHUNK_SZ {
//!            if let Some(mp3data) = mp3_loader.next() {
//!                easy.add_data_no_sync(mp3data);
//!            } else {
//!                println!("Out of data!");
//!                break 'initloop;
//!            }
//!        }
//!    }
//!    // We're past the header now, so we should be able to correctly decode an MP3 frame
//!    // Metadata is stored in every frame, so check that now:
//!    if let Ok(frame) = easy.mp3_info() {
//!        println!("First MP3 frame info: {:?}", frame);
//!    }
//!    loop {
//!           // if the buffer has space for another chunk of data from our source, load it
//!           if easy.buffer_free() >= CHUNK_SZ {
//!               if let Some(mp3data) = mp3_loader.next() {
//!                   easy.add_data(mp3data);
//!               }
//!           }
//!           // decode the next chunk of mp3
//!           match easy.decode(&mut buf) {
//!               Ok(_decoded_samples) => {
//!                   // Do something with decoded_samples (like play or store them)
//!               }
//!               Err(_e) => {
//!                   // Handle error by aborting, skipping a frame, adding more data, etc.
//!                   // This example will just exit because this is simpler
//!                   break;
//!               }
//!           }
//!       }
//! }
//!
//! ```

#![deny(unsafe_op_in_unsafe_fn)]
use crate::contig_buffer;
use crate::mp3::{DecodeErr, MP3FrameInfo, Mp3};

/// A high-level, user friendly Rust abstraction around `ThreePM`
pub struct EasyMode {
    mp3: Mp3,
    buffer: contig_buffer::Buffer,
    sync: bool,
    have_decoded: bool,
    parsed_id3: bool,
    bytes_to_skip: usize,
    frame_info: Option<MP3FrameInfo>,
}

impl EasyMode {
    /// Construct a new "easy mode" MP3 decoder
    pub const fn new() -> Self {
        EasyMode {
            mp3: Mp3::new(),
            buffer: contig_buffer::Buffer::new(),
            sync: false,
            have_decoded: false,
            parsed_id3: false,
            bytes_to_skip: 0,
            frame_info: None,
        }
    }

    /// Add MP3 data to the EasyMode internal MP3 stream buffer.
    /// This function will also attempt to find the start of stream
    pub fn add_data(&mut self, data: &[u8]) -> usize {
        let bytes_added = self.buffer.load_slice(data);
        let _ = self.find_next_sync_word();
        bytes_added
    }

    /// Every mp3 frame starts with a sync word. Find the next one, and check if it's a valid frame
    pub fn find_next_sync_word(&mut self) -> bool {
        if !self.sync {
            let start = Mp3::find_sync_word(self.buffer.borrow_slice());
            if start >= 0 {
                self.buffer.increment_start(start as usize);
                self.sync = true;
                // Also try to get frame info for next frame
                let f = self.mp3.get_next_frame_info(self.buffer.borrow_slice());
                if let Ok(frame) = f {
                    self.frame_info = Some(frame);
                    self.have_decoded = true;
                }
            } else {
                // Could not sync with any of the data in the buffer, so most of the data is useless.
                // we could have 3 bytes of sync word, so keep the last 3 bytes
                self.buffer.increment_start(self.buffer.used() - 3);
            }
        }
        self.sync
    }

    /// Add MP3 data to the EasyMode internal MP3 stream buffer
    pub fn add_data_no_sync(&mut self, data: &[u8]) -> usize {
        self.buffer.load_slice(data)
    }

    /// How much data is free in the EasyMode internal MP3 stream buffer
    pub fn buffer_free(&self) -> usize {
        self.buffer.available()
    }

    /// How much MP3 data is in the EasyMode internal MP3 stream buffer
    pub fn buffer_used(&self) -> usize {
        self.buffer.used()
    }

    /// Skip over data in the buffer without decoding it
    pub fn buffer_skip(&mut self, count: usize) -> usize {
        let to_remove = core::cmp::min(self.buffer.used(), count);
        self.buffer.increment_start(to_remove);
        to_remove
    }

    /// Skip over ID3 and anything else at the start of an MP3 stream.
    /// Returns true when we've got a valid MP3 frame
    pub fn mp3_decode_ready(&mut self) -> bool {
        if self.buffer_used() == 0 {
            false
        } else {
            if !self.parsed_id3 {
                self.parsed_id3 = true;
                let id3 = Mp3::find_id3v2(self.buffer.borrow_slice());
                self.bytes_to_skip = if let Some(id3) = id3 {
                    // start of header + size of header + length of id3v2 info
                    id3.0 + 10 + id3.1.size
                } else {
                    0
                };
            };
            if self.bytes_to_skip > 0 {
                let bytes_to_skip = core::cmp::min(self.buffer_used(), self.bytes_to_skip);
                self.buffer_skip(bytes_to_skip);
                self.bytes_to_skip -= bytes_to_skip;
            } else {
                let _ = self.find_next_sync_word();
            }

            self.parsed_id3 && self.bytes_to_skip == 0 && self.sync
        }
    }

    /// Decode the next MP3 audio frame after checking that the output buffer is large enough
    pub fn decode(&mut self, output_audio: &mut [i16]) -> Result<usize, EasyModeErr> {
        let buffered_data_len = self.buffer.used() as i32;
        let oldlen = buffered_data_len as usize;
        let next_frame = self.mp3.get_next_frame_info(self.buffer.borrow_slice())?;
        let samples = next_frame.outputSamps as usize;
        if output_audio.len() < samples {
            // Don't decode if there isn't enough space in the buffer
            Err(EasyModeErr::AudioBufferTooSmall)
        } else {
            match self
                .mp3
                .decode(self.buffer.borrow_slice(), buffered_data_len, output_audio)
            {
                Ok(newlen) => {
                    self.have_decoded = true;
                    let consumed = oldlen - newlen as usize;
                    self.buffer.increment_start(consumed);
                    self.frame_info = Some(next_frame);
                    Ok(samples)
                }
                Err(e) => Err(e.into()),
            }
        }
    }

    /// Decode the next MP3 audio frame assuming that the output buffer is large enough.
    ///
    /// # Safety
    ///
    /// Ensure output buffer is larger than your MP3 frame or this will totally ruin your day
    pub unsafe fn decode_unchecked(
        &mut self,
        output_audio: &mut [i16],
    ) -> Result<usize, EasyModeErr> {
        let buffered_data_len = self.buffer.used() as i32;
        let oldlen = buffered_data_len;
        match self
            .mp3
            .decode(self.buffer.borrow_slice(), buffered_data_len, output_audio)
        {
            Ok(newlen) => {
                self.frame_info = Some(self.mp3.get_last_frame_info());
                // we just set this so the unwrap should never fail
                let output_samps = unsafe { self.frame_info.unwrap_unchecked().outputSamps };
                let consumed = oldlen as usize - newlen as usize;
                self.buffer.increment_start(consumed);
                self.have_decoded = true;
                Ok(output_samps as usize)
            }
            Err(e) => Err(e.into()),
        }
    }

    /// Get MP3 metadata from the last MP3 frame decoded
    pub fn mp3_info(&mut self) -> Result<MP3FrameInfo, EasyModeErr> {
        if let Some(frameinfo) = self.frame_info {
            Ok(frameinfo)
        } else {
            let frame = self.mp3.get_next_frame_info(self.buffer.borrow_slice())?;
            Ok(frame)
        }
    }
}

/// Errors that occur when calling the decode function
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub enum EasyModeErr {
    Okay,
    InDataUnderflow,
    MaindataUnderfow,
    FreeBitrateSync,
    OutOfMemory,
    NullPointer,
    InvalidFrameheader,
    InvalidSideinfo,
    InvalidScalefact,
    InvalidHuffcodes,
    InvalidDequantize,
    InvalidImdct,
    InvalidSubband,
    Unknown,
    InvalidError,
    AudioBufferTooSmall,
}

impl From<DecodeErr> for EasyModeErr {
    fn from(value: DecodeErr) -> Self {
        match value {
            DecodeErr::Okay => EasyModeErr::Okay,
            DecodeErr::InDataUnderflow => EasyModeErr::InDataUnderflow,
            DecodeErr::MaindataUnderfow => EasyModeErr::MaindataUnderfow,
            DecodeErr::FreeBitrateSync => EasyModeErr::FreeBitrateSync,
            DecodeErr::OutOfMemory => EasyModeErr::OutOfMemory,
            DecodeErr::NullPointer => EasyModeErr::NullPointer,
            DecodeErr::InvalidFrameheader => EasyModeErr::InvalidFrameheader,
            DecodeErr::InvalidSideinfo => EasyModeErr::InvalidSideinfo,
            DecodeErr::InvalidScalefact => EasyModeErr::InvalidScalefact,
            DecodeErr::InvalidHuffcodes => EasyModeErr::InvalidHuffcodes,
            DecodeErr::InvalidDequantize => EasyModeErr::InvalidDequantize,
            DecodeErr::InvalidImdct => EasyModeErr::InvalidImdct,
            DecodeErr::InvalidSubband => EasyModeErr::InvalidSubband,
            DecodeErr::Unknown => EasyModeErr::Unknown,
            DecodeErr::InvalidError => EasyModeErr::InvalidError,
        }
    }
}
