//! Demo to convert from mp3 to raw audio to validate decoding
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
use picomp3lib_rs::{
    ffi::*,
    mp3::{DecodeErr, Mp3},
};
use std::{fs::File, io::Write};

fn main() {
    println!("Adafruit-mp3 decoding start");
    type Mp3ptrT = *const u8;
    type Mp3ptrptrT = *mut Mp3ptrT;
    let mut mp3ptr: Mp3ptrT = MP3.as_ptr();
    let mp3ptrptr: Mp3ptrptrT = &mut mp3ptr;
    println!(
        "mp3ptr {:?}, mp3ptrptr {:?}, mp3ptrptr_pointee {:?}",
        mp3ptr,
        mp3ptrptr,
        unsafe { *mp3ptrptr }
    );
    let mut bytes_left = MP3.len() as i32;
    let mut mp3dec_struct = Mp3::new();
    let mp3dec = unsafe { mp3dec_struct.ptr() };
    let start = unsafe { picomp3lib_rs::ffi::MP3FindSyncWord(mp3ptr, bytes_left) };
    bytes_left -= start;
    println!("start of mp3 audio data: {}", start);
    // Update our MP3 pointer to skip past the id3 tags
    let mut mp3ptr: Mp3ptrT = MP3.as_ptr().wrapping_add(start.try_into().unwrap());
    let mp3ptrptr: Mp3ptrptrT = &mut mp3ptr;

    let mut frame: MP3FrameInfo = MP3FrameInfo {
        bitrate: 0,
        nChans: 0,
        samprate: 0,
        bitsPerSample: 0,
        outputSamps: 0,
        layer: 0,
        version: 0,
        size: 0,
    };

    let f = unsafe { MP3GetNextFrameInfo(mp3dec, &mut frame, mp3ptr) };
    println!("MP3GetNextFrameInfo response: {:?}", f);
    if f != 0 {
        panic!("MP3GetNextFrameInfo was not good");
    };

    println!("info: {:?}", frame);

    let decode_len = (frame.bitsPerSample >> 3) * frame.outputSamps;
    println!("length of each frame = {:?}", decode_len);
    let mut newlen = bytes_left as i32;
    println!("mp3 len: {:?}", newlen);
    // todo: work out what a sensible buffer length is
    // check decode_len for an idea. decode_len is in bytes
    const BUFF_LEN: usize = 4608 / 2;
    let mut buf = [0i16; BUFF_LEN];

    let mut file = File::create("audio_raw.bin").unwrap();
    while newlen > 0 {
        let decoded = unsafe { MP3Decode(mp3dec, mp3ptrptr, &mut newlen, buf.as_mut_ptr(), 0) };
        if decoded != 0 {
            let decoded = match decoded {
                0 => "Okay",
                -1 => "ERR_MP3_INDATA_UNDERFLOW",
                -2 => "ERR_MP3_MAINDATA_UNDERFLOW",
                -3 => "ERR_MP3_FREE_BITRATE_SYNC",
                -4 => "ERR_MP3_OUT_OF_MEMORY",
                -5 => "ERR_MP3_NULL_POINTER",
                -6 => "ERR_MP3_INVALID_FRAMEHEADER",
                -7 => "ERR_MP3_INVALID_SIDEINFO",
                -8 => "ERR_MP3_INVALID_SCALEFACT",
                -9 => "ERR_MP3_INVALID_HUFFCODES",
                -10 => "ERR_MP3_INVALID_DEQUANTIZE",
                -11 => "ERR_MP3_INVALID_IMDCT",
                -12 => "ERR_MP3_INVALID_SUBBAND",
                -9999 => "ERR_UNKNOWN",
                _ => "ERR_INVALID_ERROR",
            };
            println!("Decoded {}", decoded);
        }

        // get info about the last frame decoded
        unsafe { MP3GetLastFrameInfo(mp3dec, &mut frame) };
        if frame.outputSamps <= BUFF_LEN as i32 {
            file.write_all((&(buf[0..(frame.outputSamps) as usize])).as_byte_slice())
                .unwrap();
        } else {
            println!(
                "Decoded frame size {} exceeds buffer size. Assume frame is corrupted",
                frame.outputSamps
            );
        }
    }
    file.flush().unwrap();
    unsafe { MP3FreeDecoder(mp3dec) };
    let _ = mp3dec;
    println!("Should be free now");
}
