// Copyright 2024 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::image::*;
use crate::internal_utils::*;
use crate::*;

use libyuv_sys::bindings::*;

impl Image {
    pub(crate) fn scale(&mut self, width: u32, height: u32, category: Category) -> AvifResult<()> {
        if self.width == width && self.height == height {
            return Ok(());
        }
        if width == 0 || height == 0 {
            return Err(AvifError::InvalidArgument);
        }
        let planes = category.planes();
        let src =
            if category != Category::Alpha && self.yuv_format == PixelFormat::AndroidP010 {
                // P010 images cannot be scaled using ScalePlane_12 since the U and V planes are
                // interleaved. Convert them into I010 and then scale each plane using
                // ScalePlane_12.
                let mut i010 = image::Image {
                    width: self.width,
                    height: self.height,
                    depth: 10,
                    yuv_format: PixelFormat::Yuv420,
                    ..image::Image::default()
                };
                i010.allocate_planes(Category::Color)?;
                let src_y_pd = self.plane_data(Plane::Y).unwrap();
                let src_uv_pd = self.plane_data(Plane::U).unwrap();
                let src_y = self.planes[Plane::Y.as_usize()].unwrap_ref().ptr16();
                let src_uv = self.planes[Plane::U.as_usize()].unwrap_ref().ptr16();
                let dst_y_pd = i010.plane_data(Plane::Y).unwrap();
                let dst_u_pd = i010.plane_data(Plane::U).unwrap();
                let dst_v_pd = i010.plane_data(Plane::V).unwrap();
                let dst_y = i010.planes[Plane::Y.as_usize()].unwrap_mut().ptr16_mut();
                let dst_u = i010.planes[Plane::U.as_usize()].unwrap_mut().ptr16_mut();
                let dst_v = i010.planes[Plane::V.as_usize()].unwrap_mut().ptr16_mut();
                // SAFETY: This function calls into libyuv which is a C++ library. We pass in
                // pointers and strides to rust slices that are guaranteed to be valid.
                let ret = unsafe {
                    P010ToI010(
                        src_y,
                        i32_from_u32(src_y_pd.row_bytes / 2)?,
                        src_uv,
                        i32_from_u32(src_uv_pd.row_bytes / 2)?,
                        dst_y,
                        i32_from_u32(dst_y_pd.row_bytes / 2)?,
                        dst_u,
                        i32_from_u32(dst_u_pd.row_bytes / 2)?,
                        dst_v,
                        i32_from_u32(dst_v_pd.row_bytes / 2)?,
                        i32_from_u32(self.width)?,
                        i32_from_u32(self.height)?,
                    )
                };
                if ret != 0 {
                    return Err(AvifError::ReformatFailed);
                }
                i010
            } else {
                image::Image {
                    width: self.width,
                    height: self.height,
                    depth: self.depth,
                    yuv_format: self.yuv_format,
                    planes: self
                        .planes
                        .as_ref()
                        .iter()
                        .map(|plane| {
                            if plane.is_some() {
                                plane.unwrap_ref().try_clone().ok()
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>()
                        .try_into()
                        .unwrap(),
                    row_bytes: self.row_bytes,
                    ..image::Image::default()
                }
            };

        self.width = width;
        self.height = height;
        self.depth = src.depth;
        self.yuv_format = src.yuv_format;
        if src.has_plane(Plane::Y) || src.has_plane(Plane::A) {
            if src.width > 16384 || src.height > 16384 {
                return Err(AvifError::NotImplemented);
            }
            if src.has_plane(Plane::Y) && category != Category::Alpha {
                self.allocate_planes(Category::Color)?;
            }
            if src.has_plane(Plane::A) && category == Category::Alpha {
                self.allocate_planes(Category::Alpha)?;
            }
        }

        if category != Category::Alpha
            && (self.yuv_format == PixelFormat::AndroidNv12
                || self.yuv_format == PixelFormat::AndroidNv21)
        {
            let src_y_pd = src.plane_data(Plane::Y).unwrap();
            let src_uv_pd = src.plane_data(Plane::U).unwrap();
            let src_y = src.planes[Plane::Y.as_usize()].unwrap_ref().ptr();
            let src_uv = src.planes[Plane::U.as_usize()].unwrap_ref().ptr();
            let dst_y_pd = self.plane_data(Plane::Y).unwrap();
            let dst_uv_pd = self.plane_data(Plane::U).unwrap();
            let dst_y = self.planes[Plane::Y.as_usize()].unwrap_mut().ptr_mut();
            let dst_uv = self.planes[Plane::U.as_usize()].unwrap_mut().ptr_mut();
            // SAFETY: This function calls into libyuv which is a C++ library. We pass in pointers
            // and strides to rust slices that are guaranteed to be valid.
            let ret = unsafe {
                NV12Scale(
                    src_y,
                    i32_from_u32(src_y_pd.row_bytes)?,
                    src_uv,
                    i32_from_u32(src_uv_pd.row_bytes)?,
                    i32_from_u32(src_y_pd.width)?,
                    i32_from_u32(src_y_pd.height)?,
                    dst_y,
                    i32_from_u32(dst_y_pd.row_bytes)?,
                    dst_uv,
                    i32_from_u32(dst_uv_pd.row_bytes)?,
                    i32_from_u32(dst_y_pd.width)?,
                    i32_from_u32(dst_y_pd.height)?,
                    FilterMode_kFilterBox,
                )
            };
            if ret != 0 {
                return Err(AvifError::ReformatFailed);
            } else {
                return Ok(());
            }
        }

        for plane in planes {
            if !src.has_plane(*plane) || !self.has_plane(*plane) {
                continue;
            }
            let src_pd = src.plane_data(*plane).unwrap();
            let dst_pd = self.plane_data(*plane).unwrap();
            // SAFETY: This function calls into libyuv which is a C++ library. We pass in pointers
            // and strides to rust slices that are guaranteed to be valid.
            //
            // libyuv versions >= 1880 reports a return value here. Older versions do not. Ignore
            // the return value for now.
            #[allow(clippy::let_unit_value)]
            let _ret = unsafe {
                if src.depth > 8 {
                    let source_ptr = src.planes[plane.as_usize()].unwrap_ref().ptr16();
                    let dst_ptr = self.planes[plane.as_usize()].unwrap_mut().ptr16_mut();
                    ScalePlane_12(
                        source_ptr,
                        i32_from_u32(src_pd.row_bytes / 2)?,
                        i32_from_u32(src_pd.width)?,
                        i32_from_u32(src_pd.height)?,
                        dst_ptr,
                        i32_from_u32(dst_pd.row_bytes / 2)?,
                        i32_from_u32(dst_pd.width)?,
                        i32_from_u32(dst_pd.height)?,
                        FilterMode_kFilterBox,
                    )
                } else {
                    let source_ptr = src.planes[plane.as_usize()].unwrap_ref().ptr();
                    let dst_ptr = self.planes[plane.as_usize()].unwrap_mut().ptr_mut();
                    ScalePlane(
                        source_ptr,
                        i32_from_u32(src_pd.row_bytes)?,
                        i32_from_u32(src_pd.width)?,
                        i32_from_u32(src_pd.height)?,
                        dst_ptr,
                        i32_from_u32(dst_pd.row_bytes)?,
                        i32_from_u32(dst_pd.width)?,
                        i32_from_u32(dst_pd.height)?,
                        FilterMode_kFilterBox,
                    )
                }
            };
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::pixels::*;
    use test_case::test_matrix;

    #[test_matrix([PixelFormat::Yuv444, PixelFormat::Yuv422, PixelFormat::Yuv420, PixelFormat::Yuv400], [false, true], [false, true])]
    fn scale(yuv_format: PixelFormat, use_alpha: bool, is_pointer_input: bool) {
        let mut yuv = image::Image {
            width: 2,
            height: 2,
            depth: 8,
            yuv_format,
            ..Default::default()
        };

        let planes: &[Plane] = match (yuv_format, use_alpha) {
            (PixelFormat::Yuv400, false) => &[Plane::Y],
            (PixelFormat::Yuv400, true) => &[Plane::Y, Plane::A],
            (_, false) => &YUV_PLANES,
            (_, true) => &ALL_PLANES,
        };
        let mut values = [
            10, 20, //
            30, 40,
        ];
        for plane in planes {
            yuv.planes[plane.as_usize()] = Some(if is_pointer_input {
                Pixels::Pointer(unsafe {
                    PointerSlice::create(values.as_mut_ptr(), values.len()).unwrap()
                })
            } else {
                Pixels::Buffer(values.to_vec())
            });
            yuv.row_bytes[plane.as_usize()] = 2;
            yuv.image_owns_planes[plane.as_usize()] = !is_pointer_input;
        }
        let categories: &[Category] =
            if use_alpha { &[Category::Color, Category::Alpha] } else { &[Category::Color] };
        for category in categories {
            // Scale will update the width and height when scaling YUV planes. Reset it back before
            // calling it again.
            yuv.width = 2;
            yuv.height = 2;
            assert!(yuv.scale(4, 4, *category).is_ok());
        }
        for plane in planes {
            let expected_samples: &[u8] = match (yuv_format, plane) {
                (PixelFormat::Yuv422, Plane::U | Plane::V) => &[
                    10, 10, //
                    10, 10, //
                    30, 30, //
                    30, 30,
                ],
                (PixelFormat::Yuv420, Plane::U | Plane::V) => &[
                    10, 10, //
                    10, 10,
                ],
                (_, _) => &[
                    10, 13, 18, 20, //
                    15, 18, 23, 25, //
                    25, 28, 33, 35, //
                    30, 33, 38, 40,
                ],
            };
            match &yuv.planes[plane.as_usize()] {
                Some(Pixels::Buffer(samples)) => {
                    assert_eq!(*samples, expected_samples)
                }
                _ => panic!(),
            }
        }
    }

    #[test]
    fn scale_nv12_odd_dimension() -> AvifResult<()> {
        let mut image = image::Image {
            width: 99,
            height: 49,
            depth: 8,
            yuv_format: PixelFormat::AndroidNv12,
            ..Default::default()
        };
        image.allocate_planes(Category::Color)?;
        assert!(image.scale(49, 24, Category::Color).is_ok());
        assert_eq!(image.width, 49);
        assert_eq!(image.height, 24);
        Ok(())
    }
}
