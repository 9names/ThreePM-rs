//! Rust bindings and high-level wrapper for `ThreePM` - which is a fixed-point MP3 decoder library written in C
//!
//! Use [easy_mode::EasyMode] for a simple MP3 decoding experience, or [mp3::Mp3] if you need low-level control over decoding.
//!
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
//!     // Set up our EasyMode decoder
//!     let mut easy = EasyMode::new();
//!     // Set up the source of our MP3 data
//!     let mp3_loader = &mut MP3.chunks(CHUNK_SZ);
//!     // Set up the buffer for the decoded audio data to be stored in
//!     let mut buf = [0i16; 2304];
//!
//!     // skip past the id3 tags and anything else up to the first mp3 sync tag
//!     while !easy.mp3_decode_ready() && easy.buffer_free() >= CHUNK_SZ {
//!         if let Some(mp3data) = mp3_loader.next() {
//!             easy.add_data(mp3data);
//!         } else {
//!             println!("Out of data!");
//!             break;
//!         }
//!     }
//!
//!     // Move our decode window up to the next sync word in the stream
//!     let syncd = easy.skip_to_next_sync_word();
//!     println!("Synced: {syncd}");
//!
//!     // We're past the header now, so we should be able to correctly decode an MP3 frame
//!     // Metadata is stored in every frame, so check that now:
//!     if let Ok(frame) = easy.mp3_info() {
//!         println!("First MP3 frame info: {:?}", frame);
//!     }
//!     loop {
//!         // if the buffer has space for another chunk of data from our source, load it
//!         if easy.buffer_free() >= CHUNK_SZ {
//!             if let Some(mp3data) = mp3_loader.next() {
//!                 easy.add_data(mp3data);
//!             }
//!         }
//!         // decode the next chunk of mp3
//!         match easy.decode(&mut buf) {
//!             Ok(_decoded_samples) => {
//!                 // Do something with decoded_samples (like play or store them)
//!             }
//!             Err(_e) => {
//!                 // Handle error by aborting, skipping a frame, adding more data, etc.
//!                 // This example will just exit because this is simpler
//!                 break;
//!             }
//!         }
//!     }
//! }
//!
//! ```
#![no_std]

// Allow the code generated by bindgen to break style rules
#[allow(dead_code)]
#[allow(non_camel_case_types)]
#[allow(non_upper_case_globals)]
#[allow(non_snake_case)]
/// Autogenerated (via bindgen) interfaces to the C ThreePM library
pub mod ffi;

mod contig_buffer;
pub mod easy_mode;
pub mod mp3;
