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

    build.include("picomp3lib/src");

    build
        // .define("LUTS_IN_RAM", None)
        // .define("CODE_IN_RAM", None)
        // .define("BYO_BUFFERS", None)
        .flag("-mlong-calls")
        .debug(false)
        .opt_level_str("s")
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
        .file("ffi/picomp3lib/src/trigtabs.c")
        .compile("picomp3lib");
}
