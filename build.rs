
fn main() {
    println!("cargo:rustc-link-search=native=/home/alh/lora-comms/temp/sx1302_hal/libloragw");
    println!("cargo:rustc-link-search=native=/home/alh/lora-comms/temp/sx1302_hal/libtools");
}
