# AK500 DIGITAL Lite

Lightweight Windows-only Rust application for the **DeepCool AK500 DIGITAL / DIGITAL WH** CPU cooler display.

## Version 0.2 — live CPU temperature + usage

- Reads real CPU package temperature from the **AMD Ryzen Master Monitoring SDK**.
- Reads aggregate CPU usage via the Windows GetSystemTimes API.
- Samples **both metrics once every second**.
- Shows **CPU temperature for five seconds**, then **CPU usage for five seconds**, repeating continuously.
- Sends 64-byte HID reports directly to the AK500 DIGITAL (USB VID 3633, PID 0003).
- Runs as **one executable/process**. A small C++ SDK adapter is compiled into the Rust binary at build time; no separate helper application, tray UI, or bridge DLL is launched.
- No PawnIO and no official DeepCool software required while running.

**Requirements:** Windows 10/11 x64; installed AMD Ryzen Master Monitoring SDK (including its official driver); run the executable as Administrator for hardware sensor access. The SDK's installed Platform.dll and Device.dll are loaded dynamically at runtime. They are **not bundled** in this repository.

## Build

Install the current stable Rust **MSVC** toolchain and the **Visual Studio C++ Build Tools** (Desktop development with C++).

The SDK must be installed at the default path:

    C:\Program Files\AMD\RyzenMasterMonitoringSDK\

If installed elsewhere, set AMDRMMONITORSDKPATH to the SDK **root** containing include and bin:

    $env:AMDRMMONITORSDKPATH = 'D:\SomeFolder\RyzenMasterMonitoringSDK'

From the repository directory, run in PowerShell:

    cargo build --release
    cargo test

Output: target\release\ak500-digital-lite.exe

## Verify before writing to the cooler

Launch PowerShell **as Administrator**. From the project directory run:

    .\target\release\ak500-digital-lite.exe --dry-run --once

Expected output includes actual temperature in Celsius, aggregate CPU usage, and 'display: temperature'.

List connected DeepCool HID devices:

    .\target\release\ak500-digital-lite.exe --list

Close the official DeepCool application before starting live mode, to avoid simultaneous HID writes:

    .\target\release\ak500-digital-lite.exe --quiet

The screen alternates as follows:

| Time since launch | Screen | Both sensors |
|---|---|---|
| 0–5 seconds | CPU temperature °C | refreshed every second |
| 5–10 seconds | CPU usage % | refreshed every second |
| 10–15 seconds | CPU temperature °C | refreshed every second |

Stop with **Ctrl+C**. The program does not install services or change startup, fan controls, BIOS settings, or Windows registry.

Additional options:

- --once: read one sample and send one temperature frame.
- --dry-run: no HID access; show sensor data in PowerShell.
- --list: identify the DeepCool HID device.
- --test-temp 45: send a **static demo** temperature for two seconds, then restore one usage frame. This is *not* a sensor measurement.
- --quiet: suppress per-update terminal output.
- --help: list options.

### Limitations

- The native AMD adapter uses headers and DLL interfaces provided by the **installed AMD SDK**; SDK installation and administrative privileges remain necessary.
- This exact version has **not been compiled or tested against the user's SDK/hardware yet**. The previous usage-only version was successfully tested on the AK500 DIGITAL WH.
- Real temperature values come from AMD GetCPUParameters / dTemperature, not per-core thermal sensors or the test-temperature command.
- A hardware read failure terminates the program rather than displaying a fake or stale temperature.
- The SDK should be polled no faster than once per second, per AMD's official recommendations.
- CPU and memory overhead still need to be benchmarked on the target machine.

## Credits and licenses

- [Russian setup guide](START_HERE_RU.md)
- [Protocol and SDK attribution](NOTICE.md)
- [Deepcool-Digital-Windows](https://github.com/Shark1nat0r/deepcool-digital-windows) (MIT license) for AK-series HID protocol.
- [SidebarMonitor RyzenShim](https://github.com/WRCX/SidebarMonitor/blob/main/native/RyzenShim/RyzenShim.cpp) (MIT license) as a reference for bridging the AMD C++ SDK interface to plain C functions.
- [AMD Ryzen Master Monitoring SDK](https://www.amd.com/en/developer/ryzen-master-monitoring-sdk.html) (proprietary AMD SDK; users obtain and install it separately).

No AMD SDK headers, proprietary DLLs, or drivers are committed to this repository.
