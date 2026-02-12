fn main() {
    // Set rpath so the binary finds libonnxruntime in the Frameworks dir of the .app bundle
    println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path/../Frameworks");
    // Also look in the same directory as the binary (for dev/testing)
    println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path");

    // Link AVFoundation so AVCaptureDevice is available for microphone permission requests
    println!("cargo:rustc-link-lib=framework=AVFoundation");

    // Link Vision framework for native OCR (VNRecognizeTextRequest)
    println!("cargo:rustc-link-lib=framework=Vision");

    tauri_build::build()
}
