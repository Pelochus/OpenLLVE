//! First-class frame types: validated, zero-copy views over caller-owned buffers.
//! `stride` is in **bytes per row** and must be at least `width * channels * element_size`.
//!
//! - [`FrameRef`] immutable input view (C `const float*`)
//! - [`Frame`] mutable output view (C `float*`)
//! - [`OwnedFrame`] owned compact copy for temporal state (no stride padding)

use crate::error::{CoreError, Result};

/// Element format of a frame buffer.
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

/// Validate frame geometry without touching the buffer: non-zero dimensions,
/// a stride that is a whole number of elements, and a stride at least one row wide.
///
/// u64 arithmetic: dimensions whose products overflow must classify as errors,
/// never wrap into a short slice.
pub fn validate_dimensions(width: u32, height: u32, channels: u32, stride: usize) -> Result<()> {
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
    let min_stride = (width as u64) * (channels as u64) * (elem as u64);
    if (stride as u64) < min_stride {
        return Err(CoreError::InvalidParameter(format!(
            "stride {} bytes must be >= width*channels*element_size ({})",
            stride, min_stride
        )));
    }
    Ok(())
}

/// Elements a buffer must hold for these dimensions, [`None`] when the frame
/// is larger than any buffer could be (`stride * height` must not wrap).
pub fn buffer_elements(height: u32, stride: usize) -> Option<u64> {
    let elem = FrameFormat::F32.element_size();
    (stride as u64).checked_mul(height as u64).map(|it| it / (elem as u64))
}

/// Validate geometry plus the host-reported buffer length (in elements).
pub fn validate(width: u32, height: u32, channels: u32, stride: usize, data_len: usize) -> Result<()> {
    validate_dimensions(width, height, channels, stride)?;
    match buffer_elements(height, stride) {
        None => Err(CoreError::InvalidParameter(
            "stride * height describes a frame larger than any buffer could be".to_string(),
        )),
        Some(min_len) if (data_len as u64) < min_len => Err(CoreError::BufferDimensionMismatch {
            expected: min_len.try_into().unwrap_or(usize::MAX),
            actual: data_len,
        }),
        Some(_) => Ok(()),
    }
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
    /// Validated view over `data`.
    ///
    /// # Errors
    /// [`CoreError::InvalidParameter`] if a dimension, the stride, or `data` is
    /// too small for the frame contract.
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

    /// View a caller-owned byte buffer as an F32 frame (4-byte aligned, at least
    /// `stride * height` bytes).
    ///
    /// # Errors
    /// [`CoreError::InvalidParameter`] if misaligned or the contract fails.
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

    #[inline]
    pub fn stride(&self) -> usize {
        self.stride
    }

    #[inline]
    pub fn format(&self) -> FrameFormat {
        self.format
    }

    #[inline]
    pub fn dims(&self) -> (u32, u32, u32) {
        (self.width, self.height, self.channels)
    }

    #[inline]
    pub fn pixel_count(&self) -> usize {
        self.width as usize * self.height as usize * self.channels as usize
    }

    #[inline]
    pub fn num_elements(&self) -> usize {
        self.stride * self.height as usize / self.format.element_size()
    }

    #[inline]
    pub fn pixel(&self, x: u32, y: u32, c: u32) -> f32 {
        debug_assert!((x < self.width) && (y < self.height) && (c < self.channels));
        let row_stride = self.stride / self.format.element_size();
        self.data[(y as usize) * row_stride + (x as usize) * (self.channels as usize) + (c as usize)]
    }

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
    /// Validated view over `data`.
    ///
    /// # Errors
    /// [`CoreError::InvalidParameter`] if a dimension, the stride, or `data` is
    /// too small for the frame contract.
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

    /// View a caller-owned byte buffer as a mutable F32 frame (4-byte aligned, at least
    /// `stride * height` bytes).
    ///
    /// # Errors
    /// [`CoreError::InvalidParameter`] if misaligned or the contract fails.
    pub fn from_bytes(width: u32, height: u32, channels: u32, stride: usize, bytes: &'a mut [u8]) -> Result<Self> {
        if !(bytes.as_ptr() as usize).is_multiple_of(FrameFormat::F32.element_size()) {
            return Err(CoreError::InvalidParameter(
                "byte buffer is not 4-byte aligned".to_string(),
            ));
        }
        let data = bytemuck::cast_slice_mut(bytes);
        Self::new(width, height, channels, stride, data)
    }

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

    #[inline]
    pub fn stride(&self) -> usize {
        self.stride
    }

    #[inline]
    pub fn format(&self) -> FrameFormat {
        self.format
    }

    #[inline]
    pub fn dims(&self) -> (u32, u32, u32) {
        (self.width, self.height, self.channels)
    }

    #[inline]
    pub fn pixel_count(&self) -> usize {
        self.width as usize * self.height as usize * self.channels as usize
    }

    #[inline]
    pub fn num_elements(&self) -> usize {
        self.stride * self.height as usize / self.format.element_size()
    }

    #[inline]
    pub fn pixel(&self, x: u32, y: u32, c: u32) -> f32 {
        debug_assert!((x < self.width) && (y < self.height) && (c < self.channels));
        let row_stride = self.stride / self.format.element_size();
        self.data[(y as usize) * row_stride + (x as usize) * (self.channels as usize) + (c as usize)]
    }

    #[inline]
    pub fn pixel_mut(&mut self, x: u32, y: u32, c: u32) -> &mut f32 {
        debug_assert!((x < self.width) && (y < self.height) && (c < self.channels));
        let row_stride = self.stride / self.format.element_size();
        &mut self.data[(y as usize) * row_stride + (x as usize) * (self.channels as usize) + (c as usize)]
    }

    #[inline]
    pub fn row(&self, y: u32) -> &[f32] {
        debug_assert!(y < self.height);
        let start = y as usize * (self.stride / self.format.element_size());
        let len = (self.width as usize) * (self.channels as usize);
        &self.data[start..start + len]
    }

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

    #[inline]
    pub fn dims(&self) -> (u32, u32, u32) {
        (self.width, self.height, self.channels)
    }

    #[inline]
    pub fn pixel_count(&self) -> usize {
        self.width as usize * self.height as usize * self.channels as usize
    }

    #[inline]
    pub fn pixel(&self, x: u32, y: u32, c: u32) -> f32 {
        debug_assert!((x < self.width) && (y < self.height) && (c < self.channels));
        self.data[(y as usize) * (self.width as usize) * (self.channels as usize)
            + (x as usize) * (self.channels as usize)
            + (c as usize)]
    }

    #[inline]
    pub fn pixel_mut(&mut self, x: u32, y: u32, c: u32) -> &mut f32 {
        debug_assert!((x < self.width) && (y < self.height) && (c < self.channels));
        &mut self.data[(y as usize) * (self.width as usize) * (self.channels as usize)
            + (x as usize) * (self.channels as usize)
            + (c as usize)]
    }

    #[inline]
    pub fn as_slice(&self) -> &[f32] {
        &self.data
    }
}

/// Zero-copy handle over a platform-native pixel buffer (AHardwareBuffer,
/// CVPixelBuffer, ...). The caller owns `data_ptr` for the handle's lifetime;
/// the handle never frees it. `stride` is in bytes and must be at least `width`.
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
        assert!(matches!(err, CoreError::BufferDimensionMismatch { .. }));
    }

    #[test]
    fn test_stride_height_products_never_wrap() {
        // 2^62 * 8 does not fit u64: it is rejected, where the old `stride * height
        // / 4` wrapped to zero and sized an empty view of a frame claiming 2^60
        // elements.
        let stride = 1usize << 62;
        let empty: [f32; 0] = [];
        assert_eq!(buffer_elements(8, stride), None);
        let err = FrameRef::new(1, 8, 1, stride, &empty).unwrap_err();
        assert!(matches!(err, CoreError::InvalidParameter(_)));
        assert_eq!(buffer_elements(8, 16), Some(32u64));
    }

    #[test]
    fn test_dimensions_are_checked_before_any_buffer_exists() {
        assert!(validate_dimensions(0, 1, 1, 4).is_err());
        assert!(validate_dimensions(1, 1, 1, 3).is_err());
        assert!(validate_dimensions(4, 1, 1, 4).is_err());
        assert!(validate_dimensions(4, 1, 1, 16).is_ok());
    }

    #[test]
    fn test_frame_stride_padding() {
        let stride = (W * C) as usize * 4 + 8;
        let mut data = vec![99.0f32; stride as usize / 4 * H as usize];
        let mut frame = Frame::new(W, H, C, stride, &mut data).unwrap();
        *frame.pixel_mut(0, 0, 0) = 1.0;
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
