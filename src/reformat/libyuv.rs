#![allow(dead_code, unused)] // TODO: remove

use super::rgb;
use super::rgb::*;

use crate::image;
use crate::image::*;
use crate::internal_utils::*;
use crate::reformat::bindings::libyuv::*;
use crate::*;

use std::os::raw::c_int;

fn find_constants(image: &image::Image) -> Option<(&YuvConstants, &YuvConstants)> {
    let matrix_coefficients = if image.yuv_format == PixelFormat::Monochrome
        && image.matrix_coefficients == MatrixCoefficients::Identity
    {
        MatrixCoefficients::Bt601
    } else {
        image.matrix_coefficients
    };
    unsafe {
        match image.full_range {
            true => match matrix_coefficients {
                MatrixCoefficients::Bt709 => Some((&kYuvF709Constants, &kYvuF709Constants)),
                MatrixCoefficients::Bt470bg
                | MatrixCoefficients::Bt601
                | MatrixCoefficients::Unspecified => Some((&kYuvJPEGConstants, &kYvuJPEGConstants)),
                MatrixCoefficients::Bt2020Ncl => Some((&kYuvV2020Constants, &kYvuV2020Constants)),
                MatrixCoefficients::ChromaDerivedNcl => match image.color_primaries {
                    ColorPrimaries::Srgb | ColorPrimaries::Unspecified => {
                        Some((&kYuvF709Constants, &kYvuF709Constants))
                    }
                    ColorPrimaries::Bt470bg | ColorPrimaries::Bt601 => {
                        Some((&kYuvJPEGConstants, &kYvuJPEGConstants))
                    }
                    ColorPrimaries::Bt2020 => Some((&kYuvV2020Constants, &kYvuV2020Constants)),
                    _ => None,
                },
                _ => None,
            },
            false => match matrix_coefficients {
                MatrixCoefficients::Bt709 => Some((&kYuvH709Constants, &kYvuH709Constants)),
                MatrixCoefficients::Bt470bg
                | MatrixCoefficients::Bt601
                | MatrixCoefficients::Unspecified => Some((&kYuvI601Constants, &kYvuI601Constants)),
                MatrixCoefficients::Bt2020Ncl => Some((&kYuv2020Constants, &kYvu2020Constants)),
                MatrixCoefficients::ChromaDerivedNcl => match image.color_primaries {
                    ColorPrimaries::Srgb | ColorPrimaries::Unspecified => {
                        Some((&kYuvH709Constants, &kYvuH709Constants))
                    }
                    ColorPrimaries::Bt470bg | ColorPrimaries::Bt601 => {
                        Some((&kYuvI601Constants, &kYvuI601Constants))
                    }
                    ColorPrimaries::Bt2020 => Some((&kYuv2020Constants, &kYvu2020Constants)),
                    _ => None,
                },
                _ => None,
            },
        }
    }
}

#[rustfmt::skip]
type YUV400ToRGBMatrix = unsafe extern "C" fn(
    *const u8, c_int, *mut u8, c_int, *const YuvConstants, c_int, c_int) -> c_int;
#[rustfmt::skip]
type YUVToRGBMatrixFilter = unsafe extern "C" fn(
    *const u8, c_int, *const u8, c_int, *const u8, c_int, *mut u8, c_int, *const YuvConstants,
    c_int, c_int, FilterMode) -> c_int;
#[rustfmt::skip]
type YUVAToRGBMatrixFilter = unsafe extern "C" fn(
    *const u8, c_int, *const u8, c_int, *const u8, c_int, *const u8, c_int, *mut u8, c_int,
    *const YuvConstants, c_int, c_int, c_int, FilterMode) -> c_int;
#[rustfmt::skip]
type YUVToRGBMatrix = unsafe extern "C" fn(
    *const u8, c_int, *const u8, c_int, *const u8, c_int, *mut u8, c_int, *const YuvConstants,
    c_int, c_int) -> c_int;
#[rustfmt::skip]
type YUVAToRGBMatrix = unsafe extern "C" fn(
    *const u8, c_int, *const u8, c_int, *const u8, c_int, *const u8, c_int, *mut u8, c_int,
    *const YuvConstants, c_int, c_int, c_int) -> c_int;
#[rustfmt::skip]
type YUVToRGBMatrixFilterHighBitDepth = unsafe extern "C" fn(
    *const u16, c_int, *const u16, c_int, *const u16, c_int, *mut u8, c_int, *const YuvConstants,
    c_int, c_int, FilterMode) -> c_int;
#[rustfmt::skip]
type YUVAToRGBMatrixFilterHighBitDepth = unsafe extern "C" fn(
    *const u16, c_int, *const u16, c_int, *const u16, c_int, *const u16, c_int, *mut u8, c_int,
    *const YuvConstants, c_int, c_int, c_int, FilterMode) -> c_int;
#[rustfmt::skip]
type YUVToRGBMatrixHighBitDepth = unsafe extern "C" fn(
    *const u16, c_int, *const u16, c_int, *const u16, c_int, *mut u8, c_int, *const YuvConstants,
    c_int, c_int) -> c_int;
#[rustfmt::skip]
type YUVAToRGBMatrixHighBitDepth = unsafe extern "C" fn(
    *const u16, c_int, *const u16, c_int, *const u16, c_int, *const u16, c_int, *mut u8, c_int,
    *const YuvConstants, c_int, c_int, c_int) -> c_int;

#[derive(Debug)]
enum ConversionFunction {
    YUV400ToRGBMatrix(YUV400ToRGBMatrix),
    YUVToRGBMatrixFilter(YUVToRGBMatrixFilter),
    YUVAToRGBMatrixFilter(YUVAToRGBMatrixFilter),
    YUVToRGBMatrix(YUVToRGBMatrix),
    YUVAToRGBMatrix(YUVAToRGBMatrix),
    YUVToRGBMatrixFilterHighBitDepth(YUVToRGBMatrixFilterHighBitDepth),
    YUVAToRGBMatrixFilterHighBitDepth(YUVAToRGBMatrixFilterHighBitDepth),
    YUVToRGBMatrixHighBitDepth(YUVToRGBMatrixHighBitDepth),
    YUVAToRGBMatrixHighBitDepth(YUVAToRGBMatrixHighBitDepth),
}

impl ConversionFunction {
    fn is_yuva(&self) -> bool {
        match self {
            ConversionFunction::YUVAToRGBMatrixFilter(_)
            | ConversionFunction::YUVAToRGBMatrix(_)
            | ConversionFunction::YUVAToRGBMatrixFilterHighBitDepth(_)
            | ConversionFunction::YUVAToRGBMatrixHighBitDepth(_) => true,
            _ => false,
        }
    }
}

fn find_conversion_function(
    yuv_format: PixelFormat,
    yuv_depth: u8,
    rgb: &rgb::Image,
    alpha_preferred: bool,
) -> Option<ConversionFunction> {
    if yuv_depth > 8 {
        // implement
    }
    if yuv_format == PixelFormat::Monochrome {
        return match rgb.format {
            Format::Rgba | Format::Bgra => {
                Some(ConversionFunction::YUV400ToRGBMatrix(I400ToARGBMatrix))
            }
            _ => None,
        };
    }
    if yuv_format != PixelFormat::Yuv444 {
        if alpha_preferred {
            match rgb.format {
                Format::Rgba | Format::Bgra => match yuv_format {
                    PixelFormat::Yuv422 => {
                        return Some(ConversionFunction::YUVAToRGBMatrixFilter(
                            I422AlphaToARGBMatrixFilter,
                        ))
                    }
                    PixelFormat::Yuv420 => {
                        return Some(ConversionFunction::YUVAToRGBMatrixFilter(
                            I420AlphaToARGBMatrixFilter,
                        ))
                    }
                    _ => {}
                },
                _ => {}
            }
        }
        match rgb.format {
            Format::Rgb | Format::Bgr => match yuv_format {
                PixelFormat::Yuv422 => {
                    return Some(ConversionFunction::YUVToRGBMatrixFilter(
                        I422ToRGB24MatrixFilter,
                    ))
                }
                PixelFormat::Yuv420 => {
                    return Some(ConversionFunction::YUVToRGBMatrixFilter(
                        I420ToRGB24MatrixFilter,
                    ))
                }
                _ => {}
            },
            Format::Rgba | Format::Bgra => match yuv_format {
                PixelFormat::Yuv422 => {
                    return Some(ConversionFunction::YUVToRGBMatrixFilter(
                        I422ToARGBMatrixFilter,
                    ))
                }
                PixelFormat::Yuv420 => {
                    return Some(ConversionFunction::YUVToRGBMatrixFilter(
                        I420ToARGBMatrixFilter,
                    ))
                }
                _ => {}
            },
            _ => {}
        }
        match rgb.chroma_upsampling {
            ChromaUpsampling::Bilinear | ChromaUpsampling::BestQuality => return None,
            _ => {}
        }
    }
    if alpha_preferred {
        match rgb.format {
            Format::Rgba | Format::Bgra => match yuv_format {
                PixelFormat::Yuv444 => {
                    return Some(ConversionFunction::YUVAToRGBMatrix(I444AlphaToARGBMatrix))
                }
                PixelFormat::Yuv422 => {
                    return Some(ConversionFunction::YUVAToRGBMatrix(I422AlphaToARGBMatrix))
                }
                PixelFormat::Yuv420 => {
                    return Some(ConversionFunction::YUVAToRGBMatrix(I420AlphaToARGBMatrix))
                }
                _ => {}
            },
            _ => {}
        }
    }
    match rgb.format {
        Format::Rgb | Format::Bgr => match yuv_format {
            PixelFormat::Yuv444 => {
                return Some(ConversionFunction::YUVToRGBMatrix(I444ToRGB24Matrix))
            }
            PixelFormat::Yuv420 => {
                return Some(ConversionFunction::YUVToRGBMatrix(I420ToRGB24Matrix))
            }
            _ => {}
        },
        Format::Rgba | Format::Bgra => match yuv_format {
            PixelFormat::Yuv444 => {
                return Some(ConversionFunction::YUVToRGBMatrix(I444ToARGBMatrix))
            }
            PixelFormat::Yuv422 => {
                return Some(ConversionFunction::YUVToRGBMatrix(I422ToARGBMatrix))
            }
            PixelFormat::Yuv420 => {
                return Some(ConversionFunction::YUVToRGBMatrix(I420ToARGBMatrix))
            }
            _ => {}
        },
        Format::Argb | Format::Abgr => match yuv_format {
            PixelFormat::Yuv422 => {
                return Some(ConversionFunction::YUVToRGBMatrix(I422ToRGBAMatrix))
            }
            PixelFormat::Yuv420 => {
                return Some(ConversionFunction::YUVToRGBMatrix(I420ToRGBAMatrix))
            }
            _ => {}
        },
        Format::Rgb565 => match yuv_format {
            PixelFormat::Yuv422 => {
                return Some(ConversionFunction::YUVToRGBMatrix(I422ToRGB565Matrix))
            }
            PixelFormat::Yuv420 => {
                return Some(ConversionFunction::YUVToRGBMatrix(I420ToRGB565Matrix))
            }
            _ => {}
        },
    }
    None
}

pub fn yuv_to_rgb(image: &image::Image, rgb: &rgb::Image, reformat_alpha: bool) -> AvifResult<()> {
    if rgb.depth != 8 || (image.depth != 8 && image.depth != 10 && image.depth != 12) {
        return Err(AvifError::NotImplemented);
    }
    let (matrix_yuv, matrix_yvu) = find_constants(image).ok_or(AvifError::NotImplemented)?;
    let alpha_preferred = reformat_alpha
        && image.planes[3].is_some()
        && !image.planes[3].unwrap().is_null()
        && image.row_bytes[3] > 0;
    let conversion_function =
        find_conversion_function(image.yuv_format, image.depth, rgb, alpha_preferred)
            .ok_or(AvifError::NotImplemented)?;
    let is_yvu = match rgb.format {
        Format::Rgb | Format::Rgba | Format::Argb => true,
        _ => false,
    };
    let matrix = if is_yvu { matrix_yvu } else { matrix_yuv };
    let u_plane_index: usize = if is_yvu { 2 } else { 1 };
    let v_plane_index: usize = if is_yvu { 1 } else { 2 };
    let filter = match rgb.chroma_upsampling {
        ChromaUpsampling::Fastest | ChromaUpsampling::Nearest => FilterMode_kFilterNone,
        _ => FilterMode_kFilterBilinear,
    };
    let mut pd: [Option<PlaneData>; 4] = [
        image.plane(Plane::Y),
        image.plane(Plane::U),
        image.plane(Plane::V),
        image.plane(Plane::A),
    ];
    let mut plane_u8: [*const u8; 4] = pd
        .iter()
        .map(|x| {
            if x.is_some() {
                x.as_ref().unwrap().data.as_ptr()
            } else {
                std::ptr::null()
            }
        })
        .collect::<Vec<*const u8>>()
        .try_into()
        .unwrap();
    let mut plane_row_bytes: [i32; 4] = pd
        .iter()
        .map(|x| {
            if x.is_some() {
                i32_from_u32(x.as_ref().unwrap().row_bytes).unwrap_or_default()
            } else {
                0
            }
        })
        .collect::<Vec<i32>>()
        .try_into()
        .unwrap();
    let rgb_row_bytes = i32_from_u32(rgb.row_bytes)?;
    let width = i32_from_u32(image.width)?;
    let height = i32_from_u32(image.height)?;
    let mut result: c_int = -1;
    unsafe {
        let mut high_bd_matched = true;
        // Apply one of the high bitdepth functions if possible.
        result = match conversion_function {
            ConversionFunction::YUVToRGBMatrixFilterHighBitDepth(func) => func(
                plane_u8[0] as *const u16,
                plane_row_bytes[0] / 2,
                plane_u8[u_plane_index] as *const u16,
                plane_row_bytes[u_plane_index] / 2,
                plane_u8[v_plane_index] as *const u16,
                plane_row_bytes[v_plane_index] / 2,
                rgb.pixels,
                rgb_row_bytes,
                matrix,
                width,
                height,
                filter,
            ),
            ConversionFunction::YUVAToRGBMatrixFilterHighBitDepth(func) => func(
                plane_u8[0] as *const u16,
                plane_row_bytes[0] / 2,
                plane_u8[u_plane_index] as *const u16,
                plane_row_bytes[u_plane_index] / 2,
                plane_u8[v_plane_index] as *const u16,
                plane_row_bytes[v_plane_index] / 2,
                plane_u8[3] as *const u16,
                plane_row_bytes[3] / 2,
                rgb.pixels,
                rgb_row_bytes,
                matrix,
                width,
                height,
                0, // attenuate
                filter,
            ),
            ConversionFunction::YUVToRGBMatrixHighBitDepth(func) => func(
                plane_u8[0] as *const u16,
                plane_row_bytes[0] / 2,
                plane_u8[u_plane_index] as *const u16,
                plane_row_bytes[u_plane_index] / 2,
                plane_u8[v_plane_index] as *const u16,
                plane_row_bytes[v_plane_index] / 2,
                rgb.pixels,
                rgb_row_bytes,
                matrix,
                width,
                height,
            ),
            ConversionFunction::YUVAToRGBMatrixHighBitDepth(func) => func(
                plane_u8[0] as *const u16,
                plane_row_bytes[0] / 2,
                plane_u8[u_plane_index] as *const u16,
                plane_row_bytes[u_plane_index] / 2,
                plane_u8[v_plane_index] as *const u16,
                plane_row_bytes[v_plane_index] / 2,
                plane_u8[3] as *const u16,
                plane_row_bytes[3] / 2,
                rgb.pixels,
                rgb_row_bytes,
                matrix,
                width,
                height,
                0, // attenuate
            ),
            _ => {
                high_bd_matched = false;
                -1
            }
        };
        if high_bd_matched {
            return if result == 0 {
                Ok(())
            } else {
                Err(AvifError::ReformatFailed)
            };
        }
        let mut image8 = image::Image::default();
        if image.depth > 8 {
            downshift_to_8bit(&image, &mut image8, conversion_function.is_yuva())?;
            pd = [
                image8.plane(Plane::Y),
                image8.plane(Plane::U),
                image8.plane(Plane::V),
                image8.plane(Plane::A),
            ];
            plane_u8 = pd
                .iter()
                .map(|x| {
                    if x.is_some() {
                        x.as_ref().unwrap().data.as_ptr()
                    } else {
                        std::ptr::null()
                    }
                })
                .collect::<Vec<*const u8>>()
                .try_into()
                .unwrap();
            plane_row_bytes = pd
                .iter()
                .map(|x| {
                    if x.is_some() {
                        i32_from_u32(x.as_ref().unwrap().row_bytes).unwrap_or_default()
                    } else {
                        0
                    }
                })
                .collect::<Vec<i32>>()
                .try_into()
                .unwrap();
        }
        result = match conversion_function {
            ConversionFunction::YUV400ToRGBMatrix(func) => func(
                plane_u8[0],
                plane_row_bytes[0],
                rgb.pixels,
                rgb_row_bytes,
                matrix,
                width,
                height,
            ),
            ConversionFunction::YUVToRGBMatrixFilter(func) => func(
                plane_u8[0],
                plane_row_bytes[0],
                plane_u8[u_plane_index],
                plane_row_bytes[u_plane_index],
                plane_u8[v_plane_index],
                plane_row_bytes[v_plane_index],
                rgb.pixels,
                rgb_row_bytes,
                matrix,
                width,
                height,
                filter,
            ),
            ConversionFunction::YUVAToRGBMatrixFilter(func) => func(
                plane_u8[0],
                plane_row_bytes[0],
                plane_u8[u_plane_index],
                plane_row_bytes[u_plane_index],
                plane_u8[v_plane_index],
                plane_row_bytes[v_plane_index],
                plane_u8[3],
                plane_row_bytes[3],
                rgb.pixels,
                rgb_row_bytes,
                matrix,
                width,
                height,
                0, // attenuate
                filter,
            ),
            ConversionFunction::YUVToRGBMatrix(func) => func(
                plane_u8[0],
                plane_row_bytes[0],
                plane_u8[u_plane_index],
                plane_row_bytes[u_plane_index],
                plane_u8[v_plane_index],
                plane_row_bytes[v_plane_index],
                rgb.pixels,
                rgb_row_bytes,
                matrix,
                width,
                height,
            ),
            ConversionFunction::YUVAToRGBMatrix(func) => func(
                plane_u8[0],
                plane_row_bytes[0],
                plane_u8[u_plane_index],
                plane_row_bytes[u_plane_index],
                plane_u8[v_plane_index],
                plane_row_bytes[v_plane_index],
                plane_u8[3],
                plane_row_bytes[3],
                rgb.pixels,
                rgb_row_bytes,
                matrix,
                width,
                height,
                0, // attenuate
            ),
            _ => 0,
        };
    }
    if result == 0 {
        Ok(())
    } else {
        Err(AvifError::ReformatFailed)
    }
}

fn downshift_to_8bit(
    image: &image::Image,
    image8: &mut image::Image,
    alpha: bool,
) -> AvifResult<()> {
    image8.width = image.width;
    image8.height = image.height;
    image8.depth = image.depth;
    image8.yuv_format = image.yuv_format;
    image8.allocate_planes(0)?;
    if alpha {
        image8.allocate_planes(1)?;
    }
    let scale = (1 << (24 - image.depth)) as i32;
    for plane in ALL_PLANES {
        let pd = image.plane(plane);
        if pd.is_none() {
            continue;
        }
        let pd = pd.unwrap();
        if pd.width == 0 {
            continue;
        }
        let mut pd8 = image8.plane(plane).unwrap();
        unsafe {
            Convert16To8Plane(
                pd.data.as_ptr() as *const u16,
                i32_from_u32(pd.row_bytes / 2)?,
                pd8.data.as_ptr() as *mut u8,
                i32_from_u32(pd8.row_bytes)?,
                scale,
                i32_from_u32(pd.width)?,
                i32_from_u32(pd.height)?,
            );
        }
    }
    Ok(())
}