# ThreePM-rs
Rust bindings for ThreePM - a fixed-point MP3 decoding library written in C

It supports decoding all MP3 CBR files. VBR is not currently supported.

To update the bindings, install bindgen-cli and run `./ffi/bindgen.sh`

For testing this library, I recommend the test samples available at  
https://espressif-docs.readthedocs-hosted.com/projects/esp-adf/en/latest/design-guide/audio-samples.html  
To run the examples, you can fetch the short 2 channel example by running the following command in your system terminal:  
```system
wget https://dl.espressif.com/dl/audio/gs-16b-2c-44100hz.mp3
```

This crate will compile the C library as part of the build process - this means you need to tell Rust about your C compiler!
With a cortex-m target, it is sufficient to have an `arm-none-eabi-` toolchain on your path as this is the target default.
With a riscv target, you also need to have an environment variable exposing the C compiler name.
For https://github.com/riscv-collab/riscv-gnu-toolchain/releases or https://www.embecosm.com/resources/tool-chain-downloads/
```system
CC=riscv32-unknown-elf-gcc cargo run --release
```
or for the xPack riscv gcc toolchain https://xpack.github.io/dev-tools/riscv-none-elf-gcc/releases/
```system
CC=riscv-none-elf-gcc cargo run --release
```