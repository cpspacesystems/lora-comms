


fn main() {
    println!("cargo:rustc-link-search=native=/home/adam/projects/cpss/drivers/lora-comms/src");
    println!("cargo:rustc-link-lib=static=lr11xx_driver"); // note: no 'lib' prefix, no '.a'
}