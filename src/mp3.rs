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

#[derive(Debug)]
pub struct Id3v2Flags {
    /// indicates that unsynchronisation is applied on all frames
    pub unsynchronisation: bool,
    /// indicates that the header is followed by an extended header
    pub extended_header: bool,
    /// an ‘experimental indicator’. This flag SHALL always be set when the tag is in an experimental stage
    pub experimental: bool,
    /// indicates a footer is present at the very end of the tag
    pub footer_present: bool,
}

#[derive(PartialEq, Eq, Debug)]
pub enum Id3v2Version {
    /// ID3v2.0
    ID3v2_0,
    /// ID3v2.1
    ID3v2_1,
    /// ID3v2.2
    ID3v2_2,
    /// ID3v2.3
    ID3v2_3,
    /// ID3v2.4
    ID3v2_4,
    /// Major version isn't 2 and minor version isn't 0-4, unknown or invalid version
    Invalid,
}

#[derive(Debug)]
pub struct Id3v2 {
    /// Id3v2 version
    pub version: Id3v2Version,
    /// Id3v2 flags
    pub flags: Id3v2Flags,
    /// Length of the Id3v2 payload (excludes header length)
    pub size: usize,
}

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

    // from https://mutagen-specs.readthedocs.io/en/latest/id3/id3v2.4.0-structure.html
    // ID3 tag format is as follows
    // $49 44 33 yy yy xx zz zz zz zz
    // yy yy is the version, xx is flags, zz zz zz zz is the ID3v2 tag size.
    //
    /// Find and decode ID3v2 header info
    /// Returns the offset in the provided slice where it found the tag and the decoded tag header, or None
    pub fn find_id3v2(mp3buf: &[u8]) -> Option<(usize, Id3v2)> {
        let window = mp3buf.windows(10);
        for (offset, slice) in window.enumerate() {
            if let [b'I', b'D', b'3', major, minor, flags, s1, s2, s3, s4] = slice {
                let version = match (major, minor) {
                    (2, 2) => Id3v2Version::ID3v2_2,
                    (2, 3) => Id3v2Version::ID3v2_3,
                    (2, 4) => Id3v2Version::ID3v2_4,
                    (_, _) => Id3v2Version::Invalid,
                };
                let id3v2_flags = Id3v2Flags {
                    unsynchronisation: flags & 0b1000_0000 == 0b1000_0000,
                    extended_header: flags & 0b0100_0000 == 0b0100_0000,
                    experimental: flags & 0b0010_0000 == 0b0010_0000,
                    footer_present: flags & 0b0001_0000 == 0b0001_0000,
                };
                // Only the top 4 bits are valid flags, the bottom 4 were never used
                let valid_flags = flags & 0b0000_1111 != 0b0000_1111;
                // The ID3v2 tag size is stored as a 32 bit synchsafe integer, making a total of 28 effective bits (representing up to 256MB).
                // a syncsafe integer is a 7bit integer where the top bit is always zero.
                let valid_syncsafe = (s1 | s2 | s3 | s4) & 0b1000_0000 != 0b1000_0000;
                if version == Id3v2Version::Invalid && valid_syncsafe && valid_flags {
                    let (s1, s2, s3, s4) = (*s1 as usize, *s2 as usize, *s3 as usize, *s4 as usize);
                    let size = s4 | s3 << 7 | s2 << 14 | s1 << 21;
                    return Some((
                        offset,
                        Id3v2 {
                            version,
                            flags: id3v2_flags,
                            size,
                        },
                    ));
                }
            }
        }
        None
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
