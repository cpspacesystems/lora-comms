# LoRa-Comms
This is the radio communications stack for CPSS. This repository contains all the code necessary to operate the radios, gather data from our networks and send them over LoRa. 

LoRa-Comms is designed to work with a LR1121 on our TOM computer on board our rockets. And with a SX1302 mounted to a raspberry-pi using a HAT. You can develop LoRa-Comms on any *unix compatible environment, although some parts may not be able to be tested unless you have the target raspberry pis. 

# Installation
Follow these steps one by one. If during any steps any errors occur, try to read the error messages and see if you can resolve them. You can always restart the process to get a clean slate. Any future steps will fail if prior steps error.

0. Go to the installation directory     
    `cd /opt/`  
    If you are not installing for a production build so you can run lora-comms using `systemd`. Then you may change the installation location. Otherwise you must install into `/opt/`.
1. Get the repository:
    ```bash
    git clone https://github.com/cpspacesystems/lora-comms
    cd lora-comms
    ```
    Note: you can select the development branch afterwards with `git checkout dev` if you need features that are still unstable.
2. Initialize and build all required libraries and artifacts:  
    ```bash
    ./setup_all.bash
    ```
3. Build LoRa-Comms     
    The steps here are for building directly on the target device. This is fine if you are building on a more powerful machine, but be warned that it will take an **Extremely LONG Time** if you were to build lora-comms on a raspberry-pi. Please see [cross compilation](#cross-compilation) if you want to cross compile on a more powerful machine and avoid waiting an eternity.     

    If you are building for an optimized, release build. Run the following command:
    ```bash 
    cargo build --release --no-default-features
    ```
    The binaries will be in `./target/release/main` for rocket side program and `./target/release/ground` for ground side program.  

    If you are building for an unoptimized, debug build. Run the following command:
    ```bash 
    cargo build
    ```
    The binaries will be in `./target/debug/main` for rocket side program and `./target/debug/ground` for ground side program.  

    By default a debug build will build with simulation features instead of using the actual hardware. Append `--no-default-features` to the command if you want actual hardware to be used for a debug build. 
    
    Check out [Features](#features) for cargo features available for this project. And also [cargo.toml](cargo.toml) for more information on how this project is configured.
4. You can now run the binary programs. `main` is the program that should be ran on the rocket side. `ground` is the program for the ground side. If you wish to set up a `systemd` service to automatically start lora-comms, then go to step 5, otherwise you have lora-comms installed!
5. Setting up `systemd` services to run lora-comms automatically:       
    
    Make sure you have lora-comms installed in `/opt/lora-comms`. If not, go back to step 0 and restart.  

    Otherwise, if you are setting up for the ground side, run:
    ```bash
    sudo ln -s /opt/lora-comms/lora_ground.service /etc/systemd/system/
    sudo systemctl enable lora_ground
    ```
    If you are setting up for the rocket side, run:
    ```bash
    sudo ln -s /opt/lora-comms/lora_tom_main.service /etc/systemd/system/
    sudo systemctl enable lora_tom_main
    ```

    lora-comms will output its logs into the systemd-journal. You can use journalctl to see the full logs or `sudo systemctl status lora_<name>` for a quick status on the service.

# Configuration

Edit the file in [./etc/config.toml](./etc/config.toml) to configure what is sent over LoRa. See documentation within config.toml for more info.

# Cross Compilation
1. Install [Docker](https://www.docker.com/)
2. Install cross: `cargo install cross`

Then run:  
    ```
./cross_build.bash "aarch64-unknown-linux-gnu" "<ssh-username>@<domain>" "/opt/lora-comms"
    ```     
Replacing `<ssh-username>@domain` with the ssh login info for the target machine.

or `./cross_release.bash "aarch64-unknown-linux-gnu" "<ssh-username>@<domain>" "/opt/lora-comms"` to build for release.

Or if you set up ssh-config and ssh public-key login. With the names `ground-station` and `cpss-tom`. For ground station and the rocket computer respectively. then you can use `cross-ground.bash` or `cross-tom.bash` to deploy for debugging purposes without having to type commands every time. 

# Features
The code should work for non-pi platforms if you modify [reset_lgw.sh](reset_lgw.sh).