#[cfg(windows)]
mod screensaver {
    use std::ffi::c_void;
    use std::io;
    use std::mem;
    use std::ptr;
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicBool, AtomicPtr, AtomicUsize, Ordering};

    type Bool = i32;
    type Dword = u32;
    type Handle = *mut c_void;
    type Hinstance = Handle;
    type Hcursor = Handle;
    type Hicon = Handle;
    type Hbrush = Handle;
    type Hdc = Handle;
    type Hwnd = Handle;
    type Lparam = isize;
    type Lresult = isize;
    type Uint = u32;
    type Wparam = usize;

    const WS_EX_TOPMOST: Dword = 0x0000_0008;
    const WS_EX_TOOLWINDOW: Dword = 0x0000_0080;
    const WS_EX_TRANSPARENT: Dword = 0x0000_0020;
    const WS_EX_LAYERED: Dword = 0x0008_0000;
    const WS_EX_NOACTIVATE: Dword = 0x0800_0000;
    const WS_POPUP: Dword = 0x8000_0000;
    const SW_SHOW: i32 = 5;
    const SW_SHOWNOACTIVATE: i32 = 4;

    const WM_DESTROY: Uint = 0x0002;
    const WM_CLOSE: Uint = 0x0010;
    const WM_SETCURSOR: Uint = 0x0020;
    const WM_PAINT: Uint = 0x000F;
    const WM_ERASEBKGND: Uint = 0x0014;
    const WM_KEYDOWN: Uint = 0x0100;
    const WM_SYSKEYDOWN: Uint = 0x0104;
    const WM_TIMER: Uint = 0x0113;
    const WM_APP: Uint = 0x8000;
    const WM_MOUSE_WAKE: Uint = WM_APP + 1;
    const WM_MOUSE_WAKE_COMPLETE: Uint = WM_APP + 2;
    const WH_MOUSE_LL: i32 = 14;
    const BLACK_BRUSH: i32 = 4;
    const LWA_COLORKEY: Dword = 0x0000_0001;
    const SWP_NOSIZE: Uint = 0x0001;
    const SWP_NOZORDER: Uint = 0x0004;
    const SWP_NOREDRAW: Uint = 0x0008;
    const SWP_NOACTIVATE: Uint = 0x0010;
    const CLOCK_TIMER_ID: Wparam = 1;
    const ANIMATION_TIMER_ID: Wparam = 2;
    const BCRYPT_USE_SYSTEM_PREFERRED_RNG: Dword = 0x0000_0002;
    const CLOCK_REFRESH_INTERVAL: Uint = 5000;
    const TIME_NOSECONDS: Dword = 0x0000_0002;
    const GGO_METRICS: Uint = 0;
    const CLOCK_FONT_SIZE: i32 = 120;
    const DATE_FONT_SIZE: i32 = 20;
    const TEXT_GAP: i32 = 15;
    const CLOCK_WIDTH_SAMPLE: &str = "00:00";
    const TEXT_COLOR: Dword = 0x00CC_CC_CC;
    const SM_XVIRTUALSCREEN: i32 = 76;
    const SM_YVIRTUALSCREEN: i32 = 77;
    const SM_CXVIRTUALSCREEN: i32 = 78;
    const SM_CYVIRTUALSCREEN: i32 = 79;

    const START_VELOCITIES: [(i32, i32); 8] = [
        (4, 3),
        (3, 4),
        (-3, 4),
        (-4, 3),
        (-4, -3),
        (-3, -4),
        (3, -4),
        (4, -3),
    ];

    static WINDOW_HANDLE: AtomicPtr<c_void> = AtomicPtr::new(ptr::null_mut());
    static CLOCK_WINDOW: AtomicPtr<c_void> = AtomicPtr::new(ptr::null_mut());
    static MOUSE_BUTTONS_HELD: AtomicUsize = AtomicUsize::new(0);
    static MOUSE_WAKE_PENDING: AtomicBool = AtomicBool::new(false);
    static CLOCK_STATE: Mutex<Option<ClockState>> = Mutex::new(None);

    struct ClockState {
        time_text: String,
        date_text: String,
        x: i32,
        y: i32,
        velocity_x: i32,
        velocity_y: i32,
        width: i32,
        height: i32,
        screen_x: i32,
        screen_y: i32,
        screen_width: i32,
        screen_height: i32,
    }

    #[repr(C)]
    struct Rect {
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct Size {
        cx: i32,
        cy: i32,
    }

    #[derive(Clone, Copy)]
    struct InkBounds {
        top: i32,
        bottom: i32,
    }

    struct ClockLayout {
        size: Size,
        clock_size: Size,
        date_size: Size,
        clock_frame_width: i32,
        clock_y: i32,
        date_y: i32,
    }

    #[repr(C)]
    struct TextMetric {
        height: i32,
        ascent: i32,
        descent: i32,
        internal_leading: i32,
        external_leading: i32,
        average_char_width: i32,
        max_char_width: i32,
        weight: i32,
        overhang: i32,
        digitized_aspect_x: i32,
        digitized_aspect_y: i32,
        first_char: u16,
        last_char: u16,
        default_char: u16,
        break_char: u16,
        italic: u8,
        underlined: u8,
        struck_out: u8,
        pitch_and_family: u8,
        char_set: u8,
    }

    #[repr(C)]
    struct GlyphMetrics {
        black_box_x: u32,
        black_box_y: u32,
        glyph_origin: Point,
        cell_inc_x: i16,
        cell_inc_y: i16,
    }

    #[repr(C)]
    struct Fixed {
        fract: u16,
        value: i16,
    }

    #[repr(C)]
    struct Mat2 {
        m11: Fixed,
        m12: Fixed,
        m21: Fixed,
        m22: Fixed,
    }

    #[repr(C)]
    struct PaintStruct {
        dc: Handle,
        erase: Bool,
        paint: Rect,
        restore: Bool,
        inc_update: Bool,
        reserved: [u8; 32],
    }

    #[repr(C)]
    struct Point {
        x: i32,
        y: i32,
    }

    #[repr(C)]
    struct MouseHookData {
        point: Point,
        mouse_data: Dword,
        flags: Dword,
        time: Dword,
        extra_info: usize,
    }

    #[repr(C)]
    struct Message {
        hwnd: Hwnd,
        message: Uint,
        w_param: Wparam,
        l_param: Lparam,
        time: Dword,
        point: Point,
        private: Dword,
    }

    type WindowProc = Option<unsafe extern "system" fn(Hwnd, Uint, Wparam, Lparam) -> Lresult>;
    type HookProc = Option<unsafe extern "system" fn(i32, Wparam, Lparam) -> Lresult>;

    #[repr(C)]
    struct WindowClass {
        style: Uint,
        window_proc: WindowProc,
        class_extra: i32,
        window_extra: i32,
        instance: Hinstance,
        icon: Hicon,
        cursor: Hcursor,
        background: Hbrush,
        menu_name: *const u16,
        class_name: *const u16,
    }

    #[link(name = "user32")]
    unsafe extern "system" {
        fn BeginPaint(hwnd: Hwnd, paint: *mut PaintStruct) -> Hdc;
        fn CreateWindowExW(
            ex_style: Dword,
            class_name: *const u16,
            window_name: *const u16,
            style: Dword,
            x: i32,
            y: i32,
            width: i32,
            height: i32,
            parent: Hwnd,
            menu: Handle,
            instance: Hinstance,
            parameter: *mut c_void,
        ) -> Hwnd;
        fn DefWindowProcW(hwnd: Hwnd, message: Uint, w_param: Wparam, l_param: Lparam) -> Lresult;
        fn DispatchMessageW(message: *const Message) -> Lresult;
        fn DestroyWindow(hwnd: Hwnd) -> Bool;
        fn EndPaint(hwnd: Hwnd, paint: *const PaintStruct) -> Bool;
        fn FillRect(dc: Hdc, rect: *const Rect, brush: Hbrush) -> i32;
        fn GetDC(hwnd: Hwnd) -> Hdc;
        fn GetMessageW(message: *mut Message, hwnd: Hwnd, min: Uint, max: Uint) -> Bool;
        fn GetSystemMetrics(index: i32) -> i32;
        fn InvalidateRect(hwnd: Hwnd, rect: *const Rect, erase: Bool) -> Bool;
        fn KillTimer(hwnd: Hwnd, timer_id: Wparam) -> Bool;
        fn LoadCursorW(instance: Hinstance, cursor_name: *const u16) -> Hcursor;
        fn PostQuitMessage(exit_code: i32);
        fn PostMessageW(hwnd: Hwnd, message: Uint, w_param: Wparam, l_param: Lparam) -> Bool;
        fn RegisterClassW(window_class: *const WindowClass) -> u16;
        fn SetCursor(cursor: Hcursor) -> Hcursor;
        fn SetForegroundWindow(hwnd: Hwnd) -> Bool;
        fn SetLayeredWindowAttributes(
            hwnd: Hwnd,
            color_key: Dword,
            alpha: u8,
            flags: Dword,
        ) -> Bool;
        fn SetTimer(
            hwnd: Hwnd,
            timer_id: Wparam,
            interval: Uint,
            timer_proc: *mut c_void,
        ) -> Wparam;
        fn SetWindowPos(
            hwnd: Hwnd,
            insert_after: Hwnd,
            x: i32,
            y: i32,
            width: i32,
            height: i32,
            flags: Uint,
        ) -> Bool;
        fn ShowWindow(hwnd: Hwnd, command: i32) -> Bool;
        fn TranslateMessage(message: *const Message) -> Bool;
        fn UnregisterClassW(class_name: *const u16, instance: Hinstance) -> Bool;
        fn UpdateWindow(hwnd: Hwnd) -> Bool;
        fn ReleaseDC(hwnd: Hwnd, dc: Hdc) -> i32;
        fn SetWindowsHookExW(
            hook_id: i32,
            hook_proc: HookProc,
            instance: Hinstance,
            thread_id: Dword,
        ) -> Handle;
        fn UnhookWindowsHookEx(hook: Handle) -> Bool;
        fn CallNextHookEx(hook: Handle, code: i32, w_param: Wparam, l_param: Lparam) -> Lresult;
    }

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetDateFormatEx(
            locale_name: *const u16,
            flags: Dword,
            date_time: *const c_void,
            format: *const u16,
            output: *mut u16,
            output_size: i32,
            calendar: *const u16,
        ) -> i32;
        fn GetModuleHandleW(module_name: *const u16) -> Hinstance;
        fn GetTimeFormatEx(
            locale_name: *const u16,
            flags: Dword,
            date_time: *const c_void,
            format: *const u16,
            output: *mut u16,
            output_size: i32,
        ) -> i32;
    }

    #[link(name = "bcrypt")]
    unsafe extern "system" {
        fn BCryptGenRandom(
            algorithm: Handle,
            buffer: *mut u8,
            buffer_length: Dword,
            flags: Dword,
        ) -> i32;
    }

    fn velocity_for_direction(direction: u8) -> (i32, i32) {
        START_VELOCITIES[direction as usize & (START_VELOCITIES.len() - 1)]
    }

    fn random_starting_velocity() -> io::Result<(i32, i32)> {
        let mut random_byte = 0u8;
        let status = unsafe {
            BCryptGenRandom(
                ptr::null_mut(),
                &mut random_byte,
                1,
                BCRYPT_USE_SYSTEM_PREFERRED_RNG,
            )
        };
        if status < 0 {
            return Err(io::Error::other(format!(
                "Windows system random generator failed (NTSTATUS 0x{:08X}).",
                status as u32
            )));
        }
        Ok(velocity_for_direction(random_byte))
    }

    fn create_font(pixel_size: i32) -> io::Result<Handle> {
        let face_name: Vec<u16> = "Segoe UI\0".encode_utf16().collect();
        let font = unsafe {
            CreateFontW(
                -pixel_size,
                0,
                0,
                0,
                400,
                0,
                0,
                0,
                1,
                0,
                0,
                4,
                0,
                face_name.as_ptr(),
            )
        };
        if font.is_null() {
            Err(io::Error::last_os_error())
        } else {
            Ok(font)
        }
    }

    fn measure_text(dc: Hdc, text: &str) -> io::Result<Size> {
        let text: Vec<u16> = text.encode_utf16().collect();
        let mut size = Size { cx: 0, cy: 0 };
        if unsafe { GetTextExtentPoint32W(dc, text.as_ptr(), text.len() as i32, &mut size) } == 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(size)
        }
    }

    fn font_metrics(dc: Hdc) -> io::Result<TextMetric> {
        let mut metrics: TextMetric = unsafe { mem::zeroed() };
        if unsafe { GetTextMetricsW(dc, &mut metrics) } == 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(metrics)
        }
    }

    fn text_ink_bounds(dc: Hdc, text: &str, metrics: &TextMetric) -> InkBounds {
        let transform = Mat2 {
            m11: Fixed { fract: 0, value: 1 },
            m12: Fixed { fract: 0, value: 0 },
            m21: Fixed { fract: 0, value: 0 },
            m22: Fixed { fract: 0, value: 1 },
        };
        let mut bounds: Option<InkBounds> = None;
        for character in text.encode_utf16() {
            let mut glyph = unsafe { mem::zeroed::<GlyphMetrics>() };
            let result = unsafe {
                GetGlyphOutlineW(
                    dc,
                    character as u32,
                    GGO_METRICS,
                    &mut glyph,
                    0,
                    ptr::null_mut(),
                    &transform,
                )
            };
            if result == u32::MAX || glyph.black_box_y == 0 {
                continue;
            }
            let top = metrics.ascent - glyph.glyph_origin.y;
            let glyph_bounds = InkBounds {
                top,
                bottom: top + glyph.black_box_y as i32,
            };
            bounds = Some(match bounds {
                Some(bounds) => InkBounds {
                    top: bounds.top.min(glyph_bounds.top),
                    bottom: bounds.bottom.max(glyph_bounds.bottom),
                },
                None => glyph_bounds,
            });
        }
        bounds.unwrap_or(InkBounds {
            top: 0,
            bottom: metrics.height,
        })
    }

    fn merge_ink_bounds(first: InkBounds, second: InkBounds) -> InkBounds {
        InkBounds {
            top: first.top.min(second.top),
            bottom: first.bottom.max(second.bottom),
        }
    }

    fn line_positions(clock_ink: InkBounds, date_ink: InkBounds) -> (i32, i32, i32) {
        let clock_y = -clock_ink.top;
        let date_y = clock_y + clock_ink.bottom + TEXT_GAP - date_ink.top;
        let height = (clock_y + clock_ink.bottom).max(date_y + date_ink.bottom);
        (clock_y, date_y, height)
    }

    fn clock_outer_width(actual_width: i32, reference_width: i32) -> i32 {
        actual_width.max(reference_width)
    }

    fn build_clock_layout(dc: Hdc, time_text: &str, date_text: &str) -> io::Result<ClockLayout> {
        let clock_font = create_font(CLOCK_FONT_SIZE)?;
        let date_font = match create_font(DATE_FONT_SIZE) {
            Ok(font) => font,
            Err(error) => {
                unsafe { DeleteObject(clock_font) };
                return Err(error);
            }
        };
        let previous_font = unsafe { SelectObject(dc, clock_font) };
        let clock_size = measure_text(dc, time_text);
        let sample_size = measure_text(dc, CLOCK_WIDTH_SAMPLE);
        let clock_ink = font_metrics(dc).map(|metrics| {
            merge_ink_bounds(
                text_ink_bounds(dc, time_text, &metrics),
                text_ink_bounds(dc, CLOCK_WIDTH_SAMPLE, &metrics),
            )
        });
        unsafe { SelectObject(dc, date_font) };
        let date_size = measure_text(dc, date_text);
        let date_ink = font_metrics(dc).map(|metrics| text_ink_bounds(dc, date_text, &metrics));
        unsafe {
            SelectObject(dc, previous_font);
            DeleteObject(clock_font);
            DeleteObject(date_font);
        }
        let clock_size = clock_size?;
        let sample_size = sample_size?;
        let clock_ink = clock_ink?;
        let date_size = date_size?;
        let date_ink = date_ink?;
        let (clock_y, date_y, height) = line_positions(clock_ink, date_ink);
        let clock_frame_width = clock_outer_width(clock_size.cx, sample_size.cx);
        Ok(ClockLayout {
            size: Size {
                cx: clock_frame_width.max(date_size.cx),
                cy: height,
            },
            clock_size,
            date_size,
            clock_frame_width,
            clock_y,
            date_y,
        })
    }

    fn measure_clock(hwnd: Hwnd, time_text: &str, date_text: &str) -> io::Result<Size> {
        let dc = unsafe { GetDC(hwnd) };
        if dc.is_null() {
            return Err(io::Error::last_os_error());
        }
        let layout = build_clock_layout(dc, time_text, date_text);
        unsafe { ReleaseDC(hwnd, dc) };
        layout.map(|layout| layout.size)
    }

    fn format_clock_text() -> io::Result<(String, String)> {
        let mut time_buffer = [0u16; 128];
        let time_length = unsafe {
            GetTimeFormatEx(
                ptr::null(),
                TIME_NOSECONDS,
                ptr::null(),
                ptr::null(),
                time_buffer.as_mut_ptr(),
                time_buffer.len() as i32,
            )
        };
        if time_length == 0 {
            return Err(io::Error::last_os_error());
        }

        let mut date_buffer = [0u16; 128];
        let date_length = unsafe {
            GetDateFormatEx(
                ptr::null(),
                0,
                ptr::null(),
                ptr::null(),
                date_buffer.as_mut_ptr(),
                date_buffer.len() as i32,
                ptr::null(),
            )
        };
        if date_length == 0 {
            return Err(io::Error::last_os_error());
        }

        Ok((
            String::from_utf16_lossy(&time_buffer[..time_length as usize - 1]),
            String::from_utf16_lossy(&date_buffer[..date_length as usize - 1]),
        ))
    }

    fn advance_axis(position: i32, velocity: i32, minimum: i32, maximum: i32) -> (i32, i32) {
        if maximum <= minimum {
            return (minimum, velocity);
        }
        let next = position + velocity;
        if next < minimum {
            (minimum, velocity.abs())
        } else if next > maximum {
            (maximum, -velocity.abs())
        } else {
            (next, velocity)
        }
    }

    fn vertical_bounds(screen_y: i32, screen_height: i32, overlay_height: i32) -> (i32, i32) {
        (screen_y, screen_y + (screen_height - overlay_height).max(0))
    }

    fn paint_clock(hwnd: Hwnd) {
        let (time_text, date_text, width, height) = {
            let state = CLOCK_STATE
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let Some(state) = state.as_ref() else {
                return;
            };
            (
                state.time_text.clone(),
                state.date_text.clone(),
                state.width,
                state.height,
            )
        };

        let mut paint: PaintStruct = unsafe { mem::zeroed() };
        let dc = unsafe { BeginPaint(hwnd, &mut paint) };
        if dc.is_null() {
            return;
        }
        let bounds = Rect {
            left: 0,
            top: 0,
            right: width,
            bottom: height,
        };
        unsafe { FillRect(dc, &bounds, GetStockObject(BLACK_BRUSH)) };

        if let Ok(layout) = build_clock_layout(dc, &time_text, &date_text) {
            let fonts = create_font(CLOCK_FONT_SIZE).and_then(|clock_font| {
                create_font(DATE_FONT_SIZE)
                    .map(|date_font| (clock_font, date_font))
                    .map_err(|error| {
                        unsafe { DeleteObject(clock_font) };
                        error
                    })
            });
            if let Ok((clock_font, date_font)) = fonts {
                let original_font = unsafe { SelectObject(dc, clock_font) };
                let clock_units: Vec<u16> = time_text.encode_utf16().collect();
                let date_units: Vec<u16> = date_text.encode_utf16().collect();
                let clock_x = (width - layout.clock_frame_width) / 2
                    + (layout.clock_frame_width - layout.clock_size.cx) / 2;
                unsafe {
                    SetBkMode(dc, 1);
                    SetTextColor(dc, TEXT_COLOR);
                    TextOutW(
                        dc,
                        clock_x,
                        layout.clock_y,
                        clock_units.as_ptr(),
                        clock_units.len() as i32,
                    );
                    SelectObject(dc, date_font);
                    TextOutW(
                        dc,
                        (width - layout.date_size.cx) / 2,
                        layout.date_y,
                        date_units.as_ptr(),
                        date_units.len() as i32,
                    );
                    SelectObject(dc, original_font);
                    DeleteObject(clock_font);
                    DeleteObject(date_font);
                }
            }
        }
        unsafe { EndPaint(hwnd, &paint) };
    }

    fn update_clock_text() {
        let Ok((time_text, date_text)) = format_clock_text() else {
            return;
        };
        let hwnd = CLOCK_WINDOW.load(Ordering::Acquire);
        if hwnd.is_null() {
            return;
        }
        let changed = {
            let state = CLOCK_STATE
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            state
                .as_ref()
                .is_some_and(|state| state.time_text != time_text || state.date_text != date_text)
        };
        if !changed {
            return;
        }
        let Ok(size) = measure_clock(hwnd, &time_text, &date_text) else {
            return;
        };

        let placement = {
            let mut state = CLOCK_STATE
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let Some(state) = state.as_mut() else {
                return;
            };
            if state.time_text == time_text && state.date_text == date_text {
                return;
            }
            state.time_text = time_text;
            state.date_text = date_text;
            state.width = size.cx;
            state.height = size.cy;
            let max_x = state.screen_x + (state.screen_width - size.cx).max(0);
            let (min_y, max_y) = vertical_bounds(state.screen_y, state.screen_height, size.cy);
            state.x = state.x.clamp(state.screen_x, max_x);
            state.y = state.y.clamp(min_y, max_y);
            (state.x, state.y, state.width, state.height)
        };
        unsafe {
            SetWindowPos(
                hwnd,
                ptr::null_mut(),
                placement.0,
                placement.1,
                placement.2,
                placement.3,
                SWP_NOZORDER | SWP_NOACTIVATE | SWP_NOREDRAW,
            );
            InvalidateRect(hwnd, ptr::null(), 0);
            UpdateWindow(hwnd);
        }
    }

    fn animate_clock() {
        let hwnd = CLOCK_WINDOW.load(Ordering::Acquire);
        if hwnd.is_null() {
            return;
        }
        let position = {
            let mut state = CLOCK_STATE
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let Some(state) = state.as_mut() else {
                return;
            };
            let max_x = state.screen_x + (state.screen_width - state.width).max(0);
            let (min_y, max_y) = vertical_bounds(state.screen_y, state.screen_height, state.height);
            (state.x, state.velocity_x) =
                advance_axis(state.x, state.velocity_x, state.screen_x, max_x);
            (state.y, state.velocity_y) = advance_axis(state.y, state.velocity_y, min_y, max_y);
            (state.x, state.y)
        };
        unsafe {
            SetWindowPos(
                hwnd,
                ptr::null_mut(),
                position.0,
                position.1,
                0,
                0,
                SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_NOREDRAW,
            );
        }
    }

    unsafe extern "system" fn clock_window_proc(
        hwnd: Hwnd,
        message: Uint,
        w_param: Wparam,
        l_param: Lparam,
    ) -> Lresult {
        if message == WM_PAINT {
            paint_clock(hwnd);
            return 0;
        }
        if message == WM_ERASEBKGND {
            return 1;
        }
        if message == WM_SETCURSOR {
            unsafe { SetCursor(ptr::null_mut()) };
            return 1;
        }
        unsafe { DefWindowProcW(hwnd, message, w_param, l_param) }
    }

    fn should_wake(message: Uint) -> bool {
        matches!(message, WM_KEYDOWN | WM_SYSKEYDOWN)
    }

    fn mouse_button(message: Uint, mouse_data: Dword) -> Option<(usize, bool)> {
        match message {
            0x0201 => Some((1, true)),
            0x0202 => Some((1, false)),
            0x0204 => Some((2, true)),
            0x0205 => Some((2, false)),
            0x0207 => Some((4, true)),
            0x0208 => Some((4, false)),
            0x020B | 0x020C => match mouse_data >> 16 {
                1 => Some((8, message == 0x020B)),
                2 => Some((16, message == 0x020B)),
                _ => None,
            },
            _ => None,
        }
    }

    unsafe extern "system" fn mouse_hook_proc(
        code: i32,
        w_param: Wparam,
        l_param: Lparam,
    ) -> Lresult {
        if code >= 0 {
            let data = unsafe { &*(l_param as *const MouseHookData) };
            if let Some((button, is_down)) = mouse_button(w_param as Uint, data.mouse_data) {
                let hwnd = WINDOW_HANDLE.load(Ordering::Acquire);
                if is_down {
                    MOUSE_BUTTONS_HELD.fetch_or(button, Ordering::AcqRel);
                    MOUSE_WAKE_PENDING.store(true, Ordering::Release);
                    unsafe {
                        PostMessageW(hwnd, WM_MOUSE_WAKE, 0, 0);
                    }
                    return 1;
                }

                if MOUSE_WAKE_PENDING.load(Ordering::Acquire) {
                    let remaining =
                        MOUSE_BUTTONS_HELD.fetch_and(!button, Ordering::AcqRel) & !button;
                    if remaining == 0 {
                        unsafe {
                            PostMessageW(hwnd, WM_MOUSE_WAKE_COMPLETE, 0, 0);
                        }
                    }
                    return 1;
                }
            }
        }

        unsafe { CallNextHookEx(ptr::null_mut(), code, w_param, l_param) }
    }

    unsafe extern "system" fn window_proc(
        hwnd: Hwnd,
        message: Uint,
        w_param: Wparam,
        l_param: Lparam,
    ) -> Lresult {
        if should_wake(message) || message == WM_CLOSE {
            unsafe {
                DestroyWindow(hwnd);
            }
            return 0;
        }

        if message == WM_MOUSE_WAKE {
            return 0;
        }

        if message == WM_MOUSE_WAKE_COMPLETE {
            if MOUSE_BUTTONS_HELD.load(Ordering::Acquire) == 0 {
                unsafe {
                    DestroyWindow(hwnd);
                }
            }
            return 0;
        }

        if message == WM_TIMER {
            if w_param == CLOCK_TIMER_ID {
                update_clock_text();
            } else if w_param == ANIMATION_TIMER_ID {
                animate_clock();
            }
            return 0;
        }

        if message == WM_DESTROY {
            unsafe {
                KillTimer(hwnd, CLOCK_TIMER_ID);
                KillTimer(hwnd, ANIMATION_TIMER_ID);
                let clock_window = CLOCK_WINDOW.swap(ptr::null_mut(), Ordering::AcqRel);
                if !clock_window.is_null() {
                    DestroyWindow(clock_window);
                }
                PostQuitMessage(0);
            }
            *CLOCK_STATE
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner()) = None;
            return 0;
        }

        if message == WM_SETCURSOR {
            unsafe {
                SetCursor(ptr::null_mut());
            }
            return 1;
        }

        unsafe { DefWindowProcW(hwnd, message, w_param, l_param) }
    }

    fn run() -> io::Result<()> {
        let class_name: Vec<u16> = "KlockerWindow\0".encode_utf16().collect();
        let clock_class_name: Vec<u16> = "KlockerClockWindow\0".encode_utf16().collect();
        let window_name: Vec<u16> = "Klocker\0".encode_utf16().collect();
        let clock_window_name: Vec<u16> = "KlockerClock\0".encode_utf16().collect();

        unsafe {
            let instance = GetModuleHandleW(ptr::null());
            if instance.is_null() {
                return Err(io::Error::last_os_error());
            }
            let (velocity_x, velocity_y) = random_starting_velocity()?;

            let window_class = WindowClass {
                style: 0,
                window_proc: Some(window_proc),
                class_extra: 0,
                window_extra: 0,
                instance,
                icon: ptr::null_mut(),
                cursor: ptr::null_mut(),
                background: GetStockObject(BLACK_BRUSH),
                menu_name: ptr::null(),
                class_name: class_name.as_ptr(),
            };
            if RegisterClassW(&window_class) == 0 {
                return Err(io::Error::last_os_error());
            }

            let clock_class = WindowClass {
                window_proc: Some(clock_window_proc),
                class_name: clock_class_name.as_ptr(),
                ..window_class
            };
            if RegisterClassW(&clock_class) == 0 {
                let error = io::Error::last_os_error();
                UnregisterClassW(class_name.as_ptr(), instance);
                return Err(error);
            }

            let x = GetSystemMetrics(SM_XVIRTUALSCREEN);
            let y = GetSystemMetrics(SM_YVIRTUALSCREEN);
            let width = GetSystemMetrics(SM_CXVIRTUALSCREEN);
            let height = GetSystemMetrics(SM_CYVIRTUALSCREEN);
            if width <= 0 || height <= 0 {
                UnregisterClassW(clock_class_name.as_ptr(), instance);
                UnregisterClassW(class_name.as_ptr(), instance);
                return Err(io::Error::other("Windows reported an invalid screen size."));
            }

            let hwnd = CreateWindowExW(
                WS_EX_TOPMOST | WS_EX_TOOLWINDOW,
                class_name.as_ptr(),
                window_name.as_ptr(),
                WS_POPUP,
                x,
                y,
                width,
                height,
                ptr::null_mut(),
                ptr::null_mut(),
                instance,
                ptr::null_mut(),
            );
            if hwnd.is_null() {
                let error = io::Error::last_os_error();
                UnregisterClassW(clock_class_name.as_ptr(), instance);
                UnregisterClassW(class_name.as_ptr(), instance);
                return Err(error);
            }

            let (time_text, date_text) = match format_clock_text() {
                Ok(text) => text,
                Err(error) => {
                    DestroyWindow(hwnd);
                    UnregisterClassW(clock_class_name.as_ptr(), instance);
                    UnregisterClassW(class_name.as_ptr(), instance);
                    return Err(error);
                }
            };
            let size = match measure_clock(hwnd, &time_text, &date_text) {
                Ok(size) => size,
                Err(error) => {
                    DestroyWindow(hwnd);
                    UnregisterClassW(clock_class_name.as_ptr(), instance);
                    UnregisterClassW(class_name.as_ptr(), instance);
                    return Err(error);
                }
            };
            let clock_x = x + (width - size.cx) / 2;
            let clock_y = y + (height - size.cy) / 2;
            let clock_hwnd = CreateWindowExW(
                WS_EX_TOPMOST
                    | WS_EX_TOOLWINDOW
                    | WS_EX_TRANSPARENT
                    | WS_EX_LAYERED
                    | WS_EX_NOACTIVATE,
                clock_class_name.as_ptr(),
                clock_window_name.as_ptr(),
                WS_POPUP,
                clock_x,
                clock_y,
                size.cx,
                size.cy,
                hwnd,
                ptr::null_mut(),
                instance,
                ptr::null_mut(),
            );
            if clock_hwnd.is_null() {
                let error = io::Error::last_os_error();
                DestroyWindow(hwnd);
                UnregisterClassW(clock_class_name.as_ptr(), instance);
                UnregisterClassW(class_name.as_ptr(), instance);
                return Err(error);
            }
            if SetLayeredWindowAttributes(clock_hwnd, 0, 0, LWA_COLORKEY) == 0 {
                let error = io::Error::last_os_error();
                DestroyWindow(clock_hwnd);
                DestroyWindow(hwnd);
                UnregisterClassW(clock_class_name.as_ptr(), instance);
                UnregisterClassW(class_name.as_ptr(), instance);
                return Err(error);
            }

            *CLOCK_STATE
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(ClockState {
                time_text,
                date_text,
                x: clock_x,
                y: clock_y,
                velocity_x,
                velocity_y,
                width: size.cx,
                height: size.cy,
                screen_x: x,
                screen_y: y,
                screen_width: width,
                screen_height: height,
            });
            CLOCK_WINDOW.store(clock_hwnd, Ordering::Release);

            let arrow_cursor = LoadCursorW(ptr::null_mut(), 32512usize as *const u16);
            SetCursor(ptr::null_mut());
            WINDOW_HANDLE.store(hwnd, Ordering::Release);
            MOUSE_BUTTONS_HELD.store(0, Ordering::Release);
            MOUSE_WAKE_PENDING.store(false, Ordering::Release);
            let mouse_hook = SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_hook_proc), instance, 0);
            if mouse_hook.is_null() {
                let error = io::Error::last_os_error();
                WINDOW_HANDLE.store(ptr::null_mut(), Ordering::Release);
                DestroyWindow(hwnd);
                SetCursor(arrow_cursor);
                UnregisterClassW(clock_class_name.as_ptr(), instance);
                UnregisterClassW(class_name.as_ptr(), instance);
                return Err(error);
            }

            if SetTimer(
                hwnd,
                CLOCK_TIMER_ID,
                CLOCK_REFRESH_INTERVAL,
                ptr::null_mut(),
            ) == 0
                || SetTimer(hwnd, ANIMATION_TIMER_ID, 16, ptr::null_mut()) == 0
            {
                let error = io::Error::last_os_error();
                KillTimer(hwnd, CLOCK_TIMER_ID);
                KillTimer(hwnd, ANIMATION_TIMER_ID);
                UnhookWindowsHookEx(mouse_hook);
                WINDOW_HANDLE.store(ptr::null_mut(), Ordering::Release);
                DestroyWindow(hwnd);
                SetCursor(arrow_cursor);
                UnregisterClassW(clock_class_name.as_ptr(), instance);
                UnregisterClassW(class_name.as_ptr(), instance);
                return Err(error);
            }

            ShowWindow(hwnd, SW_SHOW);
            UpdateWindow(hwnd);
            ShowWindow(clock_hwnd, SW_SHOWNOACTIVATE);
            UpdateWindow(clock_hwnd);
            SetForegroundWindow(hwnd);

            let mut message: Message = mem::zeroed();
            loop {
                let result = GetMessageW(&mut message, ptr::null_mut(), 0, 0);
                if result == -1 {
                    let error = io::Error::last_os_error();
                    UnhookWindowsHookEx(mouse_hook);
                    WINDOW_HANDLE.store(ptr::null_mut(), Ordering::Release);
                    DestroyWindow(hwnd);
                    SetCursor(arrow_cursor);
                    UnregisterClassW(clock_class_name.as_ptr(), instance);
                    UnregisterClassW(class_name.as_ptr(), instance);
                    return Err(error);
                }
                if result == 0 {
                    break;
                }
                TranslateMessage(&message);
                DispatchMessageW(&message);
            }

            UnhookWindowsHookEx(mouse_hook);
            WINDOW_HANDLE.store(ptr::null_mut(), Ordering::Release);
            SetCursor(arrow_cursor);
            UnregisterClassW(clock_class_name.as_ptr(), instance);
            UnregisterClassW(class_name.as_ptr(), instance);
        }
        Ok(())
    }

    #[link(name = "gdi32")]
    unsafe extern "system" {
        fn CreateFontW(
            height: i32,
            width: i32,
            escapement: i32,
            orientation: i32,
            weight: i32,
            italic: Dword,
            underline: Dword,
            strike_out: Dword,
            char_set: Dword,
            output_precision: Dword,
            clip_precision: Dword,
            quality: Dword,
            pitch_and_family: Dword,
            face_name: *const u16,
        ) -> Handle;
        fn DeleteObject(object: Handle) -> Bool;
        fn GetTextExtentPoint32W(dc: Hdc, text: *const u16, count: i32, size: *mut Size) -> Bool;
        fn GetGlyphOutlineW(
            dc: Hdc,
            character: Uint,
            format: Uint,
            metrics: *mut GlyphMetrics,
            buffer_size: Dword,
            buffer: *mut c_void,
            transform: *const Mat2,
        ) -> Dword;
        fn GetTextMetricsW(dc: Hdc, metrics: *mut TextMetric) -> Bool;
        fn GetStockObject(object: i32) -> Hbrush;
        fn SelectObject(dc: Hdc, object: Handle) -> Handle;
        fn SetBkMode(dc: Hdc, mode: i32) -> i32;
        fn SetTextColor(dc: Hdc, color: Dword) -> Dword;
        fn TextOutW(dc: Hdc, x: i32, y: i32, text: *const u16, count: i32) -> Bool;
    }

    pub fn start() -> io::Result<()> {
        run()
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn wakes_only_for_keyboard_or_mouse_button_presses() {
            for message in [WM_KEYDOWN, WM_SYSKEYDOWN] {
                assert!(should_wake(message));
            }

            for message in [0x0200, 0x0101, 0x0202] {
                assert!(!should_wake(message));
            }
        }

        #[test]
        fn recognizes_mouse_button_down_and_release_events() {
            for (down, up, data) in [
                (0x0201, 0x0202, 0),
                (0x0204, 0x0205, 0),
                (0x0207, 0x0208, 0),
                (0x020B, 0x020C, 1 << 16),
                (0x020B, 0x020C, 2 << 16),
            ] {
                let (down_button, down_state) = mouse_button(down, data).unwrap();
                let (up_button, up_state) = mouse_button(up, data).unwrap();
                assert_eq!(down_button, up_button);
                assert!(down_state);
                assert!(!up_state);
            }

            assert_eq!(mouse_button(0x0200, 0), None);
            assert_eq!(mouse_button(0x020B, 0), None);
        }

        #[test]
        fn bounce_reverses_at_each_screen_edge() {
            assert_eq!(advance_axis(10, -3, 10, 100), (10, 3));
            assert_eq!(advance_axis(100, 4, 10, 100), (100, -4));
            assert_eq!(advance_axis(50, 3, 10, 100), (53, 3));
        }

        #[test]
        fn starting_velocities_cover_eight_distinct_directions_at_same_speed() {
            for first in 0..START_VELOCITIES.len() {
                let velocity = velocity_for_direction(first as u8);
                assert_eq!(velocity.0.abs() + velocity.1.abs(), 7);

                for second in first + 1..START_VELOCITIES.len() {
                    assert_ne!(velocity, velocity_for_direction(second as u8));
                }
            }
            assert_eq!(velocity_for_direction(8), velocity_for_direction(0));
        }

        #[test]
        fn vertical_bounds_allow_the_text_edge_to_reach_the_screen_edge() {
            assert_eq!(vertical_bounds(0, 800, 200), (0, 600));
        }

        #[test]
        fn date_position_preserves_fifteen_pixels_between_ink_bounds() {
            let clock_ink = InkBounds {
                top: 18,
                bottom: 103,
            };
            let date_ink = InkBounds { top: 3, bottom: 18 };
            let (clock_y, date_y, _) = line_positions(clock_ink, date_ink);

            assert_eq!(date_y + date_ink.top - (clock_y + clock_ink.bottom), 15);
        }

        #[test]
        fn clock_frame_uses_the_larger_of_sample_or_actual_text_width() {
            assert_eq!(clock_outer_width(300, 360), 360);
            assert_eq!(clock_outer_width(400, 360), 400);
        }
    }
}

#[cfg(windows)]
fn main() -> std::process::ExitCode {
    match screensaver::start() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Klocker failed: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}

#[cfg(not(windows))]
fn main() -> std::process::ExitCode {
    eprintln!("Klocker currently supports Windows only.");
    std::process::ExitCode::FAILURE
}
