# Ninja keyboard firmware

## Build

- Clone [Embassy](https://github.com/embassy-rs/embassy) and init its submodules one folder up from ninja_firmware repo
`$ git clone https://github.com/embassy-rs/embassy`
`$ cd embassy`
`$ git submodule init`
`$ git submodule update`
`$ cargo install probe-run`
`$ cd ..`
- Conect stlink to swd header of bluepill board
- Build Ninja firmware
`$ cd ninja_firmware`
`$ cargo run`
