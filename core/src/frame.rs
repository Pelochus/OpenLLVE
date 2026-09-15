use crate::error::{CoreError, Result};

/// Abstraction for zero-copy native frame handles across platforms
/// (e.g. AHardwareBuffer on Android, CVPixelBuffer on iOS).
///
/// The caller owns the memory behind `data_ptr` for the lifetime of the
/// handle; the handle never frees it. The pixel format is caller-defined;
/// `stride` is in bytes and must be at least `width` (one byte per pixel
/// column).
pub struct NativeFrameHandle {
    width: u32,
    height: u32,
    stride: usize,
    data_ptr: *mut u8,
    is_hardware_buffer: bool,
}

impl NativeFrameHandle {
    /// Creates a frame handle.
    ///
    /// # Errors
    /// Returns [`CoreError::InvalidParameter`] if `data_ptr` is null or
    /// `stride < width`.
    pub fn new(width: u32, height: u32, stride: usize, data_ptr: *mut u8, is_hardware_buffer: bool) -> Result<Self> {
        if data_ptr.is_null() || stride < width as usize {
            return Err(CoreError::InvalidParameter(format!(
                "invalid frame handle: stride {} (bytes) must be >= width {} (pixels)",
                stride, width
            )));
        }
        Ok(Self {
            width,
            height,
            stride,
            data_ptr,
            is_hardware_buffer,
        })
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

impl std::fmt::Debug for NativeFrameHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NativeFrameHandle")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("stride", &self.stride)
            .field("data_ptr", &self.data_ptr)
            .field("is_hardware_buffer", &self.is_hardware_buffer)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_handle_valid() {
        let mut data = vec![0u8; 64];
        let handle = NativeFrameHandle::new(4, 4, 16, data.as_mut_ptr(), false).unwrap();
        assert_eq!(handle.width(), 4);
        assert_eq!(handle.height(), 4);
        assert_eq!(handle.stride(), 16);
        assert!(!handle.is_hardware_buffer());
    }

    #[test]
    fn test_frame_handle_rejects_null_ptr() {
        let err = NativeFrameHandle::new(4, 4, 16, std::ptr::null_mut(), false).unwrap_err();
        assert!(matches!(err, CoreError::InvalidParameter(_)));
    }

    #[test]
    fn test_frame_handle_rejects_small_stride() {
        let mut data = vec![0u8; 4];
        let err = NativeFrameHandle::new(4, 4, 3, data.as_mut_ptr(), false).unwrap_err();
        assert!(matches!(err, CoreError::InvalidParameter(_)));
    }
}
