use std::env;

#[path = "src/codegen/mod.rs"]
mod codegen;

fn native() {
    println!("cargo:rustc-link-search=native=./lib/sx1302_hal/libloragw");
    println!("cargo:rustc-link-search=native=./lib/sx1302_hal/libtools");
}

fn cross() {
    println!("cargo:rustc-link-search=native=./cross/lib/sx1302_hal/libloragw");
    println!("cargo:rustc-link-search=native=./cross/lib/sx1302_hal/libtools");
}

fn main() {
    codegen::parse_data_def::parse_data_def("etc/DataDefination.vcsv");

    if let Ok(v) = env::var("CROSS") && v.trim() == "1" {
        cross();
    } else {
        native();
    }
}
