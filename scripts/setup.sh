#!/bin/bash

install -Dvm 755 bin/esig /usr/local/bin/esig
install -Dvm 644 share/applications/esig.desktop /usr/local/share/applications/esig.desktop

sizes=(16 32 48 64 128 256 512)
for size in "${sizes[@]}"; do
    install -Dvm 644 share/icons/hicolor/${size}x${size}/apps/esig.png \
        /usr/local/share/icons/hicolor/${size}x${size}/apps/esig.png
done
