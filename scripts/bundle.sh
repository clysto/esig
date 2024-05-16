#!/bin/bash

if [ "$(uname)" == "Darwin" ]; then
    rm -rf target/release/bundle
    mkdir -p target/release/bundle/ESig.app/Contents/MacOS
    mkdir -p target/release/bundle/ESig.app/Contents/Resources
    cp scripts/Info.plist target/release/bundle/ESig.app/Contents
    cp target/release/esig target/release/bundle/ESig.app/Contents/MacOS
    cp assets/esig.icns target/release/bundle/ESig.app/Contents/Resources/ESig.icns
else
    rm -rf target/release/bundle
    mkdir -p target/release/bundle
    mkdir -p target/release/bundle/bin
    mkdir -p target/release/bundle/share/applications

    cp target/release/esig target/release/bundle/bin
    cp scripts/esig.desktop target/release/bundle/share/applications

    sizes=(16 32 48 64 128 256 512)
    for size in "${sizes[@]}"; do
        mkdir -p target/release/bundle/share/icons/hicolor/${size}x${size}/apps
        cp assets/icons/icon_${size}x${size}.png target/release/bundle/share/icons/hicolor/${size}x${size}/apps/esig.png
    done

    cp scripts/setup.sh target/release/bundle
    makeself target/release/bundle target/release/ESig.run ESig ./setup.sh
fi
