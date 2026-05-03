use std::{env, fs, path::Path};

fn add_linker_search_paths(lib_root: &str) {
    println!("cargo:rustc-link-search=native={}/lib/sx1302_hal/libloragw", lib_root);
    println!("cargo:rustc-link-search=native={}/lib/sx1302_hal/libtools", lib_root);
    println!("cargo:rustc-link-search=native={}/lib/lr1121_radiolib_bridge/build", lib_root);
    println!("cargo:rustc-link-search=native={}/lib/lr1121_radiolib_bridge/build/RadioLib", lib_root);
}

fn main() {
    let _source_dir = std::env::var("CARGO_MANIFEST_DIR").expect("Expected source dir to be set!");
    let _out_dir = env::var_os("OUT_DIR").expect("OUT_DIR not set!");

    if let Ok(v) = env::var("CROSS") && v.trim() == "1" {
        add_linker_search_paths("./cross");
    } else {
        add_linker_search_paths(".");
    }
    
    println!("cargo:rustc-link-lib=dylib=stdc++");
}
