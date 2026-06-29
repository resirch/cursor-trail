use anyhow::{Context, Result};
use std::mem;
use std::ptr;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    CreateCompatibleDC, DeleteDC, DeleteObject, SelectObject, AC_SRC_ALPHA, AC_SRC_OVER,
    BITMAPINFO, BITMAPINFOHEADER, BI_RGB, BLENDFUNCTION, DIB_RGB_COLORS, HGDIOBJ,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetCursorInfo, GetSystemMetrics,
    PostQuitMessage, RegisterClassW, ShowWindow, TranslateMessage, UpdateLayeredWindow,
    CS_HREDRAW, CS_VREDRAW, CURSORINFO, CURSOR_SHOWING, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN,
    SM_YVIRTUALSCREEN, SW_SHOW, ULW_ALPHA, WINDOW_EX_STYLE, WINDOW_STYLE, WM_DESTROY,
    WNDCLASSW, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOPMOST, WS_EX_TRANSPARENT, WS_POPUP,
};

use crate::render::FrameBuffer;

const CLASS_NAME: &str = "CursorTrailOverlay";

pub struct OverlayWindow {
    hwnd: HWND,
    width: u32,
    height: u32,
    origin_x: i32,
    origin_y: i32,
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

impl OverlayWindow {
    pub fn create() -> Result<Self> {
        unsafe {
            let instance = GetModuleHandleW(None)?;
            let class_name = to_wide(CLASS_NAME);

            let wc = WNDCLASSW {
                lpfnWndProc: Some(window_proc),
                hInstance: instance.into(),
                lpszClassName: PCWSTR(class_name.as_ptr()),
                style: CS_HREDRAW | CS_VREDRAW,
                ..Default::default()
            };

            let _ = RegisterClassW(&wc);

            let origin_x = GetSystemMetrics(SM_XVIRTUALSCREEN);
            let origin_y = GetSystemMetrics(SM_YVIRTUALSCREEN);
            let width = GetSystemMetrics(SM_CXVIRTUALSCREEN) as u32;
            let height = GetSystemMetrics(SM_CYVIRTUALSCREEN) as u32;

            let ex_style =
                WS_EX_LAYERED | WS_EX_TOPMOST | WS_EX_NOACTIVATE | WS_EX_TRANSPARENT;

            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE(ex_style.0),
                PCWSTR(class_name.as_ptr()),
                PCWSTR(class_name.as_ptr()),
                WINDOW_STYLE(WS_POPUP.0),
                origin_x,
                origin_y,
                width as i32,
                height as i32,
                None,
                None,
                instance,
                None,
            )?;

            let _ = ShowWindow(hwnd, SW_SHOW);

            let overlay = Self {
                hwnd,
                width,
                height,
                origin_x,
                origin_y,
            };

            let frame = FrameBuffer::new(width, height);
            overlay
                .present(&frame)
                .context("Failed to present initial overlay frame")?;
            Ok(overlay)
        }
    }

    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn origin(&self) -> (i32, i32) {
        (self.origin_x, self.origin_y)
    }

    pub fn present(&self, frame: &FrameBuffer) -> Result<()> {
        unsafe {
            let bgra = frame.as_bgra_bytes();
            let hdc_screen = windows::Win32::Graphics::Gdi::GetDC(None);
            let hdc_mem = CreateCompatibleDC(hdc_screen);

            let bmi = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: frame.width as i32,
                    biHeight: -(frame.height as i32),
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: BI_RGB.0,
                    ..Default::default()
                },
                ..Default::default()
            };

            let mut bits: *mut core::ffi::c_void = ptr::null_mut();
            let hbmp = windows::Win32::Graphics::Gdi::CreateDIBSection(
                hdc_mem,
                &bmi,
                DIB_RGB_COLORS,
                &mut bits,
                None,
                0,
            )?;
            let old = SelectObject(hdc_mem, HGDIOBJ(hbmp.0));

            ptr::copy_nonoverlapping(bgra.as_ptr(), bits as *mut u8, bgra.len());

            let point_source = POINT { x: 0, y: 0 };
            let mut size = windows::Win32::Foundation::SIZE {
                cx: frame.width as i32,
                cy: frame.height as i32,
            };
            let point_dest = POINT {
                x: self.origin_x,
                y: self.origin_y,
            };

            let blend = BLENDFUNCTION {
                BlendOp: AC_SRC_OVER as u8,
                BlendFlags: 0,
                SourceConstantAlpha: 255,
                AlphaFormat: AC_SRC_ALPHA as u8,
            };

            UpdateLayeredWindow(
                self.hwnd,
                hdc_screen,
                Some(&point_dest),
                Some(&mut size),
                hdc_mem,
                Some(&point_source),
                windows::Win32::Foundation::COLORREF(0),
                Some(&blend),
                ULW_ALPHA,
            )?;

            SelectObject(hdc_mem, old);
            let _ = DeleteObject(HGDIOBJ(hbmp.0));
            let _ = DeleteDC(hdc_mem);
            windows::Win32::Graphics::Gdi::ReleaseDC(None, hdc_screen);
        }

        Ok(())
    }

    pub fn pump_messages(&self) -> bool {
        unsafe {
            use windows::Win32::UI::WindowsAndMessaging::{PeekMessageW, PM_REMOVE, WM_QUIT};

            let mut msg = windows::Win32::UI::WindowsAndMessaging::MSG::default();
            while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).into() {
                if msg.message == WM_QUIT {
                    return false;
                }
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
            true
        }
    }
}

fn to_wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

pub fn get_cursor_position() -> Result<(i32, i32)> {
    unsafe {
        let mut point = POINT::default();
        windows::Win32::UI::WindowsAndMessaging::GetCursorPos(&mut point)
            .context("Failed to read cursor position")?;
        Ok((point.x, point.y))
    }
}

pub fn is_cursor_visible() -> Result<bool> {
    unsafe {
        let mut info = CURSORINFO {
            cbSize: mem::size_of::<CURSORINFO>() as u32,
            ..Default::default()
        };
        GetCursorInfo(&mut info).context("Failed to read cursor visibility")?;
        Ok((info.flags.0 & CURSOR_SHOWING.0) != 0)
    }
}
