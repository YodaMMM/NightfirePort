// Link helper for Linux build environments without system ALSA dev files: the runtime lib lives
// in a local prefix (libasound.so.2 + the missing libasound.so dev symlink), and pkg-config may
// resolve alsa to the system dir regardless of PKG_CONFIG_PATH, so alsa-sys's build script
// emits a search path where only `libasound.so.2` exists and the linker fails with
// "unable to find library -lasound". When NF_ALSA_LIB_DIR is set, add that dir to the linker
// search path. No effect when unset (normal machines with installed ALSA dev packages).
fn main() {
    println!("cargo:rerun-if-env-changed=NF_ALSA_LIB_DIR");
    if let Ok(dir) = std::env::var("NF_ALSA_LIB_DIR") {
        if !dir.is_empty() {
            println!("cargo:rustc-link-search=native={}", dir);
        }
    }
}
