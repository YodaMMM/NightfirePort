# nf-android

The Android build of the port: a JNI bridge (`libdxandroid.so`, wgpu on Vulkan) and the app project in `android/`.
It is not part of the default workspace build; the desktop game does not depend on it.

Build (needs the Android NDK, `cargo-ndk` and Gradle):

    cargo ndk -P 28 -t arm64-v8a -o crates/nf-android/android/app/src/main/jniLibs build -p nf-android
    cd crates/nf-android/android && gradle assembleDebug

Game data: the app reads a folder of the game's files from its own storage (see `carve_android` in nf-formats),
never bundled with the app.

Debug telemetry is off unless a debuggable build is started with the `nf_telemetry` extra; it uses loopback
sockets only (127.0.0.1:9876 out, 9877 in; reach them with `adb reverse` / `adb forward`), read by
`tools/trace/android_telemetry.py`.
