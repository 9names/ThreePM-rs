# ThreePM-rs
Rust bindings for [ThreePM](ffi/ThreePM/README.md) - a fixed-point MP3 decoding library written in C.

It supports decoding all MP3 CBR files. VBR is not currently supported.

### Build

This crate will compile ThreePM as part of the build process - this means you need to tell Rust about your C compiler!
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

### Develop

If you update or add to the interface of ThreePM you will need to update the Rust bindings.
To do so, install bindgen-cli and run `./ffi/bindgen.sh` from the root of this project.

### Test

For testing this library, I recommend the test samples available at  
https://espressif-docs.readthedocs-hosted.com/projects/esp-adf/en/latest/design-guide/audio-samples.html  
The examples in the `examples` path of project are already configured to run against a short 2 channel example from the espressif audio samples page.
You can grab this sample by running the following command from the root of this project if you have `wget` installed:  
```system
wget https://dl.espressif.com/dl/audio/gs-16b-2c-44100hz.mp3
```
