use std::{env, fs, path::Path};

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
    let source_dir = std::env::var("CARGO_MANIFEST_DIR").expect("Expected source dir to be set!");
    let out_dir = env::var_os("OUT_DIR").expect("OUT_DIR not set!");

    if let Ok(v) = env::var("CROSS") && v.trim() == "1" {
        cross();
    } else {
        native();
    }
    
    #[cfg(not(feature = "hardware_attached_full_system"))]
    codegen::parse_data_def::parse_data_def("etc/DataDefination.vcsv");
    
    #[cfg(feature = "hardware_attached_full_system")]
    {
        codegen::parse_data_def::parse_data_def("etc/SimDataDef.vcsv");
        std::process::Command::new("cargo")
            .current_dir(format!("{}/src/simulation/binaries", source_dir))
            .args(["build"])
            .spawn()
            .expect("Expected to be able to build binaries used for hardware attached simulation!");
        fs::write(Path::new(&out_dir).join("codegen_hws_binary_paths.rs"), 
            format!("
                pub const HWAS_TISM_SOURCE: &str = \"{0}/src/simulation/binaries/target/debug/sim_tism_source\"; 
                pub const HWAS_ZENOH_SOURCE: &str = \"{0}/src/simulation/binaries/target/debug/sim_zenoh_source\";
            ", source_dir)
        ).expect("Expected to be able to write to out_dir!");
    }
}
