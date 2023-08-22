#!/bin/sh

bindgen ffi/bindgen.h \
        --use-core --ctypes-prefix core::ffi \
        --output src/ffi.rs -- -Iffi/picomp3lib/src
