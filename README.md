# AK500 DIGITAL Lite

A minimal Windows-only Rust prototype for the DeepCool AK500 DIGITAL / AK500 DIGITAL WH screen.

**Current functionality:** read real aggregate CPU usage via Windows `GetSystemTimes` and send it to the display using USB HID. No PawnIO, DeepCool application, additional HID driver, or background service is needed at runtime.

**Important limitation:** real Ryzen 7 7800X3D temperature is **not implemented**. Stock Windows user-mode APIs do not expose a reliable CPU Tdie sensor. The `--test-temp` command is a short, clearly marked static **demo only** and must not be treated as a thermal reading.

## Build

On Windows 10/11 x64, install Rust (MSVC toolchain) and Visual Studio C++ Build Tools, then run:

```powershell
cargo build --release
cargo test
```

## Usage

```powershell
.\target\release\ak500-digital-lite.exe --dry-run --once
.\target\release\ak500-digital-lite.exe --list
.\target\release\ak500-digital-lite.exe --once
.\target\release\ak500-digital-lite.exe --quiet
.\target\release\ak500-digital-lite.exe --test-temp 45
```

The USB identity for AK500 DIGITAL is VID `3633`, PID `0003`. Shut down the official DeepCool application before accessing the screen to avoid conflicting USB writes. The continuous display update defaults to every 2 seconds.

`--test-temp 45` displays 45°C for two seconds, then restores a one-time CPU usage reading; 45°C is not measured.

The application changes **no Windows services, drivers, registry entries, startup entries, BIOS settings, or fan controls**. Stop it with Ctrl+C.

**Not yet built/tested on the user's Windows PC or actual AK500 DIGITAL WH hardware.** Resource savings and final binary size have not been benchmarked.

## Documentation

- [Russian setup guide](START_HERE_RU.md)
- [Protocol attribution](NOTICE.md)
- [Original protocol implementation](https://github.com/Shark1nat0r/deepcool-digital-windows)

## Protocol

The original MIT-licensed DeepCool AK series project was used as a reference for HID protocol constants and packet layout. This Rust implementation independently recreates those messages. No code or binaries from the original project are bundled.
