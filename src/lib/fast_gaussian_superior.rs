// Copyright (c) Radzivon Bartoshyk. All rights reserved.
//
// Redistribution and use in source and binary forms, with or without modification,
// are permitted provided that the following conditions are met:
//
// 1.  Redistributions of source code must retain the above copyright notice, this
// list of conditions and the following disclaimer.
//
// 2.  Redistributions in binary form must reproduce the above copyright notice,
// this list of conditions and the following disclaimer in the documentation
// and/or other materials provided with the distribution.
//
// 3.  Neither the name of the copyright holder nor the names of its
// contributors may be used to endorse or promote products derived from
// this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
// AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
// IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
// DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
// FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
// DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
// SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
// CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
// OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use crate::{FastBlurChannels, ThreadingPolicy};

mod fast_gaussian_superior {
    use num_traits::{FromPrimitive, ToPrimitive};

    use crate::unsafe_slice::UnsafeSlice;
    use crate::ThreadingPolicy;

    fn fast_gaussian_vertical_pass<
        T: FromPrimitive + ToPrimitive + Default + Into<i64> + Send + Sync,
        const CHANNELS_COUNT: usize,
    >(
        bytes: &UnsafeSlice<T>,
        stride: u32,
        width: u32,
        height: u32,
        radius: u32,
        start: u32,
        end: u32,
    ) where
        T: std::ops::AddAssign + std::ops::SubAssign + Copy,
    {
        let mut buffer_r: [i64; 2048] = [0; 2048];
        let mut buffer_g: [i64; 2048] = [0; 2048];
        let mut buffer_b: [i64; 2048] = [0; 2048];
        let mut buffer_a: [i64; 2048] = [0; 2048];
        let radius_64 = radius as i64;
        let height_wide = height as i64;
        let radius_2d = (radius as f64) * (radius as f64);
        let weight = 1.0f64 / (radius_2d * radius_2d);
        for x in start..std::cmp::min(width, end) {
            let mut dif_r: i64 = 0;
            let mut der_1_r: i64 = 0;
            let mut der_2_r: i64 = 0;
            let mut sum_r: i64 = 0;
            let mut dif_g: i64 = 0;
            let mut der_1_g: i64 = 0;
            let mut der_2_g: i64 = 0;
            let mut sum_g: i64 = 0;
            let mut dif_b: i64 = 0;
            let mut der_1_b: i64 = 0;
            let mut der_2_b: i64 = 0;
            let mut sum_b: i64 = 0;
            let mut dif_a: i64 = 0;
            let mut der_1_a: i64 = 0;
            let mut der_2_a: i64 = 0;
            let mut sum_a: i64 = 0;

            let current_px = x as usize * CHANNELS_COUNT;

            let start_y = 0i64 - 4i64 * radius as i64;
            for y in start_y..height_wide {
                let current_y = (y * (stride as i64)) as usize;
                if y >= 0 {
                    let new_r = T::from_u32(((sum_r as f64) * weight) as u32).unwrap_or_default();
                    let new_g = T::from_u32(((sum_g as f64) * weight) as u32).unwrap_or_default();
                    let new_b = T::from_u32(((sum_b as f64) * weight) as u32).unwrap_or_default();

                    let bytes_offset = current_y + current_px;

                    unsafe {
                        bytes.write(bytes_offset, new_r);
                        bytes.write(bytes_offset + 1, new_g);
                        bytes.write(bytes_offset + 2, new_b);
                        if CHANNELS_COUNT == 4 {
                            let new_a =
                                T::from_u32(((sum_a as f64) * weight) as u32).unwrap_or_default();
                            bytes.write(bytes_offset + 3, new_a);
                        }
                    }

                    let arr_index_3 = (y & 2047) as usize;
                    let arr_index_2 = ((y + radius_64) & 2047) as usize;
                    let arr_index_1 = ((y - radius_64) & 2047) as usize;
                    let arr_index_4 = ((y - 2 * radius_64) & 2047) as usize;

                    unsafe {
                        dif_r += -4
                            * ((*buffer_r.get_unchecked(arr_index_1))
                                + (*buffer_r.get_unchecked(arr_index_2)))
                            + 6 * (*buffer_r.get_unchecked(arr_index_3))
                            + (*buffer_r.get_unchecked(arr_index_4));
                        dif_g += -4
                            * ((*buffer_g.get_unchecked(arr_index_1))
                                + (*buffer_g.get_unchecked(arr_index_2)))
                            + 6 * (*buffer_g.get_unchecked(arr_index_3))
                            + (*buffer_g.get_unchecked(arr_index_4));
                        dif_b += -4
                            * ((*buffer_b.get_unchecked(arr_index_1))
                                + (*buffer_b.get_unchecked(arr_index_2)))
                            + 6 * (*buffer_b.get_unchecked(arr_index_3))
                            + (*buffer_b.get_unchecked(arr_index_4));
                        if CHANNELS_COUNT == 4 {
                            dif_a += -4
                                * ((*buffer_a.get_unchecked(arr_index_1))
                                    + (*buffer_a.get_unchecked(arr_index_2)))
                                + 6 * (*buffer_a.get_unchecked(arr_index_3))
                                + (*buffer_a.get_unchecked(arr_index_4));
                        }
                    };
                } else {
                    if y + 3 * radius_64 >= 0 {
                        let arr_index = ((y + radius_64) & 2047) as usize;
                        dif_r -= 4 * unsafe { *buffer_r.get_unchecked(arr_index) };
                        dif_g -= 4 * unsafe { *buffer_g.get_unchecked(arr_index) };
                        dif_b -= 4 * unsafe { *buffer_b.get_unchecked(arr_index) };
                        if CHANNELS_COUNT == 4 {
                            dif_a -= 4 * unsafe { *buffer_a.get_unchecked(arr_index) };
                        }
                    }
                    if y + 2 * radius_64 >= 0 {
                        let arr_index = (y & 2047) as usize;
                        dif_r += 6 * unsafe { *buffer_r.get_unchecked(arr_index) };
                        dif_g += 6 * unsafe { *buffer_g.get_unchecked(arr_index) };
                        dif_b += 6 * unsafe { *buffer_b.get_unchecked(arr_index) };
                        if CHANNELS_COUNT == 4 {
                            dif_a += 6 * unsafe { *buffer_a.get_unchecked(arr_index) };
                        }
                    }
                    if y + radius_64 >= 0 {
                        let arr_index = ((y - radius_64) & 2047) as usize;
                        dif_r -= 4 * unsafe { *buffer_r.get_unchecked(arr_index) };
                        dif_g -= 4 * unsafe { *buffer_g.get_unchecked(arr_index) };
                        dif_b -= 4 * unsafe { *buffer_b.get_unchecked(arr_index) };
                        if CHANNELS_COUNT == 4 {
                            dif_a -= 4 * unsafe { *buffer_a.get_unchecked(arr_index) };
                        }
                    }
                }

                let next_row_y =
                    (std::cmp::min(std::cmp::max(y + 2 * radius_64 - 1, 0), height_wide - 1)
                        as usize)
                        * (stride as usize);
                let next_row_x = x as usize * CHANNELS_COUNT;

                let px_idx = next_row_y + next_row_x;

                let ur8 = bytes[px_idx];
                let ug8 = bytes[px_idx + 1];
                let ub8 = bytes[px_idx + 2];

                let arr_index = ((y + 2 * radius_64) & 2047) as usize;

                dif_r += ur8.into();
                der_2_r += dif_r;
                der_1_r += der_2_r;
                sum_r += der_1_r;
                unsafe {
                    *buffer_r.get_unchecked_mut(arr_index) = ur8.into();
                }

                dif_g += ug8.into();
                der_2_g += dif_g;
                der_1_g += der_2_g;
                sum_g += der_1_g;
                unsafe {
                    *buffer_g.get_unchecked_mut(arr_index) = ug8.into();
                }

                dif_b += ub8.into();
                der_2_b += dif_b;
                der_1_b += der_2_b;
                sum_b += der_1_b;
                unsafe {
                    *buffer_b.get_unchecked_mut(arr_index) = ub8.into();
                }

                if CHANNELS_COUNT == 4 {
                    let ua8 = bytes[px_idx + 3];
                    dif_a += ua8.into();
                    der_2_a += dif_a;
                    der_1_a += der_2_a;
                    sum_a += der_1_a;
                    unsafe {
                        *buffer_a.get_unchecked_mut(arr_index) = ua8.into();
                    }
                }
            }
        }
    }

    fn fast_gaussian_horizontal_pass<
        T: FromPrimitive + ToPrimitive + Default + Into<i64> + Send + Sync,
        const CHANNELS_COUNT: usize,
    >(
        bytes: &UnsafeSlice<T>,
        stride: u32,
        width: u32,
        height: u32,
        radius: u32,
        start: u32,
        end: u32,
    ) where
        T: std::ops::AddAssign + std::ops::SubAssign + Copy,
    {
        let mut buffer_r: [i64; 2048] = [0; 2048];
        let mut buffer_g: [i64; 2048] = [0; 2048];
        let mut buffer_b: [i64; 2048] = [0; 2048];
        let mut buffer_a: [i64; 2048] = [0; 2048];
        let radius_64 = radius as i64;
        let width_wide = width as i64;
        let radius_2d = (radius as f64) * (radius as f64);
        let weight = 1.0f64 / (radius_2d * radius_2d);
        for y in start..std::cmp::min(height, end) {
            let mut dif_r: i64 = 0;
            let mut der_1_r: i64 = 0;
            let mut der_2_r: i64 = 0;
            let mut sum_r: i64 = 0;
            let mut dif_g: i64 = 0;
            let mut der_1_g: i64 = 0;
            let mut der_2_g: i64 = 0;
            let mut sum_g: i64 = 0;
            let mut dif_b: i64 = 0;
            let mut der_1_b: i64 = 0;
            let mut der_2_b: i64 = 0;
            let mut sum_b: i64 = 0;
            let mut dif_a: i64 = 0;
            let mut der_1_a: i64 = 0;
            let mut der_2_a: i64 = 0;
            let mut sum_a: i64 = 0;

            let current_y = ((y as i64) * (stride as i64)) as usize;

            for x in (0i64 - 4i64 * radius_64)..(width as i64) {
                if x >= 0 {
                    let current_px = (std::cmp::max(x, 0) as u32) as usize * CHANNELS_COUNT;
                    let new_r = T::from_u32(((sum_r as f64) * weight) as u32).unwrap_or_default();
                    let new_g = T::from_u32(((sum_g as f64) * weight) as u32).unwrap_or_default();
                    let new_b = T::from_u32(((sum_b as f64) * weight) as u32).unwrap_or_default();

                    let bytes_offset = current_y + current_px;

                    unsafe {
                        bytes.write(bytes_offset, new_r);
                        bytes.write(bytes_offset + 1, new_g);
                        bytes.write(bytes_offset + 2, new_b);
                        if CHANNELS_COUNT == 4 {
                            let new_a =
                                T::from_u32(((sum_a as f64) * weight) as u32).unwrap_or_default();
                            bytes.write(bytes_offset + 3, new_a);
                        }
                    }

                    let arr_index_3 = (x & 2047) as usize;
                    let arr_index_2 = ((x + radius_64) & 2047) as usize;
                    let arr_index_1 = ((x - radius_64) & 2047) as usize;
                    let arr_index_4 = ((x - 2 * radius_64) & 2047) as usize;

                    unsafe {
                        dif_r += -4
                            * ((*buffer_r.get_unchecked(arr_index_1))
                                + (*buffer_r.get_unchecked(arr_index_2)))
                            + 6 * (*buffer_r.get_unchecked(arr_index_3))
                            + (*buffer_r.get_unchecked(arr_index_4));
                        dif_g += -4
                            * ((*buffer_g.get_unchecked(arr_index_1))
                                + (*buffer_g.get_unchecked(arr_index_2)))
                            + 6 * (*buffer_g.get_unchecked(arr_index_3))
                            + (*buffer_g.get_unchecked(arr_index_4));
                        dif_b += -4
                            * ((*buffer_b.get_unchecked(arr_index_1))
                                + (*buffer_b.get_unchecked(arr_index_2)))
                            + 6 * (*buffer_b.get_unchecked(arr_index_3))
                            + (*buffer_b.get_unchecked(arr_index_4));
                        if CHANNELS_COUNT == 4 {
                            dif_a += -4
                                * ((*buffer_a.get_unchecked(arr_index_1))
                                    + (*buffer_a.get_unchecked(arr_index_2)))
                                + 6 * (*buffer_a.get_unchecked(arr_index_3))
                                + (*buffer_a.get_unchecked(arr_index_4));
                        }
                    }
                } else {
                    if x + 3 * radius_64 >= 0 {
                        let arr_index = ((x + radius_64) & 2047) as usize;
                        dif_r -= 4 * unsafe { *buffer_r.get_unchecked(arr_index) };
                        dif_g -= 4 * unsafe { *buffer_g.get_unchecked(arr_index) };
                        dif_b -= 4 * unsafe { *buffer_b.get_unchecked(arr_index) };
                        if CHANNELS_COUNT == 4 {
                            dif_a -= 4 * unsafe { *buffer_a.get_unchecked(arr_index) };
                        }
                    }
                    if x + 2 * radius_64 >= 0 {
                        let arr_index = (x & 2047) as usize;
                        dif_r += 6 * unsafe { *buffer_r.get_unchecked(arr_index) };
                        dif_g += 6 * unsafe { *buffer_g.get_unchecked(arr_index) };
                        dif_b += 6 * unsafe { *buffer_b.get_unchecked(arr_index) };
                        if CHANNELS_COUNT == 4 {
                            dif_a += 6 * unsafe { *buffer_a.get_unchecked(arr_index) };
                        }
                    }
                    if x + radius_64 >= 0 {
                        let arr_index = ((x - radius_64) & 2047) as usize;
                        dif_r -= 4 * unsafe { *buffer_r.get_unchecked(arr_index) };
                        dif_g -= 4 * unsafe { *buffer_g.get_unchecked(arr_index) };
                        dif_b -= 4 * unsafe { *buffer_b.get_unchecked(arr_index) };
                        if CHANNELS_COUNT == 4 {
                            dif_a -= 4 * unsafe { *buffer_a.get_unchecked(arr_index) };
                        }
                    }
                }

                let next_row_y = (y as usize) * (stride as usize);
                let next_row_x =
                    (std::cmp::min(std::cmp::max(x + 2 * radius_64 - 1, 0), width_wide - 1) as u32)
                        as usize
                        * CHANNELS_COUNT;

                let bytes_offset = next_row_y + next_row_x;

                let ur8 = bytes[bytes_offset];
                let ug8 = bytes[bytes_offset + 1];
                let ub8 = bytes[bytes_offset + 2];

                let arr_index = ((x + 2 * radius_64) & 2047) as usize;

                dif_r += ur8.into();
                der_2_r += dif_r;
                der_1_r += der_2_r;
                sum_r += der_1_r;
                unsafe {
                    *buffer_r.get_unchecked_mut(arr_index) = ur8.into();
                }

                dif_g += ug8.into();
                der_2_g += dif_g;
                der_1_g += der_2_g;
                sum_g += der_1_g;
                unsafe {
                    *buffer_g.get_unchecked_mut(arr_index) = ug8.into();
                }

                dif_b += ub8.into();
                der_2_b += dif_b;
                der_1_b += der_2_b;
                sum_b += der_1_b;
                unsafe {
                    *buffer_b.get_unchecked_mut(arr_index) = ub8.into();
                }

                if CHANNELS_COUNT == 4 {
                    let ua8 = bytes[bytes_offset + 3];
                    dif_a += ua8.into();
                    der_2_a += dif_a;
                    der_1_a += der_2_a;
                    sum_a += der_1_a;
                    unsafe {
                        *buffer_a.get_unchecked_mut(arr_index) = ua8.into();
                    }
                }
            }
        }
    }

    pub(crate) fn fast_gaussian_impl<
        T: FromPrimitive + ToPrimitive + Default + Into<i64> + Send + Sync,
        const CHANNELS_COUNT: usize,
    >(
        bytes: &mut [T],
        stride: u32,
        width: u32,
        height: u32,
        radius: u32,
        threading_policy: ThreadingPolicy,
    ) where
        T: std::ops::AddAssign + std::ops::SubAssign + Copy,
    {
        let unsafe_image = UnsafeSlice::new(bytes);
        let thread_count = threading_policy.get_threads_count(width, height) as u32;
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(thread_count as usize)
            .build()
            .unwrap();
        pool.scope(|scope| {
            let segment_size = width / thread_count;

            for i in 0..thread_count {
                let start_x = i * segment_size;
                let mut end_x = (i + 1) * segment_size;
                if i == thread_count - 1 {
                    end_x = width;
                }
                scope.spawn(move |_| {
                    fast_gaussian_vertical_pass::<T, CHANNELS_COUNT>(
                        &unsafe_image,
                        stride,
                        width,
                        height,
                        radius,
                        start_x,
                        end_x,
                    );
                });
            }
        });
        pool.scope(|scope| {
            let segment_size = height / thread_count;

            for i in 0..thread_count {
                let start_y = i * segment_size;
                let mut end_y = (i + 1) * segment_size;
                if i == thread_count - 1 {
                    end_y = height;
                }
                scope.spawn(move |_| {
                    fast_gaussian_horizontal_pass::<T, CHANNELS_COUNT>(
                        &unsafe_image,
                        stride,
                        width,
                        height,
                        radius,
                        start_y,
                        end_y,
                    );
                });
            }
        });
    }
}

/// Fast gaussian approximation. This is almost gaussian blur. Significantly slower than alternatives.
/// # Arguments
///
/// * `stride` - Lane length, default is width * channels_count if not aligned
/// * `radius` - Radius more than ~256 is not supported.
/// O(1) complexity.
pub fn fast_gaussian_superior(
    bytes: &mut [u8],
    stride: u32,
    width: u32,
    height: u32,
    radius: u32,
    channels: FastBlurChannels,
    threading_policy: ThreadingPolicy,
) {
    let acq_radius = std::cmp::min(radius, 256);
    match channels {
        FastBlurChannels::Channels3 => {
            fast_gaussian_superior::fast_gaussian_impl::<u8, 3>(
                bytes,
                stride,
                width,
                height,
                acq_radius,
                threading_policy,
            );
        }
        FastBlurChannels::Channels4 => {
            fast_gaussian_superior::fast_gaussian_impl::<u8, 4>(
                bytes,
                stride,
                width,
                height,
                acq_radius,
                threading_policy,
            );
        }
    }
}
