fn main() {
    println!("cargo:rustc-link-search=native=/home/adam/Desktop/LR1121Rust/lora-comms");
    println!("cargo:rustc-link-lib=static=lora_full");
    println!("cargo:rustc-link-lib=dylib=stdc++");
    println!("cargo:rustc-link-lib=dylib=lgpio");
}