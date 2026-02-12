//! macOS native OCR using Vision framework (VNRecognizeTextRequest).
//!
//! Uses the Neural Engine on Apple Silicon for fast text recognition (<0.5s).
//! Calls Vision framework via raw objc2 msg_send! since no typed Rust bindings exist.

use objc2::runtime::AnyObject;
use objc2::{class, msg_send};
use objc2_foundation::NSString;
use std::ffi::CStr;
use std::ptr;

/// Recognize text in a PNG image using macOS Vision framework.
///
/// Creates a VNRecognizeTextRequest with accurate recognition level and
/// multi-language support (Chinese, English, Japanese, Korean).
pub fn recognize_text(png_bytes: &[u8]) -> Result<String, String> {
    unsafe {
        // 1. Create NSData from PNG bytes
        let ns_data: *mut AnyObject = msg_send![
            class!(NSData),
            dataWithBytes: png_bytes.as_ptr() as *const std::ffi::c_void,
            length: png_bytes.len()
        ];
        if ns_data.is_null() {
            return Err("Failed to create NSData from PNG bytes".into());
        }

        // 2. Create VNImageRequestHandler with the image data
        let options: *mut AnyObject = msg_send![class!(NSDictionary), dictionary];
        let handler: *mut AnyObject = msg_send![class!(VNImageRequestHandler), alloc];
        let handler: *mut AnyObject = msg_send![handler, initWithData: ns_data, options: options];
        if handler.is_null() {
            return Err("Failed to create VNImageRequestHandler".into());
        }

        // 3. Create and configure VNRecognizeTextRequest
        let request: *mut AnyObject = msg_send![class!(VNRecognizeTextRequest), alloc];
        let request: *mut AnyObject = msg_send![request, init];
        if request.is_null() {
            return Err("Failed to create VNRecognizeTextRequest".into());
        }

        // Recognition level: Accurate = 0, Fast = 1
        let _: () = msg_send![request, setRecognitionLevel: 0i64];

        // Set recognition languages
        let lang_strs = ["zh-Hans", "zh-Hant", "en-US", "ja-JP", "ko-KR"];
        let lang_array: *mut AnyObject =
            msg_send![class!(NSMutableArray), arrayWithCapacity: lang_strs.len()];
        for lang in &lang_strs {
            let ns_str = NSString::from_str(lang);
            let _: () = msg_send![lang_array, addObject: &*ns_str];
        }
        let _: () = msg_send![request, setRecognitionLanguages: lang_array];

        // 4. Perform the request
        let req_array: *mut AnyObject = msg_send![
            class!(NSArray),
            arrayWithObject: request
        ];
        let mut error: *mut AnyObject = ptr::null_mut();
        let success: bool = msg_send![
            handler,
            performRequests: req_array,
            error: &mut error
        ];

        if !success {
            if !error.is_null() {
                let desc: *mut AnyObject = msg_send![error, localizedDescription];
                let utf8: *const i8 = msg_send![desc, UTF8String];
                if !utf8.is_null() {
                    let err = CStr::from_ptr(utf8).to_string_lossy();
                    return Err(format!("Vision OCR failed: {}", err));
                }
            }
            return Err("Vision OCR failed with unknown error".into());
        }

        // 5. Extract recognized text from results
        let results: *mut AnyObject = msg_send![request, results];
        if results.is_null() {
            return Ok(String::new());
        }

        let count: usize = msg_send![results, count];
        let mut lines = Vec::new();

        for i in 0..count {
            let observation: *mut AnyObject = msg_send![results, objectAtIndex: i];
            // topCandidates: returns top N candidates for this observation
            let candidates: *mut AnyObject = msg_send![observation, topCandidates: 1usize];
            let cand_count: usize = msg_send![candidates, count];
            if cand_count > 0 {
                let candidate: *mut AnyObject = msg_send![candidates, objectAtIndex: 0usize];
                let string: *mut AnyObject = msg_send![candidate, string];
                let utf8: *const i8 = msg_send![string, UTF8String];
                if !utf8.is_null() {
                    let line = CStr::from_ptr(utf8).to_string_lossy().into_owned();
                    if !line.is_empty() {
                        lines.push(line);
                    }
                }
            }
        }

        Ok(lines.join("\n"))
    }
}
