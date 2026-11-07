use core::ports::input::TrackingError;

/// Fast bilinear image preprocessor for downscaling raw camera RGB frames to MoveNet input tensors.
#[derive(Debug, Clone)]
pub struct Preprocessor {
    pub target_width: u32,
    pub target_height: u32,
}

impl Preprocessor {
    pub fn new(target_width: u32, target_height: u32) -> Self {
        Self {
            target_width,
            target_height,
        }
    }

    #[inline]
    pub fn target_width(&self) -> u32 {
        self.target_width
    }

    #[inline]
    pub fn target_height(&self) -> u32 {
        self.target_height
    }

    #[inline]
    pub fn target_len(&self) -> usize {
        (self.target_width as usize)
            .saturating_mul(self.target_height as usize)
            .saturating_mul(3)
    }

    /// Downscales raw RGB bytes into a flat float32 array normalized to [0.0, 1.0].
    /// Output buffer length: target_width * target_height * 3 (HWC format).
    pub fn process_rgb8(
        &self,
        src_bytes: &[u8],
        src_width: u32,
        src_height: u32,
        mirror_horizontal: bool,
        out_tensor: &mut [f32],
    ) -> Result<(), TrackingError> {
        if self.target_width == 0 || self.target_height == 0 {
            return Err(TrackingError::InferenceFailure(
                "Invalid target dimensions: target width and height must be greater than 0"
                    .to_string(),
            ));
        }

        if src_width == 0 || src_height == 0 {
            return Err(TrackingError::CaptureFailure(
                "Invalid source dimensions: src width and height must be greater than 0"
                    .to_string(),
            ));
        }

        let expected_src_len = (src_width as usize)
            .checked_mul(src_height as usize)
            .and_then(|px| px.checked_mul(3))
            .ok_or_else(|| {
                TrackingError::CaptureFailure("Source image dimensions overflow".to_string())
            })?;

        if src_bytes.len() != expected_src_len {
            return Err(TrackingError::CaptureFailure(format!(
                "Source buffer length mismatch: expected {} bytes ({}x{}x3), got {}",
                expected_src_len,
                src_width,
                src_height,
                src_bytes.len()
            )));
        }

        let expected_dst_len = (self.target_width as usize)
            .checked_mul(self.target_height as usize)
            .and_then(|px| px.checked_mul(3))
            .ok_or_else(|| {
                TrackingError::InferenceFailure("Target tensor dimensions overflow".to_string())
            })?;

        if out_tensor.len() != expected_dst_len {
            return Err(TrackingError::InferenceFailure(format!(
                "Output tensor buffer length mismatch: expected {} floats, got {}",
                expected_dst_len,
                out_tensor.len()
            )));
        }

        let scale_x = src_width as f32 / self.target_width as f32;
        let scale_y = src_height as f32 / self.target_height as f32;
        let max_src_x = (src_width - 1) as f32;
        let max_src_y = (src_height - 1) as f32;

        let src_stride = (src_width as usize) * 3;
        let dst_width = self.target_width as usize;

        for ty in 0..self.target_height {
            let sy = ((ty as f32 + 0.5) * scale_y - 0.5).clamp(0.0, max_src_y);
            let y0 = sy.floor() as usize;
            let y1 = (y0 + 1).min(src_height as usize - 1);
            let dy = sy - y0 as f32;
            let inv_dy = 1.0 - dy;

            let row_offset_y0 = y0 * src_stride;
            let row_offset_y1 = y1 * src_stride;
            let dst_row_offset = (ty as usize) * dst_width * 3;

            for tx in 0..self.target_width {
                let sample_col = if mirror_horizontal {
                    self.target_width - 1 - tx
                } else {
                    tx
                };

                let sx = ((sample_col as f32 + 0.5) * scale_x - 0.5).clamp(0.0, max_src_x);
                let x0 = sx.floor() as usize;
                let x1 = (x0 + 1).min(src_width as usize - 1);
                let dx = sx - x0 as f32;
                let inv_dx = 1.0 - dx;

                let w00 = inv_dx * inv_dy;
                let w10 = dx * inv_dy;
                let w01 = inv_dx * dy;
                let w11 = dx * dy;

                let idx00 = row_offset_y0 + x0 * 3;
                let idx10 = row_offset_y0 + x1 * 3;
                let idx01 = row_offset_y1 + x0 * 3;
                let idx11 = row_offset_y1 + x1 * 3;

                let out_idx = dst_row_offset + (tx as usize) * 3;

                for c in 0..3 {
                    let p00 = src_bytes.get(idx00 + c).copied().unwrap_or(0) as f32;
                    let p10 = src_bytes.get(idx10 + c).copied().unwrap_or(0) as f32;
                    let p01 = src_bytes.get(idx01 + c).copied().unwrap_or(0) as f32;
                    let p11 = src_bytes.get(idx11 + c).copied().unwrap_or(0) as f32;

                    let interp = w00 * p00 + w10 * p10 + w01 * p01 + w11 * p11;
                    if let Some(slot) = out_tensor.get_mut(out_idx + c) {
                        *slot = (interp * (1.0 / 255.0)).clamp(0.0, 1.0);
                    }
                }
            }
        }

        Ok(())
    }

    /// Converts HWC flat normalized floats to CHW flat normalized floats.
    pub fn hwc_to_chw(&self, hwc: &[f32], chw: &mut [f32]) -> Result<(), TrackingError> {
        let hw = (self.target_width as usize) * (self.target_height as usize);
        let expected = hw * 3;
        if hwc.len() != expected || chw.len() != expected {
            return Err(TrackingError::InferenceFailure(format!(
                "hwc_to_chw buffer length mismatch: expected {}, got hwc={}, chw={}",
                expected,
                hwc.len(),
                chw.len()
            )));
        }

        for i in 0..hw {
            for c in 0..3 {
                if let (Some(val), Some(dest)) = (hwc.get(i * 3 + c), chw.get_mut(c * hw + i)) {
                    *dest = *val;
                }
            }
        }
        Ok(())
    }
}

impl Default for Preprocessor {
    fn default() -> Self {
        Self::new(192, 192)
    }
}
