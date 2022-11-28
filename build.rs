use std::env;
use std::fs::create_dir;
use std::path::PathBuf;

use cmake::Config;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let glslang_dir = env::var("GLSLANG_TARGET_DIR")
        .expect("required environment variable GLSLANG_TARGET_DIR is not provided");
    let waifu2x_dir = out_dir.join("waifu2x");
    create_dir(&waifu2x_dir).unwrap_or_default();
    let waifu2x = {
        let mut config = Config::new("src/");
        config
            .out_dir(waifu2x_dir)
            .define("GLSLANG_TARGET_DIR", glslang_dir);
        config.build()
    };
    println!("cargo:rustc-link-search=native={}", waifu2x.join("lib").display());
    println!("cargo:rustc-link-lib=static:-bundle={}", "waifu2x-ncnn-vulkan-wrapper");
    println!("cargo:rustc-link-lib=dylib=ncnn");
    println!("cargo:rustc-link-lib=dylib={}", "stdc++");
}