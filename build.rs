use std::env;

// defines that affect C library build:
// BYO_BUFFERS: the picomp3lib uses static buffers by default, if you don't provide alloc
// the rust library allocates these in a struct if using Mp3Transparent, so we don't want
// the static allocations

#[cfg(feature = "code-in-ram")]
fn code_in_ram() -> bool {
    true
}
#[cfg(not(feature = "code-in-ram"))]
fn code_in_ram() -> bool {
    false
}

#[cfg(feature = "use-static-buffers")]
fn byo_buffers() -> bool {
    false
}
#[cfg(not(feature = "use-static-buffers"))]
fn byo_buffers() -> bool {
    true
}

fn main() {
    println!("cargo:rerun-if-changed=ffi/picomp3lib/src/bitstream.c");
    println!("cargo:rerun-if-changed=ffi/picomp3lib/src/buffers.c");
    println!("cargo:rerun-if-changed=ffi/picomp3lib/src/dct32.c");
    println!("cargo:rerun-if-changed=ffi/picomp3lib/src/dequant.c");
    println!("cargo:rerun-if-changed=ffi/picomp3lib/src/dqchan.c");
    println!("cargo:rerun-if-changed=ffi/picomp3lib/src/huffman.c");
    println!("cargo:rerun-if-changed=ffi/picomp3lib/src/hufftabs.c");
    println!("cargo:rerun-if-changed=ffi/picomp3lib/src/imdct.c");
    println!("cargo:rerun-if-changed=ffi/picomp3lib/src/mp3dec.c");
    println!("cargo:rerun-if-changed=ffi/picomp3lib/src/mp3tabs.c");
    println!("cargo:rerun-if-changed=ffi/picomp3lib/src/polyphase.c");
    println!("cargo:rerun-if-changed=ffi/picomp3lib/src/scalfact.c");
    println!("cargo:rerun-if-changed=ffi/picomp3lib/src/stproc.c");
    println!("cargo:rerun-if-changed=ffi/picomp3lib/src/subband.c");
    println!("cargo:rerun-if-changed=ffi/picomp3lib/src/trigtabs.c");

    let mut build = cc::Build::new();
    let target = env::var("TARGET").unwrap();
    let target_is_cortex_m = target.starts_with("thumbv6m-")
        || target.starts_with("thumbv7m-")
        || target.starts_with("thumbv7em-")
        || target.starts_with("thumbv8m.base")
        || target.starts_with("thumbv8m.main");

    build.include("picomp3lib/src");
    build
        .file("ffi/picomp3lib/src/bitstream.c")
        .file("ffi/picomp3lib/src/buffers.c")
        .file("ffi/picomp3lib/src/dct32.c")
        .file("ffi/picomp3lib/src/dequant.c")
        .file("ffi/picomp3lib/src/dqchan.c")
        .file("ffi/picomp3lib/src/huffman.c")
        .file("ffi/picomp3lib/src/hufftabs.c")
        .file("ffi/picomp3lib/src/imdct.c")
        .file("ffi/picomp3lib/src/mp3dec.c")
        .file("ffi/picomp3lib/src/mp3tabs.c")
        .file("ffi/picomp3lib/src/polyphase.c")
        .file("ffi/picomp3lib/src/scalfact.c")
        .file("ffi/picomp3lib/src/stproc.c")
        .file("ffi/picomp3lib/src/subband.c")
        .file("ffi/picomp3lib/src/trigtabs.c");

    if code_in_ram() {
        build.define("CODE_IN_RAM", None);
        // putting code in .data when it has debug symbols makes the linker very angry, so disable debug
        build.debug(false);
        if target_is_cortex_m {
            // If we put functions in .data, they need -mlong-calls to be able to call memcpy and non-inlined compiler-builtins
            // but this isn't compatible with other targets.
            build.flag("-mlong-calls").opt_level_str("s");
        }
    }
    if byo_buffers() {
        build.define("BYO_BUFFERS", None);
    }
    build.compile("picomp3lib");
}
