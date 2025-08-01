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

#![deny(unsafe_op_in_unsafe_fn)]
#![cfg_attr(feature = "disable_cfi", feature(no_sanitize))]

#[macro_use]
mod internal_utils;

pub mod decoder;
#[cfg(feature = "encoder")]
pub mod encoder;
pub mod gainmap;
pub mod image;
pub mod reformat;
pub mod utils;

#[cfg(feature = "capi")]
pub mod capi;

/// cbindgen:ignore
mod codecs;

mod parser;

use image::*;

// Workaround for https://bugs.chromium.org/p/chromium/issues/detail?id=1516634.
#[derive(Default)]
pub struct NonRandomHasherState;

impl std::hash::BuildHasher for NonRandomHasherState {
    type Hasher = std::collections::hash_map::DefaultHasher;
    fn build_hasher(&self) -> std::collections::hash_map::DefaultHasher {
        std::collections::hash_map::DefaultHasher::new()
    }
}

pub type HashMap<K, V> = std::collections::HashMap<K, V, NonRandomHasherState>;
pub type HashSet<K> = std::collections::HashSet<K, NonRandomHasherState>;

/// cbindgen:enum-trailing-values=[Count]
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
// See https://aomediacodec.github.io/av1-spec/#color-config-semantics.
pub enum PixelFormat {
    #[default]
    None = 0,
    Yuv444 = 1,
    Yuv422 = 2,
    Yuv420 = 3, // Also used for alpha items when 4:0:0 is not supported by the codec.
    Yuv400 = 4,
    // The following formats are not found in the AV1 spec. They are formats that are supported by
    // Android platform. They are intended to be pass-through formats that are used only by the
    // Android MediaCodec wrapper. All internal functions will treat them as opaque.
    AndroidP010 = 5,
    AndroidNv12 = 6,
    AndroidNv21 = 7,
}

impl PixelFormat {
    pub fn is_monochrome(&self) -> bool {
        *self == Self::Yuv400
    }

    pub fn plane_count(&self) -> usize {
        match self {
            PixelFormat::None
            | PixelFormat::AndroidP010
            | PixelFormat::AndroidNv12
            | PixelFormat::AndroidNv21 => 0,
            PixelFormat::Yuv400 => 1,
            PixelFormat::Yuv420 | PixelFormat::Yuv422 | PixelFormat::Yuv444 => 3,
        }
    }

    pub fn chroma_shift_x(&self) -> (u32, u32) {
        match self {
            Self::Yuv422 | Self::Yuv420 => (1, 0),
            Self::AndroidP010 | Self::AndroidNv12 => (1, 1),
            _ => (0, 0),
        }
    }

    pub fn apply_chroma_shift_x(&self, value: u32) -> u32 {
        let chroma_shift = self.chroma_shift_x();
        (value >> chroma_shift.0) << chroma_shift.1
    }

    pub fn chroma_shift_y(&self) -> u32 {
        match self {
            Self::Yuv420 | Self::AndroidP010 | Self::AndroidNv12 | Self::AndroidNv21 => 1,
            _ => 0,
        }
    }

    pub fn apply_chroma_shift_y(&self, value: u32) -> u32 {
        value >> self.chroma_shift_y()
    }
}

// See https://aomediacodec.github.io/av1-spec/#color-config-semantics
// and https://en.wikipedia.org/wiki/Chroma_subsampling#Sampling_positions.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum ChromaSamplePosition {
    #[default]
    Unknown = 0, // Corresponds to AV1's CSP_UNKNOWN.
    Vertical = 1,  // Corresponds to AV1's CSP_VERTICAL (MPEG-2, also called "left").
    Colocated = 2, // Corresponds to AV1's CSP_COLOCATED (BT.2020, also called "top-left").
    Reserved = 3,  // Corresponds to AV1's CSP_RESERVED.
}

impl ChromaSamplePosition {
    // The AV1 Specification (Version 1.0.0 with Errata 1) does not have a CSP_CENTER value
    // for chroma_sample_position, so we are forced to signal CSP_UNKNOWN in the AV1 bitstream
    // when the chroma sample position is CENTER.
    const CENTER: ChromaSamplePosition = ChromaSamplePosition::Unknown; // JPEG/"center"
}

impl From<u32> for ChromaSamplePosition {
    fn from(value: u32) -> Self {
        match value {
            0 => Self::Unknown,
            1 => Self::Vertical,
            2 => Self::Colocated,
            3 => Self::Reserved,
            _ => Self::Unknown,
        }
    }
}

// See https://aomediacodec.github.io/av1-spec/#color-config-semantics.
#[repr(u16)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum ColorPrimaries {
    Unknown = 0,
    Srgb = 1,
    #[default]
    Unspecified = 2,
    Bt470m = 4,
    Bt470bg = 5,
    Bt601 = 6,
    Smpte240 = 7,
    GenericFilm = 8,
    Bt2020 = 9,
    Xyz = 10,
    Smpte431 = 11,
    Smpte432 = 12,
    Ebu3213 = 22,
}

impl From<u16> for ColorPrimaries {
    fn from(value: u16) -> Self {
        match value {
            0 => Self::Unknown,
            1 => Self::Srgb,
            2 => Self::Unspecified,
            4 => Self::Bt470m,
            5 => Self::Bt470bg,
            6 => Self::Bt601,
            7 => Self::Smpte240,
            8 => Self::GenericFilm,
            9 => Self::Bt2020,
            10 => Self::Xyz,
            11 => Self::Smpte431,
            12 => Self::Smpte432,
            22 => Self::Ebu3213,
            _ => Self::default(),
        }
    }
}

#[allow(non_camel_case_types, non_upper_case_globals)]
impl ColorPrimaries {
    pub const Bt709: Self = Self::Srgb;
    pub const Iec61966_2_4: Self = Self::Srgb;
    pub const Bt2100: Self = Self::Bt2020;
    pub const Dci_p3: Self = Self::Smpte432;
}

// See https://aomediacodec.github.io/av1-spec/#color-config-semantics.
#[repr(u16)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum TransferCharacteristics {
    Unknown = 0,
    Bt709 = 1,
    #[default]
    Unspecified = 2,
    Reserved = 3,
    Bt470m = 4,  // 2.2 gamma
    Bt470bg = 5, // 2.8 gamma
    Bt601 = 6,
    Smpte240 = 7,
    Linear = 8,
    Log100 = 9,
    Log100Sqrt10 = 10,
    Iec61966 = 11,
    Bt1361 = 12,
    Srgb = 13,
    Bt2020_10bit = 14,
    Bt2020_12bit = 15,
    Pq = 16, // Perceptual Quantizer (HDR); BT.2100 PQ
    Smpte428 = 17,
    Hlg = 18, // Hybrid Log-Gamma (HDR); ARIB STD-B67; BT.2100 HLG
}

impl From<u16> for TransferCharacteristics {
    fn from(value: u16) -> Self {
        match value {
            0 => Self::Unknown,
            1 => Self::Bt709,
            2 => Self::Unspecified,
            3 => Self::Reserved,
            4 => Self::Bt470m,
            5 => Self::Bt470bg,
            6 => Self::Bt601,
            7 => Self::Smpte240,
            8 => Self::Linear,
            9 => Self::Log100,
            10 => Self::Log100Sqrt10,
            11 => Self::Iec61966,
            12 => Self::Bt1361,
            13 => Self::Srgb,
            14 => Self::Bt2020_10bit,
            15 => Self::Bt2020_12bit,
            16 => Self::Pq,
            17 => Self::Smpte428,
            18 => Self::Hlg,
            _ => Self::default(),
        }
    }
}

#[allow(non_upper_case_globals)]
impl TransferCharacteristics {
    pub const Smpte2084: Self = Self::Pq;
}

// See https://aomediacodec.github.io/av1-spec/#color-config-semantics.
#[repr(u16)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum MatrixCoefficients {
    Identity = 0,
    Bt709 = 1,
    #[default]
    Unspecified = 2,
    Reserved = 3,
    Fcc = 4,
    Bt470bg = 5,
    Bt601 = 6,
    Smpte240 = 7,
    Ycgco = 8,
    Bt2020Ncl = 9,
    Bt2020Cl = 10,
    Smpte2085 = 11,
    ChromaDerivedNcl = 12,
    ChromaDerivedCl = 13,
    Ictcp = 14,
    YcgcoRe = 16,
    YcgcoRo = 17,
}

impl From<u16> for MatrixCoefficients {
    fn from(value: u16) -> Self {
        match value {
            0 => Self::Identity,
            1 => Self::Bt709,
            2 => Self::Unspecified,
            3 => Self::Reserved,
            4 => Self::Fcc,
            5 => Self::Bt470bg,
            6 => Self::Bt601,
            7 => Self::Smpte240,
            8 => Self::Ycgco,
            9 => Self::Bt2020Ncl,
            10 => Self::Bt2020Cl,
            11 => Self::Smpte2085,
            12 => Self::ChromaDerivedNcl,
            13 => Self::ChromaDerivedCl,
            14 => Self::Ictcp,
            16 => Self::YcgcoRe,
            17 => Self::YcgcoRo,
            _ => Self::default(),
        }
    }
}

#[derive(Debug, Default, PartialEq)]
pub enum AvifError {
    #[default]
    Ok,
    UnknownError(String),
    InvalidFtyp,
    NoContent,
    NoYuvFormatSelected,
    ReformatFailed,
    UnsupportedDepth,
    EncodeColorFailed,
    EncodeAlphaFailed,
    BmffParseFailed(String),
    MissingImageItem,
    DecodeColorFailed,
    DecodeAlphaFailed,
    ColorAlphaSizeMismatch,
    IspeSizeMismatch,
    NoCodecAvailable,
    NoImagesRemaining,
    InvalidExifPayload,
    InvalidImageGrid(String),
    InvalidCodecSpecificOption,
    TruncatedData,
    IoNotSet,
    IoError,
    WaitingOnIo,
    InvalidArgument,
    NotImplemented,
    OutOfMemory,
    CannotChangeSetting,
    IncompatibleImage,
    EncodeGainMapFailed,
    DecodeGainMapFailed,
    InvalidToneMappedImage(String),
}

pub type AvifResult<T> = Result<T, AvifError>;

#[repr(i32)]
#[derive(Clone, Copy, Debug, Default)]
pub enum AndroidMediaCodecOutputColorFormat {
    // Flexible YUV 420 format used for 8-bit images:
    // https://developer.android.com/reference/android/media/MediaCodecInfo.CodecCapabilities#COLOR_FormatYUV420Flexible
    #[default]
    Yuv420Flexible = 2135033992,
    // YUV P010 format used for 10-bit images:
    // https://developer.android.com/reference/android/media/MediaCodecInfo.CodecCapabilities#COLOR_FormatYUVP010
    P010 = 54,
}

impl From<i32> for AndroidMediaCodecOutputColorFormat {
    fn from(value: i32) -> Self {
        match value {
            2135033992 => Self::Yuv420Flexible,
            54 => Self::P010,
            _ => Self::default(),
        }
    }
}

trait OptionExtension {
    type Value;

    fn unwrap_ref(&self) -> &Self::Value;
    fn unwrap_mut(&mut self) -> &mut Self::Value;
}

impl<T> OptionExtension for Option<T> {
    type Value = T;

    fn unwrap_ref(&self) -> &T {
        self.as_ref().unwrap()
    }

    fn unwrap_mut(&mut self) -> &mut T {
        self.as_mut().unwrap()
    }
}

macro_rules! checked_add {
    ($a:expr, $b:expr) => {
        $a.checked_add($b)
            .ok_or(AvifError::BmffParseFailed("".into()))
    };
}

macro_rules! checked_sub {
    ($a:expr, $b:expr) => {
        $a.checked_sub($b)
            .ok_or(AvifError::BmffParseFailed("".into()))
    };
}

macro_rules! checked_mul {
    ($a:expr, $b:expr) => {
        $a.checked_mul($b)
            .ok_or(AvifError::BmffParseFailed("".into()))
    };
}

macro_rules! checked_decr {
    ($a:expr, $b:expr) => {
        $a = checked_sub!($a, $b)?
    };
}

macro_rules! checked_incr {
    ($a:expr, $b:expr) => {
        $a = checked_add!($a, $b)?
    };
}

pub(crate) use checked_add;
pub(crate) use checked_decr;
pub(crate) use checked_incr;
pub(crate) use checked_mul;
pub(crate) use checked_sub;

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Grid {
    pub rows: u32,
    pub columns: u32,
    pub width: u32,
    pub height: u32,
}

#[cfg(feature = "encoder")]
impl Grid {
    pub(crate) fn is_last_column(&self, index: u32) -> bool {
        (index + 1) % self.columns == 0
    }

    pub(crate) fn is_last_row(&self, index: u32) -> bool {
        index >= (self.columns * (self.rows - 1))
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum Category {
    #[default]
    Color,
    Alpha,
    Gainmap,
}

impl Category {
    const COUNT: usize = 3;
    const ALL: [Category; Category::COUNT] = [Self::Color, Self::Alpha, Self::Gainmap];

    pub fn planes(&self) -> &[Plane] {
        match self {
            Category::Alpha => &A_PLANE,
            _ => &YUV_PLANES,
        }
    }

    #[cfg(feature = "encoder")]
    pub(crate) fn infe_name(&self) -> String {
        match self {
            Self::Color => "Color",
            Self::Alpha => "Alpha",
            Self::Gainmap => "GMap",
        }
        .into()
    }
}

/// cbindgen:rename-all=CamelCase
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[repr(C)]
pub struct PixelAspectRatio {
    pub h_spacing: u32,
    pub v_spacing: u32,
}

/// cbindgen:field-names=[maxCLL, maxPALL]
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ContentLightLevelInformation {
    pub max_cll: u16,
    pub max_pall: u16,
}

#[derive(Clone, Debug, Default)]
pub struct Nclx {
    pub color_primaries: ColorPrimaries,
    pub transfer_characteristics: TransferCharacteristics,
    pub matrix_coefficients: MatrixCoefficients,
    pub yuv_range: YuvRange,
}

pub const MAX_AV1_LAYER_COUNT: usize = 4;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RepetitionCount {
    Unknown,
    Infinite,
    Finite(u32),
}

impl Default for RepetitionCount {
    fn default() -> Self {
        Self::Finite(0)
    }
}

impl RepetitionCount {
    pub fn create_from(value: i32) -> Self {
        if value >= 0 {
            RepetitionCount::Finite(value as u32)
        } else {
            RepetitionCount::Infinite
        }
    }

    #[cfg(feature = "encoder")]
    pub(crate) fn is_infinite(&self) -> bool {
        match self {
            Self::Finite(x) if *x >= i32::MAX as u32 => true,
            Self::Infinite => true,
            _ => false,
        }
    }

    #[cfg(feature = "encoder")]
    pub(crate) fn loop_count(&self) -> u64 {
        match self {
            Self::Finite(x) if *x < i32::MAX as u32 => *x as u64 + 1,
            _ => i32::MAX as u64,
        }
    }
}

pub fn codec_versions() -> String {
    let versions = &[
        decoder::CodecChoice::versions(),
        #[cfg(feature = "aom")]
        codecs::aom::Aom::version(),
    ];
    versions.join(", ")
}
