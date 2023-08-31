/// High-level MP3 library wrapping functions.
///
/// This version uses a Rust-owned struct to own all data associated with the C FFI library.
use crate::{
    ffi::{
        CriticalBandInfo, DequantInfo, FrameHeader, HuffmanInfo, IMDCTInfo, MP3DecInfo,
        ScaleFactorInfo, ScaleFactorInfoSub, ScaleFactorJS, SideInfo, SideInfoSub, SubbandInfo,
    },
    DecodeErr, MP3FrameInfo,
};
use core::ffi::c_void;

#[allow(dead_code)]
#[derive(Debug)]
pub struct Mp3 {
    mp3_dec_info: MP3DecInfo,
    mp3_info: Mp3Info,
}

impl Mp3 {
    /// WARNING:
    /// do not move this while in a function.
    /// todo: pin or something.
    pub const fn new() -> Self {
        let mp3_info = Mp3Info::new();
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
        Self {
            mp3_dec_info,
            mp3_info,
        }
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

#[derive(Debug, Copy, Clone)]
pub struct Mp3Info {
    pub frame_header: FrameHeader,
    pub side_info: SideInfo,
    pub scale_factor_info: ScaleFactorInfo,
    pub huffman_info: HuffmanInfo,
    pub dequant_info: DequantInfo,
    pub imdct_info: IMDCTInfo,
    pub subband_info: SubbandInfo,
}

impl Mp3Info {
    pub const fn new() -> Self {
        let fh = FrameHeader {
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
        };
        let sh = SideInfo {
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
        };
        let sfi = ScaleFactorInfo {
            sfis: [
                [
                    ScaleFactorInfoSub {
                        l: [0; 23],
                        s: [[0; 3]; 13],
                    },
                    ScaleFactorInfoSub {
                        l: [0; 23],
                        s: [[0; 3]; 13],
                    },
                ],
                [
                    ScaleFactorInfoSub {
                        l: [0; 23],
                        s: [[0; 3]; 13],
                    },
                    ScaleFactorInfoSub {
                        l: [0; 23],
                        s: [[0; 3]; 13],
                    },
                ],
            ],
            sfjs: ScaleFactorJS {
                intensityScale: 0,
                slen: [0; 4],
                nr: [0; 4],
            },
        };
        let hi = HuffmanInfo {
            huffDecBuf: [[0; 576]; 2],
            nonZeroBound: [0; 2],
            gb: [0; 2],
        };
        let di = DequantInfo {
            workBuf: [0; 198],
            cbi: [
                CriticalBandInfo {
                    cbType: 0,
                    cbEndS: [0; 3],
                    cbEndSMax: 0,
                    cbEndL: 0,
                },
                CriticalBandInfo {
                    cbType: 0,
                    cbEndS: [0; 3],
                    cbEndSMax: 0,
                    cbEndL: 0,
                },
            ],
        };
        let ii = IMDCTInfo {
            outBuf: [[[0; 32]; 18]; 2],
            overBuf: [[0; 288]; 2],
            numPrevIMDCT: [0; 2],
            prevType: [0; 2],
            prevWinSwitch: [0; 2],
            gb: [0; 2],
        };
        let sbi = SubbandInfo {
            vbuf: [0; 2176],
            vindex: 0,
        };

        Self {
            frame_header: fh,
            side_info: sh,
            scale_factor_info: sfi,
            huffman_info: hi,
            dequant_info: di,
            imdct_info: ii,
            subband_info: sbi,
        }
    }
}

impl Default for Mp3Info {
    fn default() -> Self {
        Self::new()
    }
}
