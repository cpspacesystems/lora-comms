use std::env;

fn native() {
    println!("cargo:rustc-link-search=native=./lib/sx1302_hal/libloragw");
    println!("cargo:rustc-link-search=native=./lib/sx1302_hal/libtools");
}

fn cross() {
    println!("cargo:rustc-link-search=native=./cross/lib/sx1302_hal/libloragw");
    println!("cargo:rustc-link-search=native=./cross/lib/sx1302_hal/libtools");
}
fn main() {
    println!("cargo:rustc-link-search=native=./");
    //println!("cargo:rustc-link-search=native=/home/adam/Desktop/LR1121Rust/lora-comms");
    // println!("cargo:rustc-link-lib=static=lora_full");
    // println!("cargo:rustc-link-lib=dylib=stdc++");
    // println!("cargo:rustc-link-lib=dylib=lgpio");

    if let Ok(v) = env::var("CROSS") && v.trim() == "1" {
        cross();
    } else {
        native();
    }
}