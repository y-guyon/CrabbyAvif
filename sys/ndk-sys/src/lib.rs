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

#![allow(warnings)]
#[cfg(feature = "android_mediacodec")]
pub mod bindings {
    #[cfg(not(android_soong))]
    include!(env!("CRABBYAVIF_ANDROID_NDK_MEDIA_BINDINGS_RS"));
    // Android's soong build system does not support setting environment variables. Set the source
    // file name directly relative to the OUT_DIR environment variable.
    #[cfg(android_soong)]
    include!(concat!(env!("OUT_DIR"), "/ndk_media_bindgen.rs"));
}
