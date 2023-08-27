# picolibmp3-rs
Rust bindings for picolibmp3 (WIP)

To update the bindings, install bindgen-cli and run `./ffi/bindgen.sh`

For testing this library, I recommend the test samples available at  
https://espressif-docs.readthedocs-hosted.com/projects/esp-adf/en/latest/design-guide/audio-samples.html  
To run the examples, you can fetch the short 2 channel example by running the following command in your system terminal:  
```system
wget https://dl.espressif.com/dl/audio/gs-16b-2c-44100hz.mp3
```