default: run

clean-libs:
    @echo "Cleaning libs..."
    cd mobile/android && rm -r app/src/main/jniLibs/*/*.so || true

build-play: clean-libs
    @echo "Building for Google Play Store..."
    cargo ndk \
        --target=arm64-v8a \
        --target=armeabi-v7a \
        --target=x86 \
        --target=x86_64 \
        --platform=34 \
        -o mobile/android/app/src/main/jniLibs \
        build --release -p mobile
    cd mobile/android && ./gradlew bundleRelease
    cp mobile/android/stones/build/outputs/bundle/release/*.aab ./build/bundle/

build-android: clean-libs
    @echo "Building for android..."
    cd mobile/android && rm -r app/src/main/jniLibs
    cargo ndk \
        --target=arm64-v8a \
        --platform=34 \
        -o mobile/android/app/src/main/jniLibs \
        build --release -p mobile
    cd mobile/android && ./gradlew assembleRelease --warning-mode all
    cp mobile/android/stones/build/outputs/apk/release/*.apk ./build/apk/

build-android-emulator: clean-libs
    @echo "Building for android emulator..."
    cargo ndk \
        --target=x86_64 \
        --platform=34 \
        -o mobile/android/app/src/main/jniLibs \
        build --release -p mobile
    cd mobile/android && ./gradlew assembleRelease --warning-mode all
    cp mobile/android/stones/build/outputs/apk/release/*.apk ./build/apk/

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
