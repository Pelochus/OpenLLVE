/// Abstraction for zero-copy native frame handles across platforms 
/// (e.g. AHardwareBuffer on Android, CVPixelBuffer on iOS).
pub struct NativeFrameHandle {
    width: u32,
    height: u32,
    stride: usize,
    data_ptr: *mut u8,
    is_hardware_buffer: bool,
}

impl NativeFrameHandle {
    pub fn new(width: u32, height: u32, stride: usize, data_ptr: *mut u8, is_hardware_buffer: bool) -> Self {
        Self {
            width,
            height,
            stride,
            data_ptr,
            is_hardware_buffer,
        }
    }

    #[inline]
    pub fn width(&self) -> u32 {
        self.width
    }

    #[inline]
    pub fn height(&self) -> u32 {
        self.height
    }

    #[inline]
    pub fn stride(&self) -> usize {
        self.stride
    }

    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        self.data_ptr as *const u8
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.data_ptr
    }

    #[inline]
    pub fn is_hardware_buffer(&self) -> bool {
        self.is_hardware_buffer
    }
}
