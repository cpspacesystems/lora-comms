
fn main() {
    println!("cargo:rustc-link-search=native=./lib/sx1302_hal/libloragw");
    println!("cargo:rustc-link-search=native=./lib/sx1302_hal/libtools");
}
