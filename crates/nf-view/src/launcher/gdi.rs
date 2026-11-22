//! Windows: the launcher's frame shown with GDI (`SetDIBitsToDevice`), no GPU at all.
//!
//! The launcher is a CPU canvas; a GPU only ever copied it to the window. Some laptop drivers fault while a device
//! opens (the integrated GPU a low-power request picks), which closed the launcher before it drew anything. GDI
//! copies the same bytes 1:1 onto the client area, so the picture is the same pixel for pixel. Hand-written
//! declarations, like platform.rs: a few calls don't justify a dependency.

use std::ffi::c_void;

type Hwnd = *mut c_void;
type Hdc = *mut c_void;

#[repr(C)]
#[allow(non_snake_case)]
struct BitmapInfoHeader {
    biSize: u32,
    biWidth: i32,
    biHeight: i32,
    biPlanes: u16,
    biBitCount: u16,
    biCompression: u32,
    biSizeImage: u32,
    biXPelsPerMeter: i32,
    biYPelsPerMeter: i32,
    biClrUsed: u32,
    biClrImportant: u32,
}

/// BITMAPINFO: the header and one (unused for 32-bit BI_RGB) colour entry.
#[repr(C)]
struct BitmapInfo {
    header: BitmapInfoHeader,
    colors: [u32; 1],
}

#[link(name = "user32")]
extern "system" {
    fn GetDC(hwnd: Hwnd) -> Hdc;
    fn ReleaseDC(hwnd: Hwnd, hdc: Hdc) -> i32;
    fn PrintWindow(hwnd: Hwnd, hdc: Hdc, flags: u32) -> i32;
}
#[link(name = "gdi32")]
extern "system" {
    fn SetDIBitsToDevice(hdc: Hdc, xd: i32, yd: i32, w: u32, h: u32, xs: i32, ys: i32, start: u32, lines: u32, bits: *const c_void, bmi: *const BitmapInfo, usage: u32) -> i32;
    fn CreateCompatibleDC(hdc: Hdc) -> Hdc;
    fn CreateCompatibleBitmap(hdc: Hdc, w: i32, h: i32) -> *mut c_void;
    fn SelectObject(hdc: Hdc, obj: *mut c_void) -> *mut c_void;
    fn DeleteObject(obj: *mut c_void) -> i32;
    fn DeleteDC(hdc: Hdc) -> i32;
    fn GetDIBits(hdc: Hdc, bmp: *mut c_void, start: u32, lines: u32, bits: *mut c_void, bmi: *mut BitmapInfo, usage: u32) -> i32;
}

const DIB_RGB_COLORS: u32 = 0;
/// PrintWindow: the client area only, and the window's composed content (also a GPU swap chain's).
const PW_CLIENTONLY_RENDERFULLCONTENT: u32 = 0x1 | 0x2;

/// A top-down 32-bit BGRX bitmap header of `w`×`h`.
fn info(w: u32, h: u32) -> BitmapInfo {
    BitmapInfo {
        header: BitmapInfoHeader { biSize: std::mem::size_of::<BitmapInfoHeader>() as u32, biWidth: w as i32, biHeight: -(h as i32), biPlanes: 1, biBitCount: 32, biCompression: 0, biSizeImage: 0, biXPelsPerMeter: 0, biYPelsPerMeter: 0, biClrUsed: 0, biClrImportant: 0 },
        colors: [0],
    }
}

/// RGBA → GDI's BGRX (and back: the swap is its own inverse; alpha is set opaque).
pub fn swap_rb(src: &[u8], out: &mut Vec<u8>) {
    out.clear();
    out.reserve(src.len());
    for p in src.chunks_exact(4) { out.extend_from_slice(&[p[2], p[1], p[0], 255]); }
}

pub struct Gdi {
    hwnd: Hwnd,
    bgra: Vec<u8>,
}

impl Gdi {
    pub fn new(hwnd: Hwnd) -> Gdi { Gdi { hwnd, bgra: Vec::new() } }

    /// Copies the RGBA frame 1:1 to the client area's top-left.
    pub fn present(&mut self, rgba: &[u8], w: u32, h: u32) {
        if w == 0 || h == 0 || rgba.len() < (w * h * 4) as usize { return; }
        swap_rb(rgba, &mut self.bgra);
        let bmi = info(w, h);
        // SAFETY: hwnd is the live launcher window; `bgra` holds w*h BGRX pixels matching `bmi` for the call.
        unsafe {
            let hdc = GetDC(self.hwnd);
            if hdc.is_null() { return; }
            SetDIBitsToDevice(hdc, 0, 0, w, h, 0, 0, 0, h, self.bgra.as_ptr() as *const c_void, &bmi, DIB_RGB_COLORS);
            ReleaseDC(self.hwnd, hdc);
        }
    }
}

/// The window's client area as RGBA, read back through `PrintWindow` (works for a GDI or a GPU-presented window).
/// The `NF_LAUNCHER_SHOT` check uses it to compare what each path really put on screen.
pub fn capture_client(hwnd: Hwnd, w: u32, h: u32) -> Option<Vec<u8>> {
    if w == 0 || h == 0 { return None; }
    let mut bmi = info(w, h);
    let mut bgra = vec![0u8; (w * h * 4) as usize];
    // SAFETY: plain GDI object lifetimes, each released below; `bgra` is w*h*4 bytes, as `bmi` describes.
    let ok = unsafe {
        let wdc = GetDC(hwnd);
        if wdc.is_null() { return None; }
        let mdc = CreateCompatibleDC(wdc);
        let bmp = CreateCompatibleBitmap(wdc, w as i32, h as i32);
        let old = SelectObject(mdc, bmp);
        let printed = PrintWindow(hwnd, mdc, PW_CLIENTONLY_RENDERFULLCONTENT) != 0;
        SelectObject(mdc, old);
        let lines = GetDIBits(mdc, bmp, 0, h, bgra.as_mut_ptr() as *mut c_void, &mut bmi, DIB_RGB_COLORS);
        DeleteObject(bmp);
        DeleteDC(mdc);
        ReleaseDC(hwnd, wdc);
        printed && lines == h as i32
    };
    if !ok { return None; }
    let mut rgba = Vec::new();
    swap_rb(&bgra, &mut rgba);
    Some(rgba)
}

#[cfg(test)]
mod tests {
    #[test]
    fn bitmap_header_is_the_win32_size() {
        // sizeof(BITMAPINFOHEADER) (wingdi.h); GDI rejects a wrong biSize.
        assert_eq!(std::mem::size_of::<super::BitmapInfoHeader>(), 40);
        assert_eq!(std::mem::size_of::<super::BitmapInfo>(), 44);
    }

    #[test]
    fn swap_rb_round_trips() {
        let rgba = [10u8, 20, 30, 255, 1, 2, 3, 255];
        let mut bgra = Vec::new();
        super::swap_rb(&rgba, &mut bgra);
        assert_eq!(bgra, [30, 20, 10, 255, 3, 2, 1, 255]);
        let mut back = Vec::new();
        super::swap_rb(&bgra, &mut back);
        assert_eq!(back, rgba);
        assert_ne!(bgra.as_slice(), rgba.as_slice(), "the channels really move");
    }
}
