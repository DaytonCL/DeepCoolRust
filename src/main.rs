#![cfg_attr(not(windows), allow(unused))]
#[cfg(not(windows))]
compile_error!("ak500-digital-lite supports Windows only.");

#[cfg(windows)]
mod app {
    use hidapi::{HidApi, HidDevice};
    use std::{error::Error, io, thread, time::Duration};

    const VID: u16 = 0x3633;
    const PID: u16 = 0x0003;
    const LEN: usize = 64;
    const REPORT_ID: u8 = 0x10;

    #[repr(C)]
    #[derive(Clone, Copy, Default)]
    struct FileTime {
        low: u32,
        high: u32,
    }

    impl FileTime {
        fn value(self) -> u64 {
            (u64::from(self.high) << 32) | u64::from(self.low)
        }
    }

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetSystemTimes(idle: *mut FileTime, kernel: *mut FileTime, user: *mut FileTime) -> i32;
    }

    #[derive(Clone, Copy)]
    struct CpuTimes {
        idle: u64,
        total: u64,
    }

    fn times() -> io::Result<CpuTimes> {
        let mut idle = FileTime::default();
        let mut kernel = FileTime::default();
        let mut user = FileTime::default();
        // Windows reports kernel time inclusive of idle time.
        if unsafe { GetSystemTimes(&mut idle, &mut kernel, &mut user) } == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(CpuTimes {
            idle: idle.value(),
            total: kernel.value().saturating_add(user.value()),
        })
    }

    fn usage(old: CpuTimes, new: CpuTimes) -> Option<f64> {
        let total = new.total.saturating_sub(old.total);
        if total == 0 {
            return None;
        }
        let idle = new.idle.saturating_sub(old.idle);
        Some((100.0 * total.saturating_sub(idle) as f64 / total as f64).clamp(0.0, 100.0))
    }

    fn initialize_packet() -> [u8; LEN] {
        let mut p = [0u8; LEN];
        p[0] = REPORT_ID;
        p[1] = 0xAA;
        p
    }

    fn usage_packet(percent: f64) -> [u8; LEN] {
        let mut p = [0u8; LEN];
        let percent = percent.clamp(0.0, 100.0);
        let rounded = percent.round() as u16;
        p[0] = REPORT_ID;
        p[1] = 0x4C;
        p[2] = if percent < 15.0 {
            1
        } else {
            (percent / 10.0).round().clamp(1.0, 10.0) as u8
        };
        p[3] = (rounded / 100) as u8;
        p[4] = ((rounded / 10) % 10) as u8;
        p[5] = (rounded % 10) as u8;
        p
    }

    fn test_temp_packet(temp: u16) -> [u8; LEN] {
        let mut p = [0u8; LEN];
        let temp = temp.min(120);
        p[0] = REPORT_ID;
        p[1] = 0x13;
        p[2] = ((temp as f64 / 9.0).round() as u8).min(10);
        p[3] = (temp / 100) as u8;
        p[4] = ((temp / 10) % 10) as u8;
        p[5] = (temp % 10) as u8;
        p[6] = u8::from(temp >= 90);
        p
    }

    fn send(device: &HidDevice, p: &[u8; LEN]) -> Result<(), Box<dyn Error>> {
        let written = device.write(p)?;
        if written != LEN {
            return Err(format!("Incomplete HID write: {written}/{LEN} bytes").into());
        }
        Ok(())
    }

    fn connect(api: &HidApi) -> Result<HidDevice, Box<dyn Error>> {
        let mut seen = false;
        let mut failures = Vec::new();
        for info in api.device_list().filter(|d| d.vendor_id() == VID && d.product_id() == PID) {
            seen = true;
            match info.open_device(api) {
                Ok(d) => match send(&d, &initialize_packet()) {
                    Ok(()) => return Ok(d),
                    Err(e) => failures.push(e.to_string()),
                },
                Err(e) => failures.push(e.to_string()),
            }
        }
        if !seen {
            return Err("AK500 DIGITAL display not found (expected VID 3633 PID 0003)".into());
        }
        Err(format!(
            "Display found but opening/initialization failed: {}. Close the original DeepCool software.",
            failures.join("; ")
        ).into())
    }

    struct Options {
        list: bool,
        once: bool,
        dry_run: bool,
        quiet: bool,
        interval_ms: u64,
        test_temp: Option<u16>,
    }

    impl Default for Options {
        fn default() -> Self {
            Self { list: false, once: false, dry_run: false, quiet: false, interval_ms: 2000, test_temp: None }
        }
    }

    fn parse_options() -> Result<Option<Options>, Box<dyn Error>> {
        let mut o = Options::default();
        let mut args = std::env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "-h" | "--help" => {
                    println!(
                        "AK500 DIGITAL Lite\n\
                        --list                List DeepCool USB HID devices\n\
                        --once                Send one update and exit\n\
                        --interval-ms N       Update interval 500-10000 (default 2000)\n\
                        --quiet               Suppress per-update logs\n\
                        --dry-run             Print without writing to the display\n\
                        --test-temp N         STATIC test Celsius value 0-120, never live temperature"
                    );
                    return Ok(None);
                }
                "--list" => o.list = true,
                "--once" => o.once = true,
                "--quiet" => o.quiet = true,
                "--dry-run" => o.dry_run = true,
                "--interval-ms" => {
                    o.interval_ms = args.next().ok_or("Missing --interval-ms value")?.parse()?;
                    if !(500..=10000).contains(&o.interval_ms) {
                        return Err("--interval-ms must be 500..10000".into());
                    }
                }
                "--test-temp" => {
                    let value: u16 = args.next().ok_or("Missing --test-temp value")?.parse()?;
                    if value > 120 {
                        return Err("--test-temp must be 0..120".into());
                    }
                    o.test_temp = Some(value);
                    o.once = true;
                }
                _ => return Err(format!("Unknown option: {arg}").into()),
            }
        }
        Ok(Some(o))
    }

    fn print_packet(p: &[u8; LEN]) {
        println!("HID report first bytes: {:02X?}", &p[..8]);
    }

    pub fn run() -> Result<(), Box<dyn Error>> {
        let Some(options) = parse_options()? else { return Ok(()); };

        if options.dry_run {
            if let Some(temp) = options.test_temp {
                println!("STATIC TEST ONLY: {temp} C. Not a measured temperature.");
                print_packet(&test_temp_packet(temp));
            } else {
                let old = times()?;
                thread::sleep(Duration::from_millis(options.interval_ms));
                let new = times()?;
                let percent = usage(old, new).ok_or("Could not sample CPU usage")?;
                println!("CPU usage: {percent:.1}%");
                print_packet(&usage_packet(percent));
            }
            return Ok(());
        }

        let mut api = HidApi::new()?;
        if options.list {
            let mut count = 0;
            for d in api.device_list().filter(|d| d.vendor_id() == VID) {
                println!(
                    "VID={:04X} PID={:04X} UsagePage={:04X} Usage={:04X} Product={}",
                    d.vendor_id(), d.product_id(), d.usage_page(), d.usage(),
                    d.product_string().unwrap_or("Unknown")
                );
                count += 1;
            }
            if count == 0 {
                println!("No DeepCool HID devices found.");
            }
            return Ok(());
        }

        let mut device = connect(&api)?;
        println!("AK500 DIGITAL display connected via Windows HID.");

        if let Some(temp) = options.test_temp {
            println!("STATIC TEST ONLY: {temp} C for two seconds. Not a real sensor reading.");
            send(&device, &test_temp_packet(temp))?;
            thread::sleep(Duration::from_secs(2));
            let old = times()?;
            thread::sleep(Duration::from_millis(500));
            let percent = usage(old, times()?).unwrap_or(0.0);
            send(&device, &usage_packet(percent))?;
            println!("Restored one CPU usage sample: {percent:.1}%");
            return Ok(());
        }

        let mut old = times()?;
        loop {
            thread::sleep(Duration::from_millis(options.interval_ms));
            let new = times()?;
            if let Some(percent) = usage(old, new) {
                let packet = usage_packet(percent);
                if let Err(err) = send(&device, &packet) {
                    eprintln!("HID write failed: {err}. Reconnecting...");
                    drop(device);
                    thread::sleep(Duration::from_secs(1));
                    api.refresh_devices()?;
                    device = connect(&api)?;
                } else if !options.quiet {
                    println!("CPU usage: {percent:.1}%");
                }
            }
            old = new;
            if options.once { break; }
        }
        Ok(())
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn initialization_packet_is_correct() {
            assert_eq!(&initialize_packet()[..3], &[0x10, 0xAA, 0]);
        }

        #[test]
        fn usage_packet_has_expected_digits() {
            assert_eq!(&usage_packet(57.0)[..7], &[0x10, 0x4C, 6, 0, 5, 7, 0]);
        }

        #[test]
        fn usage_clamps_to_100() {
            assert_eq!(&usage_packet(200.0)[..7], &[0x10, 0x4C, 10, 1, 0, 0, 0]);
        }

        #[test]
        fn static_temperature_test_has_expected_digits() {
            assert_eq!(&test_temp_packet(45)[..7], &[0x10, 0x13, 5, 0, 4, 5, 0]);
        }

        #[test]
        fn cpu_delta_is_correct() {
            let old = CpuTimes { idle: 400, total: 1000 };
            let new = CpuTimes { idle: 440, total: 1100 };
            assert_eq!(usage(old, new), Some(60.0));
        }
    }
}

#[cfg(windows)]
fn main() {
    if let Err(e) = app::run() {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
