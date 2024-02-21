default: run

build-play:
    @echo "Building..."
    x build --store play --release -p mobile
    jarsigner target/x/release/android/mobile.aab android_release_key -keystore ~/.android/release.keystore

run: run-release

run-phone: 
    @echo "Running on phone..."
    @echo "Make sure you set gradle to false in order to run directly on phone"
    x run --release -p mobile --device $(x devices | grep arm64 | awk '{print $1}')

run-release:
    @echo "Running release..."
    cargo run --release

run-debug: 
    @echo "Running debug..."
    cargo run
