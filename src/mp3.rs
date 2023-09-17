/// High-level MP3 library wrapping functions.
///
/// This version uses a Rust-owned struct to own all data associated with the C FFI library.
use crate::ffi::{
    CriticalBandInfo, DequantInfo, FrameHeader, HuffmanInfo, IMDCTInfo, MP3DecInfo,
    ScaleFactorInfo, ScaleFactorInfoSub, ScaleFactorJS, SideInfo, SideInfoSub, SubbandInfo,
};
use core::ffi::c_void;

/// MP3 metadata (MPEG type, bitrate, etc)
///
/// MP3 does not store its metadata in a file header, instead a copy is in every frame of the MP3
/// This is handy for a streaming protocol, as you can fully recover a corrupted stream.
///
/// MP3FrameInfo is returned by [get_last_frame_info] and [get_next_frame_info]
pub use crate::ffi::_MP3FrameInfo as MP3FrameInfo;

impl MP3FrameInfo {
    pub fn new() -> MP3FrameInfo {
        MP3FrameInfo {
            bitrate: 0,
            nChans: 0,
            samprate: 0,
            bitsPerSample: 0,
            outputSamps: 0,
            layer: 0,
            version: 0,
            size: 0,
        }
    }
}

impl Default for MP3FrameInfo {
    fn default() -> Self {
        Self::new()
    }
}
/// Errors that occur when calling the decode function
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub enum DecodeErr {
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
}

impl From<i32> for DecodeErr {
    fn from(value: i32) -> Self {
        use DecodeErr::*;
        match value {
            0 => Okay,
            -1 => InDataUnderflow,
            -2 => MaindataUnderfow,
            -3 => FreeBitrateSync,
            -4 => OutOfMemory,
            -5 => NullPointer,
            -6 => InvalidFrameheader,
            -7 => InvalidSideinfo,
            -8 => InvalidScalefact,
            -9 => InvalidHuffcodes,
            -10 => InvalidDequantize,
            -11 => InvalidImdct,
            -12 => InvalidSubband,
            -9999 => Unknown,
            _ => InvalidError,
        }
    }
}

/// Low-level MP3 decode data
///
/// This struct contains all of the data structures required for the
/// low-level C library to operate as plain-old-data types (structs, arrays).
/// This allows it to be sized (which makes Rust happy).
///
/// Note: this struct is very large by embedded standards (~24KB).
/// Plan accordingly.
#[allow(dead_code)]
#[derive(Debug)]
pub struct Mp3 {
    mp3_dec_info: MP3DecInfo,
}

impl Mp3 {
    pub const fn new() -> Self {
        let mp3_dec_info = MP3DecInfo {
            mainBuf: [0; 1940],
            freeBitrateFlag: 0,
            freeBitrateSlots: 0,
            bitrate: 0,
            nChans: 0,
            samprate: 0,
            nGrans: 0,
            nGranSamps: 0,
            nSlots: 0,
            layer: 0,
            version: 0,
            size: 0,
            mainDataBegin: 0,
            mainDataBytes: 0,
            part23Length: [[0; 2]; 2],
            di: DequantInfo {
                workBuf: [0; 198],
                cbi: [CriticalBandInfo {
                    cbType: 0,
                    cbEndS: [0; 3],
                    cbEndSMax: 0,
                    cbEndL: 0,
                }; 2],
            },
            fh: FrameHeader {
                ver: 0,
                layer: 0,
                crc: 0,
                brIdx: 0,
                srIdx: 0,
                paddingBit: 0,
                privateBit: 0,
                sMode: 0,
                modeExt: 0,
                copyFlag: 0,
                origFlag: 0,
                emphasis: 0,
                CRCWord: 0,
            },
            si: SideInfo {
                mainDataBegin: 0,
                privateBits: 0,
                scfsi: [[0; 4]; 2],
                sis: [[SideInfoSub {
                    part23Length: 0,
                    nBigvals: 0,
                    globalGain: 0,
                    sfCompress: 0,
                    winSwitchFlag: 0,
                    blockType: 0,
                    mixedBlock: 0,
                    tableSelect: [0; 3],
                    subBlockGain: [0; 3],
                    region0Count: 0,
                    region1Count: 0,
                    preFlag: 0,
                    sfactScale: 0,
                    count1TableSelect: 0,
                }; 2]; 2],
            },
            sfi: ScaleFactorInfo {
                sfis: [[ScaleFactorInfoSub {
                    l: [0; 23],
                    s: [[0; 3]; 13],
                }; 2]; 2],
                sfjs: ScaleFactorJS {
                    intensityScale: 0,
                    slen: [0; 4],
                    nr: [0; 4],
                }, //
            },
            hi: HuffmanInfo {
                huffDecBuf: [[0; 576]; 2],
                nonZeroBound: [0; 2],
                gb: [0; 2],
            },
            mi: IMDCTInfo {
                outBuf: [[[0; 32]; 18]; 2],
                overBuf: [[0; 288]; 2],
                numPrevIMDCT: [0; 2],
                prevType: [0; 2],
                prevWinSwitch: [0; 2],
                gb: [0; 2],
            },
            sbi: SubbandInfo {
                vbuf: [0; 2176],
                vindex: 0,
            },
        };
        Self { mp3_dec_info }
    }

    /// Find the offset of the next sync word in the MP3 stream. Use this to find the next frame
    pub fn find_sync_word(mp3buf: &[u8]) -> i32 {
        unsafe { crate::ffi::MP3FindSyncWord(mp3buf.as_ptr(), mp3buf.len() as i32) }
    }

    /// Get info for the most recently decoded MP3 frame
    pub fn get_last_frame_info(&mut self) -> MP3FrameInfo {
        let mut frame = MP3FrameInfo::new();
        unsafe { crate::ffi::MP3GetLastFrameInfo(self.ptr(), &mut frame) };
        frame
    }

    /// Get info for the next MP3 frame
    pub fn get_next_frame_info(&mut self, mp3buf: &[u8]) -> Result<MP3FrameInfo, DecodeErr> {
        let mut frame = MP3FrameInfo::new();
        let err =
            unsafe { crate::ffi::MP3GetNextFrameInfo(self.ptr(), &mut frame, mp3buf.as_ptr()) };
        if err == 0 {
            // No error, return the frame info
            Ok(frame)
        } else {
            Err(err.into())
        }
    }

    /// Decode the next MP3 frame
    pub fn decode(
        &mut self,
        mp3buf: &[u8],
        newlen: i32,
        buf: &mut [i16],
    ) -> Result<i32, DecodeErr> {
        let mut newlen = newlen;
        let err = unsafe {
            crate::ffi::MP3Decode(
                self.ptr(),
                &mut mp3buf.as_ptr(),
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
        core::ptr::addr_of_mut!(self.mp3_dec_info) as *mut c_void
    }
}

impl Default for Mp3 {
    fn default() -> Self {
        Self::new()
    }
}
