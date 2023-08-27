/// High-level MP3 library wrapping functions.
///
/// This version uses an opaque pointer, just like the C++ wrapper does,
/// with all data associated with MP3 decoding living in static memory declared by the C library.
use crate::{DecodeErr, MP3FrameInfo};
use core::ffi::c_void;

pub struct Mp3 {
    pub mp3dec: *mut c_void,
}

impl Mp3 {
    pub fn new() -> Mp3 {
        let dec = unsafe { crate::ffi::MP3InitDecoder() };
        Mp3 { mp3dec: dec }
    }

    /// Find the offset of the next sync word in the MP3 stream. Use this to find the next frame
    pub fn find_sync_word(mp3buf: &[u8]) -> i32 {
        let mp3baseptr: *const u8 = mp3buf.as_ptr();
        unsafe { crate::ffi::MP3FindSyncWord(mp3baseptr, mp3buf.len() as i32) }
    }

    /// Get info for the last decoded MP3 frame
    pub fn get_last_frame_info(&mut self) -> MP3FrameInfo {
        let mut frame = MP3FrameInfo::new();
        unsafe { crate::ffi::MP3GetLastFrameInfo(self.mp3dec, &mut frame) };
        frame
    }

    /// Get info for the next MP3 frame
    pub fn get_next_frame_info(&mut self, mp3buf: &[u8]) -> Result<MP3FrameInfo, DecodeErr> {
        let mut frame = MP3FrameInfo::new();
        let err =
            unsafe { crate::ffi::MP3GetNextFrameInfo(self.mp3dec, &mut frame, mp3buf.as_ptr()) };
        if err == 0 {
            // No error, return the frame info
            Ok(frame)
        } else {
            Err(err.into())
        }
    }

    /// Decode the next MP3 frame
    pub fn decode(&self, mp3buf: &[u8], newlen: i32, buf: &mut [i16]) -> Result<i32, DecodeErr> {
        let mut mp3baseptr: *const u8 = mp3buf.as_ptr();
        let mut newlen = newlen;
        let err = unsafe {
            crate::ffi::MP3Decode(
                self.mp3dec,
                &mut mp3baseptr,
                &mut newlen,
                buf.as_mut_ptr(),
                0,
            )
        };
        if err == 0 {
            // No error, return the new length of the source buffer
            Ok(newlen)
        } else {
            Err(err.into())
        }
    }

    /// Expose underlying C void pointer HMP3Decoder. For when you need to use ffi functions that aren't wrapped
    ///
    /// # Safety
    ///
    /// use only with ffi::* from within this library
    pub unsafe fn ptr(&mut self) -> *mut c_void {
        self.mp3dec
    }
}

impl Default for Mp3 {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Mp3 {
    fn drop(&mut self) {
        unsafe { crate::ffi::MP3FreeDecoder(self.mp3dec) }
    }
}
