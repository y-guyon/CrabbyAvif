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

// Build rust library and bindings for libyuv.

use std::env;
use std::path::Path;
use std::path::PathBuf;

fn path_buf(inputs: &[&str]) -> PathBuf {
    let path: PathBuf = inputs.iter().collect();
    path
}

fn main() -> Result<(), String> {
    println!("cargo:rerun-if-changed=build.rs");
    if !cfg!(feature = "libgav1") {
        return Ok(());
    }

    let build_target = std::env::var("TARGET").unwrap();
    let build_dir = if build_target.contains("android") {
        if build_target.contains("x86_64") {
            "build.android/x86_64"
        } else if build_target.contains("x86") {
            "build.android/x86"
        } else if build_target.contains("aarch64") {
            "build.android/aarch64"
        } else if build_target.contains("arm") {
            "build.android/arm"
        } else {
            return Err(
                "Unknown target_arch for android. Must be one of x86, x86_64, arm, aarch64.".into(),
            );
        }
    } else {
        "build"
    };

    let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let abs_library_dir = PathBuf::from(&project_root).join("libgav1");
    let abs_object_dir = PathBuf::from(&abs_library_dir).join(build_dir);
    let library_file = PathBuf::from(&abs_object_dir).join(if cfg!(target_os = "windows") {
        "libgav1.lib"
    } else {
        "libgav1.a"
    });
    if !Path::new(&library_file).exists() {
        return Err(
            "libgav1 binaries could not be found locally. Disable the libgav1 feature or \
            build the dependency locally by running libgav1.cmd from sys/libgav1-sys."
                .into(),
        );
    }
    println!("cargo:rustc-link-search={}", abs_object_dir.display());
    let library_name = if cfg!(target_os = "windows") { "libgav1" } else { "gav1" };
    println!("cargo:rustc-link-lib=static={library_name}");

    // Generate bindings.
    let header_file = PathBuf::from(&abs_library_dir).join(path_buf(&["src", "gav1", "decoder.h"]));
    let version_dir = PathBuf::from(&abs_library_dir).join(path_buf(&["src"]));
    let outdir = std::env::var("OUT_DIR").expect("OUT_DIR not set");
    let outfile = PathBuf::from(&outdir).join("libgav1_bindgen.rs");
    let extra_includes_str = format!("-I{}", version_dir.display());
    let mut bindings = bindgen::Builder::default()
        .header(header_file.into_os_string().into_string().unwrap())
        .clang_arg(extra_includes_str)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .layout_tests(false)
        .generate_comments(false);
    let allowlist_items = &[
        "Libgav1DecoderCreate",
        "Libgav1DecoderDequeueFrame",
        "Libgav1DecoderDestroy",
        "Libgav1DecoderEnqueueFrame",
        "Libgav1DecoderSettingsInitDefault",
    ];
    for allowlist_item in allowlist_items {
        bindings = bindings.allowlist_item(allowlist_item);
    }
    let bindings = bindings.generate().map_err(|err| err.to_string())?;
    bindings
        .write_to_file(outfile.as_path())
        .map_err(|err| err.to_string())?;
    Ok(())
}
