//! RGB to YUV420 conversion for ToxAV.
//!
//! ToxAV requires video frames in YUV420 planar format.
//! This module provides conversion from common camera formats.

/// Convert RGB24 buffer to YUV420 planar format.
///
/// Uses BT.601 conversion coefficients:
/// - Y =  0.299 * R + 0.587 * G + 0.114 * B
/// - U = -0.169 * R - 0.331 * G + 0.500 * B + 128
/// - V =  0.500 * R - 0.419 * G - 0.081 * B + 128
///
/// Returns (Y plane, U plane, V plane).
pub fn rgb_to_yuv420(rgb: &[u8], width: usize, height: usize) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    let y_size = width * height;
    let uv_width = width / 2;
    let uv_height = height / 2;
    let uv_size = uv_width * uv_height;

    let mut y_plane = vec![0u8; y_size];
    let mut u_plane = vec![0u8; uv_size];
    let mut v_plane = vec![0u8; uv_size];

    // First pass: calculate Y for every pixel
    for row in 0..height {
        for col in 0..width {
            let idx = (row * width + col) * 3;
            let r = rgb[idx] as f32;
            let g = rgb[idx + 1] as f32;
            let b = rgb[idx + 2] as f32;

            let y = (0.299 * r + 0.587 * g + 0.114 * b).clamp(0.0, 255.0) as u8;
            y_plane[row * width + col] = y;
        }
    }

    // Second pass: calculate U and V for 2x2 blocks (subsampled)
    for row in 0..uv_height {
        for col in 0..uv_width {
            // Sample from the top-left pixel of each 2x2 block
            let src_row = row * 2;
            let src_col = col * 2;
            let idx = (src_row * width + src_col) * 3;

            let r = rgb[idx] as f32;
            let g = rgb[idx + 1] as f32;
            let b = rgb[idx + 2] as f32;

            let u = (-0.169 * r - 0.331 * g + 0.500 * b + 128.0).clamp(0.0, 255.0) as u8;
            let v = (0.500 * r - 0.419 * g - 0.081 * b + 128.0).clamp(0.0, 255.0) as u8;

            u_plane[row * uv_width + col] = u;
            v_plane[row * uv_width + col] = v;
        }
    }

    (y_plane, u_plane, v_plane)
}

/// Convert RGBA32 buffer to YUV420 planar format.
///
/// Same as rgb_to_yuv420 but skips the alpha channel.
pub fn rgba_to_yuv420(rgba: &[u8], width: usize, height: usize) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    let y_size = width * height;
    let uv_width = width / 2;
    let uv_height = height / 2;
    let uv_size = uv_width * uv_height;

    let mut y_plane = vec![0u8; y_size];
    let mut u_plane = vec![0u8; uv_size];
    let mut v_plane = vec![0u8; uv_size];

    // First pass: calculate Y for every pixel
    for row in 0..height {
        for col in 0..width {
            let idx = (row * width + col) * 4;
            let r = rgba[idx] as f32;
            let g = rgba[idx + 1] as f32;
            let b = rgba[idx + 2] as f32;
            // Alpha at idx + 3 is ignored

            let y = (0.299 * r + 0.587 * g + 0.114 * b).clamp(0.0, 255.0) as u8;
            y_plane[row * width + col] = y;
        }
    }

    // Second pass: calculate U and V for 2x2 blocks (subsampled)
    for row in 0..uv_height {
        for col in 0..uv_width {
            let src_row = row * 2;
            let src_col = col * 2;
            let idx = (src_row * width + src_col) * 4;

            let r = rgba[idx] as f32;
            let g = rgba[idx + 1] as f32;
            let b = rgba[idx + 2] as f32;

            let u = (-0.169 * r - 0.331 * g + 0.500 * b + 128.0).clamp(0.0, 255.0) as u8;
            let v = (0.500 * r - 0.419 * g - 0.081 * b + 128.0).clamp(0.0, 255.0) as u8;

            u_plane[row * uv_width + col] = u;
            v_plane[row * uv_width + col] = v;
        }
    }

    (y_plane, u_plane, v_plane)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgb_to_yuv420_dimensions() {
        let width = 640;
        let height = 480;
        let rgb = vec![128u8; width * height * 3];

        let (y, u, v) = rgb_to_yuv420(&rgb, width, height);

        assert_eq!(y.len(), width * height);
        assert_eq!(u.len(), (width / 2) * (height / 2));
        assert_eq!(v.len(), (width / 2) * (height / 2));
    }

    #[test]
    fn test_white_to_yuv() {
        // White RGB (255, 255, 255) should give Y=255, U=128, V=128
        let rgb = vec![255u8; 4 * 4 * 3]; // 4x4 white image
        let (y, u, v) = rgb_to_yuv420(&rgb, 4, 4);

        assert!(y.iter().all(|&val| val == 255));
        assert!(u.iter().all(|&val| (val as i32 - 128).abs() <= 1));
        assert!(v.iter().all(|&val| (val as i32 - 128).abs() <= 1));
    }

    #[test]
    fn test_black_to_yuv() {
        // Black RGB (0, 0, 0) should give Y=0, U=128, V=128
        let rgb = vec![0u8; 4 * 4 * 3]; // 4x4 black image
        let (y, u, v) = rgb_to_yuv420(&rgb, 4, 4);

        assert!(y.iter().all(|&val| val == 0));
        assert!(u.iter().all(|&val| val == 128));
        assert!(v.iter().all(|&val| val == 128));
    }
}
