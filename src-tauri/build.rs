fn main() {
    // Set rpath so the binary finds libonnxruntime in the Frameworks dir of the .app bundle
    println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path/../Frameworks");
    // Also look in the same directory as the binary (for dev/testing)
    println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path");

    // Link AVFoundation so AVCaptureDevice is available for microphone permission requests
    println!("cargo:rustc-link-lib=framework=AVFoundation");

    // Link Vision framework for native OCR (VNRecognizeTextRequest)
    println!("cargo:rustc-link-lib=framework=Vision");

    // Get Git branch and commit info
    let git_branch = std::process::Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let git_commit = std::process::Command::new("git")
        .args(["rev-parse", "--short=7", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    println!("cargo:rustc-env=GIT_BRANCH={}", git_branch);
    println!("cargo:rustc-env=GIT_COMMIT={}", git_commit);

    tauri_build::build()
}
