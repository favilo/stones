default: run

build-play:
    @echo "Building..."
    cargo ndk \
        --target=arm64-v8a \
        --target=armeabi-v7a \
        --target=x86 \
        --target=x86_64 \
        --platform=33 \
        -o mobile/android/app/src/main/jniLibs \
        build --release -p mobile
    cd mobile/android && ./gradlew bundleRelease
    apksigner sign --ks ~/.android/release.keystore --ks-key-alias android_release_key --ks-pass stdin target/play/release/app-release.aab
    # RUST_LOG=debug x build --store play --release -p mobile
    # jarsigner target/x/release/android/mobile.aab android_release_key -keystore ~/.android/release.keystore

build-android:
    @echo "Building for android emulator..."
    cargo ndk \
        --target=arm64-v8a \
        --target=x86_64 \
        --platform=33 \
        -o mobile/android/app/src/main/jniLibs \
        build --release -p mobile
    mobile/gradlew assembleDebug
    apksigner sign --ks ~/.android/release.keystore --ks-key-alias android_release_key --ks-pass stdin target/play/release/app-release.aab

run-android-emulator:
    @echo "Running on android emulator..."
    RUST_BACKTRACE=1 XBUILD_LOG=debug x run --release -p mobile --keystore="${HOME}/.android/release.keystore" --device $(x devices | grep emulator | awk '{print $1}') --format=apk

run: run-release

run-phone: 
    @echo "Running on phone..."
    @echo "Make sure you set gradle to false in manifest.yml in order to run directly on phone"
    x run --release -p mobile --device $(x devices | grep arm64 | awk '{print $1}') --format=apk

run-release:
    @echo "Running release..."
    cargo run --release

run-debug: 
    @echo "Running debug..."
    cargo run
