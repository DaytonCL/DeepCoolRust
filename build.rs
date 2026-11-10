use std::{env, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=native/amd_sensor.cpp");
    println!("cargo:rerun-if-env-changed=AMDRMMONITORSDKPATH");

    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }

    if env::var("CARGO_CFG_TARGET_ENV").as_deref() != Ok("msvc") {
        panic!("The AMD SDK adapter requires the x86_64-pc-windows-msvc toolchain.");
    }

    let sdk = env::var_os("AMDRMMONITORSDKPATH")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\Program Files\AMD\RyzenMasterMonitoringSDK"));
    let includes = sdk.join("include");

    for header in ["IPlatform.h", "IDeviceManager.h", "ICPUEx.h", "DeviceType.h"] {
        if !includes.join(header).is_file() {
            panic!(
                "AMD SDK header not found: {}. Install the Ryzen Master Monitoring SDK or set AMDRMMONITORSDKPATH to its root directory.",
                includes.join(header).display()
            );
        }
    }

    cc::Build::new()
        .cpp(true)
        .std("c++17")
        .include(includes)
        .file("native/amd_sensor.cpp")
        .warnings(true)
        .compile("ak500_amd_sensor");

    println!("cargo:rustc-link-lib=Advapi32");
}
