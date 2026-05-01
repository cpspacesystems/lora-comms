

static TOKIO_RT: std::sync::OnceLock<tokio::runtime::Runtime> = std::sync::OnceLock::new();
pub fn get_tokio_or_init() -> tokio::runtime::Handle {
    if let Ok(h) = tokio::runtime::Handle::try_current() {
        h
    } else {
        TOKIO_RT.get_or_init(|| {
            tokio::runtime::Runtime::new().unwrap()
        }).handle().clone()
    }
}

#[cfg(feature = "hardware_attached_full_system")]
pub mod hardware_attached {
    use std::{process, thread::sleep, time};

    use crate::simulation::get_tokio_or_init;

    mod codegen {
        include!{concat!(env!("OUT_DIR"), "/codegen_hws_binary_paths.rs")}
    }

    pub fn spawn_tism(size: usize, path: impl AsRef<str>) {
        println!("HWAS: Spawn tism pub size of {} at {}", size, path.as_ref());
        let args = [size.to_string(), path.as_ref().to_string()];

        std::thread::spawn(|| {
            process::Command::new(codegen::HWAS_TISM_SOURCE)
                .args(args)
                .spawn().expect("HWAS: Faield to spawn TISM publisher.");
        });
        // get_tokio_or_init().spawn(async {
        //     let _ = tokio::process::Command::new(codegen::HWAS_TISM_SOURCE)
        //     .args(args)
        //     .kill_on_drop(true)
        //     .spawn().expect("HWAS: Faield to spawn TISM publisher.")
        //     .wait().await
        //     ;
        // });
    }

    pub fn spwan_zenoh(size: usize, path: impl AsRef<str>) {
        todo!();
        // println!("HWAS: Spawn Zenoh pub size of {} at {}", size, path.as_ref());
        // let args = [size.to_string(), path.as_ref().to_string()];
        // get_tokio_or_init().spawn(async {
        //     let _ = tokio::process::Command::new(codegen::HWAS_ZENOH_SOURCE)
        //     .args(args)
        //     .kill_on_drop(true)
        //     .spawn().expect("HWAS: Faield to spawn Zenoh publisher.")
        //     .wait().await
        //     ;
        // });
    }
}