//! Windows screenshot capture using GDI (BitBlt).

use image::{ImageBuffer, Rgba};
use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::UI::WindowsAndMessaging::*;

/// Capture the main display.
pub fn capture_screen() -> Result<Vec<u8>, String> {
    unsafe {
        // Get the desktop window (entire screen)
        let hwnd = GetDesktopWindow();
        let hdc_screen = GetDC(hwnd);

        if hdc_screen.is_invalid() {
            return Err("Failed to get screen DC".to_string());
        }

        // Get screen dimensions
        let width = GetSystemMetrics(SM_CXSCREEN);
        let height = GetSystemMetrics(SM_CYSCREEN);

        println!("[screenshot] Capturing screen: {}x{}", width, height);

        // Create a compatible DC and bitmap
        let hdc_mem = CreateCompatibleDC(hdc_screen);
        if hdc_mem.is_invalid() {
            ReleaseDC(hwnd, hdc_screen);
            return Err("Failed to create compatible DC".to_string());
        }

        let hbitmap = CreateCompatibleBitmap(hdc_screen, width, height);
        if hbitmap.is_invalid() {
            DeleteDC(hdc_mem);
            ReleaseDC(hwnd, hdc_screen);
            return Err("Failed to create compatible bitmap".to_string());
        }

        let old_bitmap = SelectObject(hdc_mem, hbitmap);

        // Copy screen to bitmap
        let result = BitBlt(
            hdc_mem,
            0, 0,
            width, height,
            hdc_screen,
            0, 0,
            SRCCOPY,
        );

        if result.is_err() {
            SelectObject(hdc_mem, old_bitmap);
            DeleteObject(hbitmap);
            DeleteDC(hdc_mem);
            ReleaseDC(hwnd, hdc_screen);
            return Err("BitBlt failed".to_string());
        }

        // Get bitmap info
        let mut bitmap_info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: -height, // Negative for top-down DIB
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0 as u32,
                biSizeImage: 0,
                biXPelsPerMeter: 0,
                biYPelsPerMeter: 0,
                biClrUsed: 0,
                biClrImportant: 0,
            },
            bmiColors: [RGBQUAD::default(); 1],
        };

        // Allocate buffer for pixel data
        let row_size = ((width * 4 + 3) / 4) * 4; // Align to 4 bytes
        let buffer_size = row_size * height;
        let mut buffer: Vec<u8> = vec![0; buffer_size as usize];

        // Get pixel data
        let result = GetDIBits(
            hdc_mem,
            hbitmap,
            0,
            height as u32,
            Some(buffer.as_mut_ptr() as *mut _),
            &mut bitmap_info,
            DIB_RGB_COLORS,
        );

        if result == 0 {
            SelectObject(hdc_mem, old_bitmap);
            DeleteObject(hbitmap);
            DeleteDC(hdc_mem);
            ReleaseDC(hwnd, hdc_screen);
            return Err("GetDIBits failed".to_string());
        }

        // Convert BGRA to RGBA and create image buffer
        let mut img_buffer = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(width as u32, height as u32);

        for y in 0..height {
            for x in 0..width {
                let src_offset = (y * row_size + x * 4) as usize;
                let b = buffer[src_offset];
                let g = buffer[src_offset + 1];
                let r = buffer[src_offset + 2];
                let a = buffer[src_offset + 3];
                img_buffer.put_pixel(x as u32, y as u32, Rgba([r, g, b, a]));
            }
        }

        // Cleanup
        SelectObject(hdc_mem, old_bitmap);
        DeleteObject(hbitmap);
        DeleteDC(hdc_mem);
        ReleaseDC(hwnd, hdc_screen);

        // Encode to PNG
        let mut output = Vec::new();
        let encoder = image::codecs::png::PngEncoder::new(&mut output);
        img_buffer
            .write_with_encoder(encoder)
            .map_err(|e| format!("Failed to encode PNG: {}", e))?;

        println!("[screenshot] Captured {} bytes", output.len());
        Ok(output)
    }
}
