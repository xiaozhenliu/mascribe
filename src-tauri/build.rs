fn main() {
    // Set rpath so the binary finds libonnxruntime in the Frameworks dir of the .app bundle
    println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path/../Frameworks");
    // Also look in the same directory as the binary (for dev/testing)
    println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path");

    tauri_build::build()
}
