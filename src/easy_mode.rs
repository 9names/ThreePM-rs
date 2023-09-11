#![deny(unsafe_op_in_unsafe_fn)]

use crate::contig_buffer;
use crate::mp3::{DecodeErr, MP3FrameInfo, Mp3};

pub struct EasyMode {
    mp3: Mp3,
    buffer: contig_buffer::Buffer,
    sync: bool,
    have_decoded: bool,
}

impl EasyMode {
    /// Construct a new "easy mode" MP3 decoder
    pub const fn new() -> Self {
        EasyMode {
            mp3: Mp3::new(),
            buffer: contig_buffer::Buffer::new(),
            sync: false,
            have_decoded: false,
        }
    }

    /// Add MP3 data to the EasyMode internal MP3 stream buffer
    /// This function will also attempt
    pub fn add_data(&mut self, data: &[u8]) -> usize {
        let bytes_added = self.buffer.load_slice(data);
        if !self.sync {
            let start = Mp3::find_sync_word(self.buffer.borrow_slice());
            if start >= 0 {
                self.buffer.increment_start(start as usize);
                self.sync = true;
                // Also try to get frame info for next frame
                let f = self.mp3.get_next_frame_info(self.buffer.borrow_slice());
                if f.is_ok() {
                    self.have_decoded = true;
                }
            } else {
                // Could not sync with any of the data in the buffer, so most of the data is useless.
                // we could have 3 bytes of sync word, so keep the last 3 bytes
                self.buffer.increment_start(self.buffer.used() - 3);
            }
        }
        bytes_added
    }

    /// How much data is free in the EasyMode internal MP3 stream buffer
    pub fn buffer_free(&self) -> usize {
        self.buffer.available()
    }

    /// How much MP3 data is in the EasyMode internal MP3 stream buffer
    pub fn buffer_used(&self) -> usize {
        self.buffer.used()
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
                let frame = self.mp3.get_last_frame_info();
                let consumed = oldlen as usize - newlen as usize;
                self.buffer.increment_start(consumed);
                self.have_decoded = true;
                Ok(frame.outputSamps as usize)
            }
            Err(e) => Err(e.into()),
        }
    }

    /// Get MP3 metadata from the last MP3 frame decoded
    pub fn mp3_info(&mut self) -> Result<MP3FrameInfo, EasyModeErr> {
        if self.have_decoded {
            Ok(self.mp3.get_last_frame_info())
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
