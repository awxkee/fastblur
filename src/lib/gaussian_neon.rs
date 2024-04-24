use crate::unsafe_slice::UnsafeSlice;
use std::arch::aarch64::{
    float32x4_t, uint8x8_t, vaddq_f32, vcombine_u16, vcvtq_f32_u32, vcvtq_u32_f32, vdupq_n_f32,
    vget_low_u16, vld1_u8, vld1q_f32, vmovl_u16, vmovl_u8, vmulq_f32, vqmovn_u16, vqmovn_u32,
    vst1_u8,
};
use std::ptr;

#[allow(dead_code)]
#[cfg(target_arch = "aarch64")]
pub fn gaussian_blur_horizontal_pass_impl_neon_3channels_u8(
    src: &Vec<u8>,
    src_stride: u32,
    unsafe_dst: &UnsafeSlice<u8>,
    dst_stride: u32,
    width: u32,
    kernel_size: usize,
    kernel: &Vec<f32>,
    start_y: u32,
    end_y: u32,
) {
    let half_kernel = (kernel_size / 2) as i32;
    let mut safe_transient_store: Vec<u8> = Vec::with_capacity(8);
    safe_transient_store.resize(8, 0);
    let eraser_store: [f32; 4] = [1f32, 1f32, 1f32, 0f32];

    let eraser: float32x4_t = unsafe { vld1q_f32(eraser_store.as_ptr()) };

    for y in start_y..end_y {
        let y_src_shift = y as usize * src_stride as usize;
        let y_dst_shift = y as usize * dst_stride as usize;
        for x in 0..width {
            let mut store: float32x4_t = unsafe { vdupq_n_f32(0f32) };
            for r in -half_kernel..=half_kernel {
                let source_ptr: *const u8;
                let px = std::cmp::min(std::cmp::max(x as i64 + r as i64, 0), (width - 1) as i64)
                    as usize
                    * 3;
                let s_ptr = unsafe { src.as_ptr().add(y_src_shift + px) };
                if x + 3 < width {
                    source_ptr = s_ptr;
                } else {
                    unsafe {
                        ptr::copy_nonoverlapping(s_ptr, safe_transient_store.as_mut_ptr(), 3);
                    }
                    source_ptr = safe_transient_store.as_ptr();
                }
                let pixel_colors: uint8x8_t = unsafe { vld1_u8(source_ptr) };
                let pixel_colors_u16 = unsafe { vmovl_u8(pixel_colors) };
                let pixel_colors_u32 = unsafe { vmovl_u16(vget_low_u16(pixel_colors_u16)) };
                let mut pixel_colors_f32 =
                    unsafe { vmulq_f32(vcvtq_f32_u32(pixel_colors_u32), eraser) };
                let weight = kernel[(r + half_kernel) as usize];
                let f_weight: float32x4_t = unsafe { vdupq_n_f32(weight) };
                pixel_colors_f32 = unsafe { vmulq_f32(pixel_colors_f32, f_weight) };
                store = unsafe { vaddq_f32(store, pixel_colors_f32) };
            }

            let px = x as usize * 3;

            let dst_ptr = unsafe { unsafe_dst.slice.as_ptr().add(y_dst_shift + px) as *mut u8 };
            let px_16 = unsafe { vqmovn_u32(vcvtq_u32_f32(store)) };
            let px_8 = unsafe { vqmovn_u16(vcombine_u16(px_16, px_16)) };
            if x + 3 < width {
                unsafe {
                    vst1_u8(dst_ptr, px_8);
                };
            } else {
                let px_8_full = unsafe { vqmovn_u16(vcombine_u16(px_16, px_16)) };
                unsafe {
                    vst1_u8(safe_transient_store.as_mut_ptr(), px_8_full);
                }
                unsafe {
                    unsafe_dst.write(y_dst_shift + px, safe_transient_store[0]);
                    unsafe_dst.write(y_dst_shift + px + 1, safe_transient_store[1]);
                    unsafe_dst.write(y_dst_shift + px + 2, safe_transient_store[2]);
                }
            }
        }
    }
}

#[allow(dead_code)]
#[cfg(target_arch = "aarch64")]
pub fn gaussian_blur_vertical_pass_impl_neon_3channels_u8(
    src: &Vec<u8>,
    src_stride: u32,
    unsafe_dst: &UnsafeSlice<u8>,
    dst_stride: u32,
    width: u32,
    height: u32,
    kernel_size: usize,
    kernel: &Vec<f32>,
    start_y: u32,
    end_y: u32,
) {
    let half_kernel = (kernel_size / 2) as i32;
    let mut safe_transient_store: Vec<u8> = Vec::with_capacity(8);
    safe_transient_store.resize(8, 0);
    let eraser_store: [f32; 4] = [1f32, 1f32, 1f32, 0f32];

    let eraser: float32x4_t = unsafe { vld1q_f32(eraser_store.as_ptr()) };

    for y in start_y..end_y {
        let y_dst_shift = y as usize * dst_stride as usize;
        for x in 0..width {
            let mut store: float32x4_t = unsafe { vdupq_n_f32(0f32) };
            let px = x as usize * 3;
            for r in -half_kernel..=half_kernel {
                let py = std::cmp::min(std::cmp::max(y as i64 + r as i64, 0), (height - 1) as i64);
                let y_src_shift = py as usize * src_stride as usize;

                let source_ptr: *const u8;
                let s_ptr = unsafe { src.as_ptr().add(y_src_shift + px) };
                if x + 3 < width {
                    source_ptr = s_ptr;
                } else {
                    unsafe {
                        ptr::copy_nonoverlapping(s_ptr, safe_transient_store.as_mut_ptr(), 3);
                    }
                    source_ptr = safe_transient_store.as_ptr();
                }
                let pixel_colors: uint8x8_t = unsafe { vld1_u8(source_ptr) };
                let pixel_colors_u16 = unsafe { vmovl_u8(pixel_colors) };
                let pixel_colors_u32 = unsafe { vmovl_u16(vget_low_u16(pixel_colors_u16)) };
                let mut pixel_colors_f32 =
                    unsafe { vmulq_f32(vcvtq_f32_u32(pixel_colors_u32), eraser) };
                let weight = kernel[(r + half_kernel) as usize];
                let f_weight: float32x4_t = unsafe { vdupq_n_f32(weight) };
                pixel_colors_f32 = unsafe { vmulq_f32(pixel_colors_f32, f_weight) };
                store = unsafe { vaddq_f32(store, pixel_colors_f32) };
            }


            let dst_ptr = unsafe { unsafe_dst.slice.as_ptr().add(y_dst_shift + px) as *mut u8 };
            let px_16 = unsafe { vqmovn_u32(vcvtq_u32_f32(store)) };
            let px_8 = unsafe { vqmovn_u16(vcombine_u16(px_16, px_16)) };
            if x + 3 < width {
                unsafe {
                    vst1_u8(dst_ptr, px_8);
                };
            } else {
                let px_8_full = unsafe { vqmovn_u16(vcombine_u16(px_16, px_16)) };
                unsafe {
                    vst1_u8(safe_transient_store.as_mut_ptr(), px_8_full);
                }
                unsafe {
                    unsafe_dst.write(y_dst_shift + px, safe_transient_store[0]);
                    unsafe_dst.write(y_dst_shift + px + 1, safe_transient_store[1]);
                    unsafe_dst.write(y_dst_shift + px + 2, safe_transient_store[2]);
                }
            }
        }
    }
}
