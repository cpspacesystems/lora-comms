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
    if let Ok(v) = env::var("CROSS") && v.trim() == "1" {
        cross();
    } else {
        native();
    }
}
