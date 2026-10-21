//! Small ABI boundary, declared from the installed Apple public SDK headers.
//! All create/copy references are owned; get/property references are borrowed.
use std::ffi::{c_char, c_void, CStr};
use std::ptr;

use crate::{error, Result};

pub type Ref = *const c_void;
pub type MutRef = *mut c_void;
pub type Id = *mut c_void;
pub type Sel = *const c_void;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[repr(C)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[repr(C)]
pub struct Size {
    pub width: f64,
    pub height: f64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[repr(C)]
pub struct Rect {
    pub origin: Point,
    pub size: Size,
}

pub struct OwnedCf(pub Ref);
impl OwnedCf {
    pub fn new(value: Ref, what: &str) -> Result<Self> {
        if value.is_null() {
            Err(error(format!("{what} returned NULL")))
        } else {
            Ok(Self(value))
        }
    }
}
impl Drop for OwnedCf {
    fn drop(&mut self) {
        // Safety: self owns one non-null create/copy CF reference and releases it once.
        unsafe { CFRelease(self.0) }
    }
}

#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    pub fn CFRelease(value: Ref);
    pub fn CFRetain(value: Ref) -> Ref;
    pub fn CFDataGetBytePtr(data: Ref) -> *const u8;
    pub fn CFDataGetLength(data: Ref) -> isize;
    pub fn CFDataCreate(allocator: Ref, bytes: *const u8, length: isize) -> Ref;
    pub fn CFStringGetLength(string: Ref) -> isize;
    pub fn CFStringCreateMutable(allocator: Ref, maximum: isize) -> MutRef;
    pub fn CFStringAppendCharacters(string: MutRef, characters: *const u16, count: isize);
    pub fn CFStringNormalize(string: MutRef, form: isize);
    pub fn CFStringGetCharacterAtIndex(string: Ref, index: isize) -> u16;
    pub fn CFStringGetMaximumSizeForEncoding(length: isize, encoding: u32) -> isize;
    pub fn CFStringGetCString(string: Ref, buffer: *mut c_char, size: isize, encoding: u32) -> bool;
    pub fn CFDictionaryCreate(
        allocator: Ref,
        keys: *const Ref,
        values: *const Ref,
        count: isize,
        key_callbacks: Ref,
        value_callbacks: Ref,
    ) -> Ref;
    pub fn CFDictionaryGetValue(dictionary: Ref, key: Ref) -> Ref;
    pub fn CFNumberGetValue(number: Ref, number_type: isize, output: *mut c_void) -> bool;
    pub fn CFRunLoopGetCurrent() -> Ref;
    pub fn CFRunLoopAddSource(run_loop: Ref, source: Ref, mode: Ref);
    pub fn CFRunLoopRemoveSource(run_loop: Ref, source: Ref, mode: Ref);
    pub fn CFRunLoopRunInMode(mode: Ref, seconds: f64, return_after_source: bool) -> i32;
    pub fn CFRunLoopWakeUp(run_loop: Ref);
    pub static kCFRunLoopCommonModes: Ref;
    pub static kCFRunLoopDefaultMode: Ref;
    pub fn CFMachPortCreateRunLoopSource(allocator: Ref, port: Ref, order: isize) -> Ref;
    pub fn CFMachPortInvalidate(port: Ref);
}

pub fn cf_string(value: Ref) -> Result<String> {
    if value.is_null() {
        return Err(error("missing CFString"));
    }
    // Safety: live native references and checked buffers outlive these SDK-matched public API calls.
    unsafe {
        let size = CFStringGetMaximumSizeForEncoding(CFStringGetLength(value), 0x08000100) + 1;
        if size <= 0 {
            return Err(error("invalid CFString length"));
        }
        let mut bytes = vec![0u8; size as usize];
        if !CFStringGetCString(value, bytes.as_mut_ptr().cast(), size, 0x08000100) {
            return Err(error("cannot encode CFString as UTF-8"));
        }
        Ok(CStr::from_ptr(bytes.as_ptr().cast()).to_string_lossy().into_owned())
    }
}

#[link(name = "Carbon", kind = "framework")]
unsafe extern "C" {
    pub fn TISCopyCurrentKeyboardLayoutInputSource() -> Ref;
    pub fn TISGetInputSourceProperty(source: Ref, key: Ref) -> Ref;
    pub static kTISPropertyInputSourceID: Ref;
    pub static kTISPropertyUnicodeKeyLayoutData: Ref;
    pub fn UCKeyTranslate(
        layout: Ref,
        code: u16,
        action: u16,
        modifiers: u32,
        keyboard_type: u32,
        options: u32,
        dead_state: *mut u32,
        maximum: usize,
        actual: *mut usize,
        output: *mut u16,
    ) -> i32;
    pub fn IsSecureEventInputEnabled() -> bool;
}

pub type TapCallback = unsafe extern "C" fn(MutRef, u32, MutRef, MutRef) -> MutRef;

#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
    pub fn CGEventSourceCreate(state: i32) -> Ref;
    pub fn CGEventSourceGetKeyboardType(source: Ref) -> u32;
    pub fn CGEventSourceCounterForEventType(state: i32, kind: u32) -> u32;
    pub fn CGEventSourceSetPixelsPerLine(source: Ref, pixels: f64);
    pub fn CGEventSourceSetLocalEventsSuppressionInterval(source: Ref, seconds: f64);
    pub fn CGEventCreate(source: Ref) -> MutRef;
    pub fn CGEventCreateMouseEvent(source: Ref, kind: u32, point: Point, button: u32) -> MutRef;
    pub fn CGEventCreateKeyboardEvent(source: Ref, code: u16, down: bool) -> MutRef;
    pub fn CGEventCreateScrollWheelEvent2(
        source: Ref,
        units: u32,
        axes: u32,
        vertical: i32,
        horizontal: i32,
        third: i32,
    ) -> MutRef;
    pub fn CGEventGetLocation(event: Ref) -> Point;
    pub fn CGEventGetFlags(event: Ref) -> u64;
    pub fn CGEventSetFlags(event: MutRef, flags: u64);
    pub fn CGEventSetIntegerValueField(event: MutRef, field: u32, value: i64);
    pub fn CGEventGetIntegerValueField(event: Ref, field: u32) -> i64;
    pub fn CGEventSetDoubleValueField(event: MutRef, field: u32, value: f64);
    pub fn CGEventGetDoubleValueField(event: Ref, field: u32) -> f64;
    pub fn CGEventKeyboardSetUnicodeString(event: MutRef, length: usize, text: *const u16);
    pub fn CGEventPost(tap: u32, event: Ref);
    pub fn CGEventTapCreate(tap: u32, place: u32, options: u32, mask: u64, callback: TapCallback, user: MutRef) -> Ref;
    pub fn CGEventTapEnable(tap: Ref, enable: bool);
    pub fn CGPreflightListenEventAccess() -> bool;
    pub fn CGPreflightPostEventAccess() -> bool;
    pub fn CGWarpMouseCursorPosition(point: Point) -> i32;
    pub fn CGDisplayHideCursor(display: u32) -> i32;
    pub fn CGDisplayShowCursor(display: u32) -> i32;
    pub fn CGMainDisplayID() -> u32;
    pub fn CGGetActiveDisplayList(maximum: u32, displays: *mut u32, count: *mut u32) -> i32;
    pub fn CGDisplayBounds(display: u32) -> Rect;
    pub fn CGDisplayCopyDisplayMode(display: u32) -> Ref;
    pub fn CGDisplayModeGetWidth(mode: Ref) -> usize;
    pub fn CGDisplayModeGetHeight(mode: Ref) -> usize;
    pub fn CGDisplayModeGetPixelWidth(mode: Ref) -> usize;
    pub fn CGDisplayModeGetPixelHeight(mode: Ref) -> usize;
    pub fn CGColorSpaceCreateWithName(name: Ref) -> Ref;
    pub static kCGColorSpaceSRGB: Ref;
    pub fn CGBitmapContextCreate(
        data: MutRef,
        width: usize,
        height: usize,
        bits: usize,
        row_bytes: usize,
        color_space: Ref,
        bitmap_info: u32,
    ) -> Ref;
    pub fn CGContextDrawImage(context: Ref, rect: Rect, image: Ref);
    pub fn CGImageGetWidth(image: Ref) -> usize;
    pub fn CGImageGetHeight(image: Ref) -> usize;
}

#[link(name = "ApplicationServices", kind = "framework")]
unsafe extern "C" {
    pub fn AXIsProcessTrusted() -> bool;
}

// ImageIO is Apple's public image codec framework; no external image library.
#[link(name = "ImageIO", kind = "framework")]
unsafe extern "C" {
    pub fn CGImageSourceCreateWithData(data: Ref, options: Ref) -> Ref;
    pub fn CGImageSourceCopyPropertiesAtIndex(source: Ref, index: usize, options: Ref) -> Ref;
    pub static kCGImagePropertyPixelWidth: Ref;
    pub static kCGImagePropertyPixelHeight: Ref;
}

#[link(name = "AppKit", kind = "framework")]
unsafe extern "C" {}
#[link(name = "Foundation", kind = "framework")]
unsafe extern "C" {}
#[link(name = "objc")]
unsafe extern "C" {
    fn objc_getClass(name: *const c_char) -> Id;
    fn sel_registerName(name: *const c_char) -> Sel;
    fn objc_msgSend();
}

pub fn class(name: &'static CStr) -> Id {
    // Safety: name is a static NUL-terminated Objective-C class name.
    unsafe { objc_getClass(name.as_ptr()) }
}
fn selector(name: &'static CStr) -> Sel {
    // Safety: name is a static NUL-terminated public selector name.
    unsafe { sel_registerName(name.as_ptr()) }
}

// Only private, statically selected selectors use these helpers. Each call must
// match the public header's exact signature. No struct-return messages are used.
pub unsafe fn msg0<R>(object: Id, name: &'static CStr) -> R {
    let function: unsafe extern "C" fn(Id, Sel) -> R =
        // Safety: call sites supply the exact argument and return ABI of the static public selector.
        unsafe { std::mem::transmute(objc_msgSend as unsafe extern "C" fn()) };
    // Safety: caller guarantees the receiver and arguments match this selector ABI.
    unsafe { function(object, selector(name)) }
}
pub unsafe fn msg1<R, A>(object: Id, name: &'static CStr, a: A) -> R {
    let function: unsafe extern "C" fn(Id, Sel, A) -> R =
        // Safety: call sites supply the exact argument and return ABI of the static public selector.
        unsafe { std::mem::transmute(objc_msgSend as unsafe extern "C" fn()) };
    // Safety: caller guarantees the receiver and arguments match this selector ABI.
    unsafe { function(object, selector(name), a) }
}
pub unsafe fn msg2<R, A, B>(object: Id, name: &'static CStr, a: A, b: B) -> R {
    let function: unsafe extern "C" fn(Id, Sel, A, B) -> R =
        // Safety: call sites supply the exact argument and return ABI of the static public selector.
        unsafe { std::mem::transmute(objc_msgSend as unsafe extern "C" fn()) };
    // Safety: caller guarantees the receiver and arguments match this selector ABI.
    unsafe { function(object, selector(name), a, b) }
}
pub unsafe fn msg3<R, A, B, C>(object: Id, name: &'static CStr, a: A, b: B, c: C) -> R {
    let function: unsafe extern "C" fn(Id, Sel, A, B, C) -> R =
        // Safety: call sites supply the exact argument and return ABI of the static public selector.
        unsafe { std::mem::transmute(objc_msgSend as unsafe extern "C" fn()) };
    // Safety: caller guarantees the receiver and arguments match this selector ABI.
    unsafe { function(object, selector(name), a, b, c) }
}

pub struct OwnedObj(pub Id);
impl OwnedObj {
    pub fn new(value: Id, what: &str) -> Result<Self> {
        if value.is_null() {
            Err(error(format!("{what} returned nil")))
        } else {
            Ok(Self(value))
        }
    }
}
impl Drop for OwnedObj {
    fn drop(&mut self) {
        // Safety: self owns one retained Objective-C reference and releases it once.
        unsafe { msg0::<()>(self.0, c"release") }
    }
}
pub struct Pool(Id);
impl Pool {
    pub fn new() -> Self {
        // Safety: NSAutoreleasePool alloc/init has the object-return ABI and stays on this thread.
        unsafe { Self(msg0(msg0::<Id>(class(c"NSAutoreleasePool"), c"alloc"), c"init")) }
    }
}
impl Drop for Pool {
    fn drop(&mut self) {
        // Safety: this live thread-local autorelease pool drains exactly once.
        unsafe { msg0::<()>(self.0, c"drain") }
    }
}
pub fn ns_string(text: &str) -> Result<OwnedObj> {
    // Safety: live native references and checked buffers outlive these SDK-matched public API calls.
    unsafe {
        let allocated: Id = msg0(class(c"NSString"), c"alloc");
        OwnedObj::new(
            msg3(
                allocated,
                c"initWithBytes:length:encoding:",
                text.as_ptr(),
                text.len(),
                4usize,
            ),
            "NSString init",
        )
    }
}
pub fn ns_data(bytes: &[u8]) -> Result<OwnedObj> {
    // Safety: live native references and checked buffers outlive these SDK-matched public API calls.
    unsafe {
        let allocated: Id = msg0(class(c"NSData"), c"alloc");
        OwnedObj::new(
            msg2(allocated, c"initWithBytes:length:", bytes.as_ptr(), bytes.len()),
            "NSData init",
        )
    }
}
pub fn data_bytes(object: Id, maximum: usize) -> Result<Vec<u8>> {
    if object.is_null() {
        return Err(error("missing NSData"));
    }
    // Safety: live native references and checked buffers outlive these SDK-matched public API calls.
    unsafe {
        let length: usize = msg0(object, c"length");
        if length > maximum {
            return Err(error(format!("data length {length} exceeds max_bytes {maximum}")));
        }
        if length == 0 {
            return Ok(Vec::new());
        }
        let bytes: *const u8 = msg0(object, c"bytes");
        if bytes.is_null() {
            return Err(error("NSData bytes is NULL"));
        }
        Ok(std::slice::from_raw_parts(bytes, length).to_vec())
    }
}

pub fn empty_dictionary() -> Id {
    // Safety: NSDictionary dictionary returns an autoreleased object inside the caller pool.
    unsafe { msg0(class(c"NSDictionary"), c"dictionary") }
}
pub fn null() -> Ref {
    ptr::null()
}

/// Allocate an owned, non-planar, 8-bit straight RGBA bitmap with its own bytes.
pub fn rgba_bitmap(width: usize, height: usize) -> Result<OwnedObj> {
    // Safety: live native references and checked buffers outlive these SDK-matched public API calls.
    unsafe {
        let allocated: Id = msg0(class(c"NSBitmapImageRep"), c"alloc");
        let function: unsafe extern "C" fn(
            Id,
            Sel,
            *mut *mut u8,
            isize,
            isize,
            isize,
            isize,
            bool,
            bool,
            Id,
            usize,
            isize,
            isize,
        ) -> Id = std::mem::transmute(objc_msgSend as unsafe extern "C" fn());
        let color_space = ns_string("NSCalibratedRGBColorSpace")?;
        let bitmap = OwnedObj::new(function(allocated,
            selector(c"initWithBitmapDataPlanes:pixelsWide:pixelsHigh:bitsPerSample:samplesPerPixel:hasAlpha:isPlanar:colorSpaceName:bitmapFormat:bytesPerRow:bitsPerPixel:"),
            ptr::null_mut(), width as isize, height as isize, 8, 4, true, false,
            color_space.0, 2, (width * 4) as isize, 32), "RGBA NSBitmapImageRep")?;
        // The initializer accepts a legacy color-space name. Retag the bytes
        // explicitly as sRGB (without color conversion) before PNG/TIFF encode.
        let srgb: Id = msg0(class(c"NSColorSpace"), c"sRGBColorSpace");
        let tagged: Id = msg1(bitmap.0, c"bitmapImageRepByRetaggingWithColorSpace:", srgb);
        if tagged.is_null() {
            return Err(error("cannot tag RGBA bitmap as sRGB"));
        }
        OwnedObj::new(msg0(tagged, c"retain"), "retained sRGB bitmap")
    }
}

// Safety: each owned CF value moves exclusively between threads; it is never
// shared mutably. Tap/run-loop objects are created and destroyed on their owner.
unsafe impl Send for OwnedCf {}

#[repr(C)]
pub struct RunLoopSourceContext {
    pub version: isize,
    pub info: MutRef,
    pub retain: Option<unsafe extern "C" fn(Ref) -> Ref>,
    pub release: Option<unsafe extern "C" fn(Ref)>,
    pub description: Option<unsafe extern "C" fn(Ref) -> Ref>,
    pub equal: Option<unsafe extern "C" fn(Ref, Ref) -> bool>,
    pub hash: Option<unsafe extern "C" fn(Ref) -> usize>,
    pub schedule: Option<unsafe extern "C" fn(MutRef, Ref, Ref)>,
    pub cancel: Option<unsafe extern "C" fn(MutRef, Ref, Ref)>,
    pub perform: Option<unsafe extern "C" fn(MutRef)>,
}

pub type NotificationCallback = unsafe extern "C" fn(Ref, MutRef, Ref, Ref, Ref);
#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    pub fn CFRunLoopSourceCreate(allocator: Ref, order: isize, context: *mut RunLoopSourceContext) -> Ref;
    pub fn CFRunLoopSourceSignal(source: Ref);
    pub fn CFRunLoopSourceInvalidate(source: Ref);
    pub fn CFNotificationCenterGetDistributedCenter() -> Ref;
    pub fn CFNotificationCenterAddObserver(
        center: Ref,
        observer: Ref,
        callback: NotificationCallback,
        name: Ref,
        object: Ref,
        suspension: isize,
    );
    pub fn CFNotificationCenterRemoveEveryObserver(center: Ref, observer: Ref);
}
#[link(name = "Carbon", kind = "framework")]
unsafe extern "C" {
    pub static kTISNotifySelectedKeyboardInputSourceChanged: Ref;
}
#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
    pub fn CGEventGetType(event: Ref) -> u32;
    pub fn CGEventKeyboardGetUnicodeString(event: Ref, maximum: usize, actual: *mut usize, text: *mut u16);
    pub fn CGAssociateMouseAndMouseCursorPosition(connected: bool) -> i32;
    pub fn CGDisplayRegisterReconfigurationCallback(
        callback: unsafe extern "C" fn(u32, u32, MutRef),
        user: MutRef,
    ) -> i32;
    pub fn CGDisplayRemoveReconfigurationCallback(
        callback: unsafe extern "C" fn(u32, u32, MutRef),
        user: MutRef,
    ) -> i32;
}
#[link(name = "IOKit", kind = "framework")]
unsafe extern "C" {
    pub fn IORegisterForSystemPower(
        user: MutRef,
        port: *mut MutRef,
        callback: unsafe extern "C" fn(MutRef, u32, u32, MutRef),
        notifier: *mut u32,
    ) -> u32;
    pub fn IODeregisterForSystemPower(notifier: *mut u32) -> i32;
    pub fn IOAllowPowerChange(connection: u32, notification: isize) -> i32;
    pub fn IOServiceClose(connection: u32) -> i32;
    pub fn IONotificationPortGetRunLoopSource(port: MutRef) -> Ref;
    pub fn IONotificationPortDestroy(port: MutRef);
    pub fn IOPMAssertionCreateWithName(kind: Ref, level: u32, name: Ref, assertion: *mut u32) -> i32;
    pub fn IOPMAssertionRelease(assertion: u32) -> i32;
}

/// A signallable source prevents the stop-before-run race without polling.
/// The controller holds retained source + loop refs until after thread join.
pub struct RunLoopWake {
    pub loop_ref: usize,
    pub source: usize,
}
impl RunLoopWake {
    pub fn new() -> Result<(Self, OwnedCf)> {
        let mut context = RunLoopSourceContext {
            version: 0,
            info: ptr::null_mut(),
            retain: None,
            release: None,
            description: None,
            equal: None,
            hash: None,
            schedule: None,
            cancel: None,
            perform: Some(wake_perform),
        };
        // Safety: the copied context contains no borrowed data; loop is current.
        unsafe {
            let source = OwnedCf::new(
                CFRunLoopSourceCreate(null(), 0, &mut context),
                "control run-loop source",
            )?;
            let loop_ref = CFRetain(CFRunLoopGetCurrent()) as usize;
            CFRunLoopAddSource(loop_ref as Ref, source.0, kCFRunLoopCommonModes);
            Ok((
                Self {
                    loop_ref,
                    source: CFRetain(source.0) as usize,
                },
                source,
            ))
        }
    }
    pub fn wake(&self) {
        // Safety: these retained refs outlive wake calls, and both APIs are thread-safe.
        unsafe {
            CFRunLoopSourceSignal(self.source as Ref);
            CFRunLoopWakeUp(self.loop_ref as Ref);
        }
    }
}
unsafe extern "C" fn wake_perform(_: MutRef) {}
impl Drop for RunLoopWake {
    fn drop(&mut self) {
        // Safety: controller/thread have joined or hold their own retained refs.
        unsafe {
            CFRelease(self.source as Ref);
            CFRelease(self.loop_ref as Ref);
        }
    }
}

#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    pub static kCFBooleanTrue: Ref;
}
#[link(name = "ApplicationServices", kind = "framework")]
unsafe extern "C" {
    pub fn AXIsProcessTrustedWithOptions(options: Ref) -> bool;
    pub static kAXTrustedCheckOptionPrompt: Ref;
}
#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
    pub fn CGRequestListenEventAccess() -> bool;
    pub fn CGRequestPostEventAccess() -> bool;
}
