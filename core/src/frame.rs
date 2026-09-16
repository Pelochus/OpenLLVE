//! First-class frame types.
//!
//! Frames are validated, zero-copy views over caller-owned buffers. The
//! caller owns the memory for the lifetime of the view; the views never
//! free it. `stride` is in **bytes per row** and must be at least
//! `width * channels * element_size`.
//!
//! - [`FrameRef`] is an immutable view (input frames; C `const float*`).
//! - [`Frame`] is a mutable view (output frames; C `float*`).
//! - [`OwnedFrame`] is an owned, compact frame (no stride padding) used for
//!   temporal model state (e.g. EWMA's previous frame).

use crate::error::{CoreError, Result};

/// Element format of a frame buffer.
///
/// Only `F32` exists today; further formats (e.g. `U8` for hardware
/// buffers) will be added when `NativeFrameHandle` is wired into the FFI
/// (P1.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameFormat {
    /// 32-bit float per channel.
    F32,
}

impl FrameFormat {
    /// Size of one element in bytes.
    pub fn element_size(&self) -> usize {
        match self {
            FrameFormat::F32 => 4,
        }
    }
}

/// Validates frame dimensions against a data buffer length (in elements).
fn validate(width: u32, height: u32, channels: u32, stride: usize, data_len: usize) -> Result<()> {
    let elem = FrameFormat::F32.element_size();
    if width == 0 || height == 0 || channels == 0 {
        return Err(CoreError::InvalidParameter(
            "frame width, height and channels must be non-zero".to_string(),
        ));
    }
    if !stride.is_multiple_of(elem) {
        return Err(CoreError::InvalidParameter(format!(
            "stride {} bytes must be a multiple of the element size ({} bytes)",
            stride, elem
        )));
    }
    let min_stride = width as usize * channels as usize * elem;
    if stride < min_stride {
        return Err(CoreError::InvalidParameter(format!(
            "stride {} bytes must be >= width*channels*element_size ({})",
            stride, min_stride
        )));
    }
    let min_len = stride * height as usize / elem;
    if data_len < min_len {
        return Err(CoreError::InvalidParameter(format!(
            "buffer of {} elements must be >= stride*height/element_size ({})",
            data_len, min_len
        )));
    }
    Ok(())
}

/// Immutable, zero-copy view of a caller-owned frame buffer.
///
/// Created from a `&[f32]` (or a byte buffer via [`FrameRef::from_bytes`]).
/// The caller owns the buffer; the view never frees it.
#[derive(Debug)]
pub struct FrameRef<'a> {
    width: u32,
    height: u32,
    channels: u32,
    stride: usize,
    format: FrameFormat,
    data: &'a [f32],
}

impl<'a> FrameRef<'a> {
    /// Creates a validated frame view over `data`.
    ///
    /// `stride` is in bytes per row and must be a multiple of the element
    /// size, at least `width * channels * element_size`; `data` must hold at
    /// least `stride * height / element_size` elements.
    ///
    /// # Errors
    /// Returns [`CoreError::InvalidParameter`] if any dimension or buffer
    /// size is invalid.
    pub fn new(width: u32, height: u32, channels: u32, stride: usize, data: &'a [f32]) -> Result<Self> {
        validate(width, height, channels, stride, data.len())?;
        Ok(Self {
            width,
            height,
            channels,
            stride,
            format: FrameFormat::F32,
            data,
        })
    }

    /// Views a caller-owned byte buffer as an F32 frame (bytemuck typed
    /// view). The buffer must be 4-byte aligned and at least
    /// `stride * height` bytes long.
    ///
    /// # Errors
    /// Returns [`CoreError::InvalidParameter`] if the buffer is misaligned
    /// or any dimension is invalid.
    pub fn from_bytes(width: u32, height: u32, channels: u32, stride: usize, bytes: &'a [u8]) -> Result<Self> {
        if !(bytes.as_ptr() as usize).is_multiple_of(FrameFormat::F32.element_size()) {
            return Err(CoreError::InvalidParameter(
                "byte buffer is not 4-byte aligned".to_string(),
            ));
        }
        let data = bytemuck::cast_slice(bytes);
        Self::new(width, height, channels, stride, data)
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
    pub fn channels(&self) -> u32 {
        self.channels
    }

    /// Stride in bytes per row.
    #[inline]
    pub fn stride(&self) -> usize {
        self.stride
    }

    #[inline]
    pub fn format(&self) -> FrameFormat {
        self.format
    }

    /// `(width, height, channels)`.
    #[inline]
    pub fn dims(&self) -> (u32, u32, u32) {
        (self.width, self.height, self.channels)
    }

    /// Number of valid pixels (`width * height * channels`).
    #[inline]
    pub fn pixel_count(&self) -> usize {
        self.width as usize * self.height as usize * self.channels as usize
    }

    /// Number of elements in the underlying buffer (`stride * height / element_size`).
    #[inline]
    pub fn num_elements(&self) -> usize {
        self.stride * self.height as usize / self.format.element_size()
    }

    /// Reads the pixel at `(x, y)` in channel `c`.
    ///
    /// Panics if the coordinates are out of range (caller invariant; the
    /// FFI validates dimensions before views are created).
    #[inline]
    pub fn pixel(&self, x: u32, y: u32, c: u32) -> f32 {
        debug_assert!((x < self.width) && (y < self.height) && (c < self.channels));
        let row_stride = self.stride / self.format.element_size();
        self.data[(y as usize) * row_stride + (x as usize) * (self.channels as usize) + (c as usize)]
    }

    /// The valid pixels of row `y` (`width * channels` elements).
    ///
    /// Panics if `y` is out of range.
    #[inline]
    pub fn row(&self, y: u32) -> &[f32] {
        debug_assert!(y < self.height);
        let start = y as usize * (self.stride / self.format.element_size());
        let len = (self.width as usize) * (self.channels as usize);
        &self.data[start..start + len]
    }

    /// The whole underlying buffer (including stride padding).
    #[inline]
    pub fn as_slice(&self) -> &[f32] {
        self.data
    }
}

/// Mutable, zero-copy view of a caller-owned frame buffer.
///
/// Created from a `&mut [f32]` (or a byte buffer via [`Frame::from_bytes`]).
/// The caller owns the buffer; the view never frees it.
#[derive(Debug)]
pub struct Frame<'a> {
    width: u32,
    height: u32,
    channels: u32,
    stride: usize,
    format: FrameFormat,
    data: &'a mut [f32],
}

impl<'a> Frame<'a> {
    /// Creates a validated frame view over `data`.
    ///
    /// `stride` is in bytes per row and must be a multiple of the element
    /// size, at least `width * channels * element_size`; `data` must hold at
    /// least `stride * height / element_size` elements.
    ///
    /// # Errors
    /// Returns [`CoreError::InvalidParameter`] if any dimension or buffer
    /// size is invalid.
    pub fn new(width: u32, height: u32, channels: u32, stride: usize, data: &'a mut [f32]) -> Result<Self> {
        validate(width, height, channels, stride, data.len())?;
        Ok(Self {
            width,
            height,
            channels,
            stride,
            format: FrameFormat::F32,
            data,
        })
    }

    /// Views a caller-owned byte buffer as a mutable F32 frame (bytemuck
    /// typed view). The buffer must be 4-byte aligned and at least
    /// `stride * height` bytes long.
    ///
    /// # Errors
    /// Returns [`CoreError::InvalidParameter`] if the buffer is misaligned
    /// or any dimension is invalid.
    pub fn from_bytes(width: u32, height: u32, channels: u32, stride: usize, bytes: &'a mut [u8]) -> Result<Self> {
        if !(bytes.as_ptr() as usize).is_multiple_of(FrameFormat::F32.element_size()) {
            return Err(CoreError::InvalidParameter(
                "byte buffer is not 4-byte aligned".to_string(),
            ));
        }
        let data = bytemuck::cast_slice_mut(bytes);
        Self::new(width, height, channels, stride, data)
    }

    /// Borrows the frame as an immutable view (valid for this borrow).
    #[inline]
    pub fn as_ref(&self) -> FrameRef<'_> {
        FrameRef {
            width: self.width,
            height: self.height,
            channels: self.channels,
            stride: self.stride,
            format: self.format,
            data: self.data,
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
    pub fn channels(&self) -> u32 {
        self.channels
    }

    /// Stride in bytes per row.
    #[inline]
    pub fn stride(&self) -> usize {
        self.stride
    }

    #[inline]
    pub fn format(&self) -> FrameFormat {
        self.format
    }

    /// `(width, height, channels)`.
    #[inline]
    pub fn dims(&self) -> (u32, u32, u32) {
        (self.width, self.height, self.channels)
    }

    /// Number of valid pixels (`width * height * channels`).
    #[inline]
    pub fn pixel_count(&self) -> usize {
        self.width as usize * self.height as usize * self.channels as usize
    }

    /// Number of elements in the underlying buffer (`stride * height / element_size`).
    #[inline]
    pub fn num_elements(&self) -> usize {
        self.stride * self.height as usize / self.format.element_size()
    }

    /// Reads the pixel at `(x, y)` in channel `c`.
    ///
    /// Panics if the coordinates are out of range.
    #[inline]
    pub fn pixel(&self, x: u32, y: u32, c: u32) -> f32 {
        debug_assert!((x < self.width) && (y < self.height) && (c < self.channels));
        let row_stride = self.stride / self.format.element_size();
        self.data[(y as usize) * row_stride + (x as usize) * (self.channels as usize) + (c as usize)]
    }

    /// Mutable access to the pixel at `(x, y)` in channel `c`.
    ///
    /// Panics if the coordinates are out of range.
    #[inline]
    pub fn pixel_mut(&mut self, x: u32, y: u32, c: u32) -> &mut f32 {
        debug_assert!((x < self.width) && (y < self.height) && (c < self.channels));
        let row_stride = self.stride / self.format.element_size();
        &mut self.data[(y as usize) * row_stride + (x as usize) * (self.channels as usize) + (c as usize)]
    }

    /// The valid pixels of row `y` (`width * channels` elements).
    ///
    /// Panics if `y` is out of range.
    #[inline]
    pub fn row(&self, y: u32) -> &[f32] {
        debug_assert!(y < self.height);
        let start = y as usize * (self.stride / self.format.element_size());
        let len = (self.width as usize) * (self.channels as usize);
        &self.data[start..start + len]
    }

    /// Mutable access to the valid pixels of row `y`.
    ///
    /// Panics if `y` is out of range.
    #[inline]
    pub fn row_mut(&mut self, y: u32) -> &mut [f32] {
        debug_assert!(y < self.height);
        let start = y as usize * (self.stride / self.format.element_size());
        let len = (self.width as usize) * (self.channels as usize);
        &mut self.data[start..start + len]
    }

    /// The whole underlying buffer (including stride padding).
    #[inline]
    pub fn as_slice(&self) -> &[f32] {
        self.data
    }

    /// The whole underlying buffer (including stride padding), mutably.
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [f32] {
        self.data
    }

    /// Copies `source` into this frame.
    ///
    /// # Errors
    /// Returns [`CoreError::BufferDimensionMismatch`] if the frames have
    /// different `(width, height, channels)`.
    pub fn copy_from(&mut self, source: &FrameRef) -> Result<()> {
        if self.dims() != source.dims() {
            return Err(CoreError::BufferDimensionMismatch {
                expected: source.pixel_count(),
                actual: self.pixel_count(),
            });
        }
        for y in 0..self.height {
            self.row_mut(y).copy_from_slice(source.row(y));
        }
        Ok(())
    }
}

/// Owned, compact frame (no stride padding) used for temporal model state
/// (e.g. EWMA's previous frame).
#[derive(Debug, Clone)]
pub struct OwnedFrame {
    width: u32,
    height: u32,
    channels: u32,
    data: Vec<f32>,
}

impl OwnedFrame {
    /// Makes a compact copy of a frame view (stride padding dropped).
    pub fn from_frame(frame: &FrameRef) -> Self {
        let mut data = Vec::with_capacity(frame.pixel_count());
        for y in 0..frame.height() {
            data.extend_from_slice(frame.row(y));
        }
        Self {
            width: frame.width(),
            height: frame.height(),
            channels: frame.channels(),
            data,
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
    pub fn channels(&self) -> u32 {
        self.channels
    }

    /// `(width, height, channels)`.
    #[inline]
    pub fn dims(&self) -> (u32, u32, u32) {
        (self.width, self.height, self.channels)
    }

    /// Number of pixels (`width * height * channels`).
    #[inline]
    pub fn pixel_count(&self) -> usize {
        self.width as usize * self.height as usize * self.channels as usize
    }

    /// Reads the pixel at `(x, y)` in channel `c`.
    ///
    /// Panics if the coordinates are out of range.
    #[inline]
    pub fn pixel(&self, x: u32, y: u32, c: u32) -> f32 {
        debug_assert!((x < self.width) && (y < self.height) && (c < self.channels));
        self.data[(y as usize) * (self.width as usize) * (self.channels as usize)
            + (x as usize) * (self.channels as usize)
            + (c as usize)]
    }

    /// Mutable access to the pixel at `(x, y)` in channel `c`.
    ///
    /// Panics if the coordinates are out of range.
    #[inline]
    pub fn pixel_mut(&mut self, x: u32, y: u32, c: u32) -> &mut f32 {
        debug_assert!((x < self.width) && (y < self.height) && (c < self.channels));
        &mut self.data[(y as usize) * (self.width as usize) * (self.channels as usize)
            + (x as usize) * (self.channels as usize)
            + (c as usize)]
    }

    /// The whole compact buffer.
    #[inline]
    pub fn as_slice(&self) -> &[f32] {
        &self.data
    }
}

/// Abstraction for zero-copy native frame handles across platforms
/// (e.g. AHardwareBuffer on Android, CVPixelBuffer on iOS).
///
/// The caller owns the memory behind `data_ptr` for the lifetime of the
/// handle; the handle never frees it. The pixel format is caller-defined;
/// `stride` is in bytes and must be at least `width` (one byte per pixel
/// column).
///
/// This is the platform-side wrapper that will *produce* [`Frame`]/[`FrameRef`]
/// views for the FFI once native buffers are wired in (P1.1).
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

    const W: u32 = 4;
    const H: u32 = 4;
    const C: u32 = 3;
    const STRIDE: usize = (W * C) as usize * 4; // 48 bytes

    fn make_input() -> Vec<f32> {
        let mut data = vec![0.0f32; (W * H * C) as usize];
        for y in 0..H {
            for x in 0..W {
                for c in 0..C {
                    data[(y * W * C + x * C + c) as usize] = (x as f32) + (y as f32) * 10.0 + (c as f32) * 100.0;
                }
            }
        }
        data
    }

    #[test]
    fn test_frame_ref_valid() {
        let data = make_input();
        let frame = FrameRef::new(W, H, C, STRIDE, &data).unwrap();
        assert_eq!(frame.width(), W);
        assert_eq!(frame.height(), H);
        assert_eq!(frame.channels(), C);
        assert_eq!(frame.stride(), STRIDE);
        assert_eq!(frame.pixel_count(), (W * H * C) as usize);
        assert_eq!(frame.num_elements(), (W * H * C) as usize);
        assert_eq!(frame.pixel(1, 2, 0), 1.0 + 20.0);
        assert_eq!(frame.row(0).len(), (W * C) as usize);
    }

    #[test]
    fn test_frame_valid_and_pixel_mut() {
        let mut data = vec![0.0f32; (W * H * C) as usize];
        let mut frame = Frame::new(W, H, C, STRIDE, &mut data).unwrap();
        *frame.pixel_mut(0, 0, 0) = 42.0;
        assert_eq!(frame.pixel(0, 0, 0), 42.0);
        assert_eq!(frame.as_ref().pixel(0, 0, 0), 42.0);
    }

    #[test]
    fn test_frame_from_bytes() {
        let mut bytes = vec![0u8; (W * H * C) as usize * 4];
        let mut frame = Frame::from_bytes(W, H, C, STRIDE, &mut bytes).unwrap();
        assert_eq!(frame.pixel_count(), (W * H * C) as usize);
        *frame.pixel_mut(2, 1, 2) = 7.5;
        assert_eq!(frame.pixel(2, 1, 2), 7.5);
    }

    #[test]
    fn test_frame_ref_from_bytes() {
        let bytes = vec![0u8; (W * H * C) as usize * 4];
        let frame = FrameRef::from_bytes(W, H, C, STRIDE, &bytes).unwrap();
        assert_eq!(frame.pixel(0, 0, 0), 0.0);
    }

    #[test]
    fn test_frame_rejects_zero_dims() {
        let data = vec![0.0f32; 16];
        let err = FrameRef::new(0, H, C, STRIDE, &data).unwrap_err();
        assert!(matches!(err, CoreError::InvalidParameter(_)));
    }

    #[test]
    fn test_frame_rejects_stride_not_multiple_of_element() {
        let data = vec![0.0f32; 16];
        let err = FrameRef::new(W, H, C, STRIDE + 2, &data).unwrap_err();
        assert!(matches!(err, CoreError::InvalidParameter(_)));
    }

    #[test]
    fn test_frame_rejects_small_stride() {
        let data = vec![0.0f32; 16];
        let err = FrameRef::new(W, H, C, 12, &data).unwrap_err();
        assert!(matches!(err, CoreError::InvalidParameter(_)));
    }

    #[test]
    fn test_frame_rejects_small_buffer() {
        let mut data = vec![0.0f32; 10];
        let err = Frame::new(W, H, C, STRIDE, &mut data).unwrap_err();
        assert!(matches!(err, CoreError::InvalidParameter(_)));
    }

    #[test]
    fn test_frame_stride_padding() {
        // Stride larger than the minimum: padding must be skipped.
        let stride = (W * C) as usize * 4 + 8;
        let mut data = vec![99.0f32; stride as usize / 4 * H as usize];
        let mut frame = Frame::new(W, H, C, stride, &mut data).unwrap();
        *frame.pixel_mut(0, 0, 0) = 1.0;
        // Only valid pixels are touched; padding stays untouched.
        for y in 0..H {
            for i in (W * C) as usize..stride / 4 {
                assert_eq!(data[y as usize * stride / 4 + i], 99.0);
            }
        }
    }

    #[test]
    fn test_copy_from() {
        let src_data = make_input();
        let src = FrameRef::new(W, H, C, STRIDE, &src_data).unwrap();
        let mut dst_data = vec![0.0f32; (W * H * C) as usize];
        let mut dst = Frame::new(W, H, C, STRIDE, &mut dst_data).unwrap();
        dst.copy_from(&src).unwrap();
        assert_eq!(dst.as_slice(), &src_data);
    }

    #[test]
    fn test_copy_from_dim_mismatch() {
        let src_data = make_input();
        let src = FrameRef::new(W, H, C, STRIDE, &src_data).unwrap();
        // Destination has 4 channels: valid stride for 4 channels is 64 bytes.
        let mut dst_data = vec![0.0f32; (W * H * (C + 1)) as usize];
        let mut dst = Frame::new(W, H, C + 1, (W * (C + 1)) as usize * 4, &mut dst_data).unwrap();
        let err = dst.copy_from(&src).unwrap_err();
        assert!(matches!(err, CoreError::BufferDimensionMismatch { .. }));
    }

    #[test]
    fn test_owned_frame() {
        let data = make_input();
        let frame = FrameRef::new(W, H, C, STRIDE, &data).unwrap();
        let mut owned = OwnedFrame::from_frame(&frame);
        assert_eq!(owned.pixel_count(), (W * H * C) as usize);
        assert_eq!(owned.pixel(1, 2, 0), 1.0 + 20.0);
        *owned.pixel_mut(0, 0, 0) = -1.0;
        assert_eq!(owned.pixel(0, 0, 0), -1.0);
    }

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
