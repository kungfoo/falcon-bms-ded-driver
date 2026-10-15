# falcon-bms-ded-driver

This drives the dot matrix screen of a WINCTRL ICP device over USB hidapi from Falcon BMS shared memory. 

## Settings

One can set the `log_level` and the `update_period_ms` in `config.toml`.
Default values are info and 33ms, which equates to 30fps.
Theoretically, one can also override the `vendor_id` and `product_id` (if there are ever new hardware revisions).

## How to build this?

This will build for windows (probably `x86_64-pc-windows-gnu`, since it directly uses Win32 API and there is no point in building it for other platforms. After all BMS currently runs on windows (or wine) only. 

- Install MSYS2 MINGW64
- Install the required packages:
	- rustup / cargo
	- mingw-w64-clang-x86_64-clang
	- cmake

That should be all that is needed for a succefull incantation of `cargo build`.

It will also build on linux with `cargo build --target x86_64-pc-windows-gnu`.

## License
