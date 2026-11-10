#![cfg_attr(not(windows), allow(unused))]

#[cfg(not(windows))]
compile_error!("ak500-digital-lite supports Windows only.");

#[cfg(windows)]
mod app {
    use hidapi::{HidApi, HidDevice};
    use std::error::Error;
    use std::io;
    use std::thread;
    use std::time::{Duration, Instant};

    const VENDOR_ID: u16 = 0x3633;
    const AK500_DIGITAL_PRODUCT_ID: u16 = 0x0003;
    const REPORT_LENGTH: usize = 64;
    const REPORT_ID: u8 = 0x10;
    const INITIALIZE: u8 = 0xAA;
    const CPU_USAGE_MODE: u8 = 0x4C;
    const CPU_TEMPERATURE_CELSIUS_MODE: u8 = 0x13;
    const SAMPLE_INTERVAL: Duration = Duration::from_secs(1);
    const MODE_INTERVAL: Duration = Duration::from_secs(5);

    unsafe extern "C" {
        fn ak500_sdk_open() -> i32;
        fn ak500_sdk_read(temperature_c: *mut f64) -> i32;
        fn ak500_sdk_close();
    }

    struct AmdTemperature;

    impl AmdTemperature {
        fn open() -> Result<Self, Box<dyn Error>> {
            let result = unsafe { ak500_sdk_open() };
            if result != 0 {
                return Err(format!(
                    "AMD Monitoring SDK initialization failed (code {result}). Run PowerShell as Administrator, verify the SDK installation, and close other Ryzen monitoring apps."
                ).into());
            }
            Ok(Self)
        }

        fn read(&self) -> Result<f64, Box<dyn Error>> {
            let mut celsius = 0.0;
            let result = unsafe { ak500_sdk_read(&mut celsius) };
            if result != 0 {
                return Err(format!(
                    "AMD Monitoring SDK temperature read failed (code {result}). No stale or fake temperature will be sent."
                ).into());
            }
            if !celsius.is_finite() || !(0.0..=125.0).contains(&celsius) {
                return Err("AMD Monitoring SDK returned an invalid temperature".into());
            }
            Ok(celsius)
        }
    }

    impl Drop for AmdTemperature {
        fn drop(&mut self) {
            unsafe { ak500_sdk_close() };
        }
    }

    #[repr(C)]
    #[derive(Clone, Copy, Default)]
    struct FileTime {
        low: u32,
        high: u32,
    }

    impl FileTime {
        fn as_u64(self) -> u64 {
            (u64::from(self.high) << 32) | u64::from(self.low)
        }
    }

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetSystemTimes(idle: *mut FileTime, kernel: *mut FileTime, user: *mut FileTime)
            -> i32;
    }

    #[derive(Clone, Copy)]
    struct CpuTimes {
        idle: u64,
        total: u64,
    }

    fn cpu_times() -> io::Result<CpuTimes> {
        let mut idle = FileTime::default();
        let mut kernel = FileTime::default();
        let mut user = FileTime::default();
        let result = unsafe { GetSystemTimes(&mut idle, &mut kernel, &mut user) };
        if result == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(CpuTimes {
            idle: idle.as_u64(),
            total: kernel.as_u64().saturating_add(user.as_u64()),
        })
    }

    fn cpu_usage(previous: CpuTimes, current: CpuTimes) -> Option<f64> {
        let total_delta = current.total.saturating_sub(previous.total);
        if total_delta == 0 {
            return None;
        }
        let idle_delta = current.idle.saturating_sub(previous.idle);
        let busy = total_delta.saturating_sub(idle_delta);
        Some((100.0 * busy as f64 / total_delta as f64).clamp(0.0, 100.0))
    }

    fn initialization_report() -> [u8; REPORT_LENGTH] {
        let mut packet = [0u8; REPORT_LENGTH];
        packet[0] = REPORT_ID;
        packet[1] = INITIALIZE;
        packet
    }

    fn usage_report(percentage: f64) -> [u8; REPORT_LENGTH] {
        let percentage = percentage.clamp(0.0, 100.0);
        let value = percentage.round() as u16;
        let mut packet = [0u8; REPORT_LENGTH];
        packet[0] = REPORT_ID;
        packet[1] = CPU_USAGE_MODE;
        packet[2] = if percentage < 15.0 {
            1
        } else {
            (percentage / 10.0).round().clamp(1.0, 10.0) as u8
        };
        packet[3] = (value / 100) as u8;
        packet[4] = ((value / 10) % 10) as u8;
        packet[5] = (value % 10) as u8;
        packet
    }

    fn temperature_report(celsius: f64) -> [u8; REPORT_LENGTH] {
        let temperature = celsius.clamp(0.0, 120.0).round() as u16;
        let mut packet = [0u8; REPORT_LENGTH];
        packet[0] = REPORT_ID;
        packet[1] = CPU_TEMPERATURE_CELSIUS_MODE;
        packet[2] = ((temperature as f64 / 9.0).round() as u8).min(10);
        packet[3] = (temperature / 100) as u8;
        packet[4] = ((temperature / 10) % 10) as u8;
        packet[5] = (temperature % 10) as u8;
        packet[6] = u8::from(temperature >= 90);
        packet
    }

    fn show_temperature(elapsed: Duration) -> bool {
        (elapsed.as_secs() / MODE_INTERVAL.as_secs()) % 2 == 0
    }

    fn send_report(device: &HidDevice, packet: &[u8; REPORT_LENGTH]) -> Result<(), Box<dyn Error>> {
        let written = device.write(packet)?;
        if written != REPORT_LENGTH {
            return Err(format!("Incomplete HID report: wrote {written} of {REPORT_LENGTH} bytes").into());
        }
        Ok(())
    }

    fn open_display(api: &HidApi) -> Result<HidDevice, Box<dyn Error>> {
        let mut failures = Vec::new();
        let mut found = false;
        for info in api.device_list() {
            if info.vendor_id() != VENDOR_ID || info.product_id() != AK500_DIGITAL_PRODUCT_ID {
                continue;
            }
            found = true;
            match info.open_device(api) {
                Ok(device) => match send_report(&device, &initialization_report()) {
                    Ok(()) => return Ok(device),
                    Err(error) => failures.push(error.to_string()),
                },
                Err(error) => failures.push(error.to_string()),
            }
        }

        if !found {
            return Err("AK500 DIGITAL not found (expected USB VID 3633, PID 0003)".into());
        }
        Err(format!(
            "AK500 DIGITAL detected but cannot be initialized: {}. Close DeepCool and other HID utilities.",
            failures.join("; ")
        ).into())
    }

    #[derive(Default)]
    struct Options {
        list: bool,
        once: bool,
        dry_run: bool,
        quiet: bool,
        test_temperature: Option<u16>,
    }

    fn usage_text() -> &'static str {
        "AK500 DIGITAL Lite 0.2 (Windows + AMD Monitoring SDK)\n\
         Modes: CPU temperature 5 seconds, CPU usage 5 seconds, repeated.\n\
         Both sensors are sampled every second.\n\
         Options:\n\
           --list              List detected DeepCool USB HID devices\n\
           --once              Read both metrics and send one temperature frame\n\
           --dry-run           Print readings without opening the USB display\n\
           --test-temp N       Send a STATIC test temperature (0-120 C) and exit\n\
           --quiet             Suppress per-update logs\n\
           --help              Print this help\n\
         Run as Administrator for access to the installed AMD Monitoring SDK."
    }

    fn parse_options() -> Result<Option<Options>, Box<dyn Error>> {
        let mut opts = Options::default();
        let mut args = std::env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--help" | "-h" => {
                    println!("{}", usage_text());
                    return Ok(None);
                }
                "--list" => opts.list = true,
                "--once" => opts.once = true,
                "--dry-run" => opts.dry_run = true,
                "--quiet" => opts.quiet = true,
                "--test-temp" => {
                    let value = args.next().ok_or("Missing value for --test-temp")?;
                    let parsed: u16 = value.parse()?;
                    if parsed > 120 {
                        return Err("--test-temp must be between 0 and 120".into());
                    }
                    opts.test_temperature = Some(parsed);
                }
                _ => return Err(format!("Unknown option: {arg}. Use --help.").into()),
            }
        }
        Ok(Some(opts))
    }

    fn list_deepcool(api: &HidApi) {
        let mut count = 0;
        for info in api.device_list().filter(|i| i.vendor_id() == VENDOR_ID) {
            println!(
                "VID={:04X} PID={:04X} UsagePage={:04X} Usage={:04X} Product={}",
                info.vendor_id(),
                info.product_id(),
                info.usage_page(),
                info.usage(),
                info.product_string().unwrap_or("Unknown")
            );
            count += 1;
        }
        if count == 0 {
            println!("No DeepCool HID devices found.");
        }
    }

    pub fn run() -> Result<(), Box<dyn Error>> {
        let Some(options) = parse_options()? else {
            return Ok(());
        };

        if options.list {
            list_deepcool(&HidApi::new()?);
            return Ok(());
        }

        if let Some(temperature) = options.test_temperature {
            println!("STATIC TEST: {temperature} C, not measured by AMD SDK.");
            if options.dry_run {
                println!("{:02X?}", &temperature_report(temperature as f64)[..8]);
                return Ok(());
            }
            let api = HidApi::new()?;
            let display = open_display(&api)?;
            send_report(&display, &temperature_report(temperature as f64))?;
            thread::sleep(Duration::from_secs(2));
            let previous = cpu_times()?;
            thread::sleep(Duration::from_millis(500));
            let percentage = cpu_usage(previous, cpu_times()?).unwrap_or(0.0);
            send_report(&display, &usage_report(percentage))?;
            println!("Static test finished; restored one CPU usage frame.");
            return Ok(());
        }

        let sdk = AmdTemperature::open()?;
        let api = if options.dry_run { None } else { Some(HidApi::new()?) };
        let display = match api.as_ref() {
            Some(api) => Some(open_display(api)?),
            None => None,
        };

        if options.dry_run {
            println!("Dry run: reading AMD temperature and Windows CPU usage; no USB writes.");
        } else {
            println!("AK500 DIGITAL connected. Updating both metrics every second; switching display every five seconds.");
        }

        let start = Instant::now();
        let mut next_sample = start + SAMPLE_INTERVAL;
        let mut previous = cpu_times()?;

        loop {
            thread::sleep(next_sample.saturating_duration_since(Instant::now()));
            let now = Instant::now();
            let current = cpu_times()?;
            let usage = cpu_usage(previous, current).ok_or("Unable to sample CPU usage")?;
            previous = current;

            // The AMD SDK must not be queried more than once per second.
            let temperature = sdk.read()?;
            let temperature_mode = show_temperature(now.duration_since(start));
            let report = if temperature_mode {
                temperature_report(temperature)
            } else {
                usage_report(usage)
            };

            if let Some(device) = display.as_ref() {
                send_report(device, &report)?;
            }

            if !options.quiet {
                println!(
                    "CPU {:.1} C | CPU usage {:.1}% | display: {}",
                    temperature,
                    usage,
                    if temperature_mode { "temperature" } else { "usage" }
                );
            }

            if options.once {
                break;
            }

            next_sample += SAMPLE_INTERVAL;
            if next_sample <= Instant::now() {
                next_sample = Instant::now() + SAMPLE_INTERVAL;
            }
        }
        Ok(())
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn initialization_packet_has_expected_bytes() {
            assert_eq!(&initialization_report()[..3], &[0x10, 0xAA, 0]);
        }

        #[test]
        fn usage_packet_has_expected_digits() {
            assert_eq!(&usage_report(57.0)[..7], &[0x10, 0x4C, 6, 0, 5, 7, 0]);
        }

        #[test]
        fn temperature_packet_has_expected_digits() {
            assert_eq!(&temperature_report(49.97)[..7], &[0x10, 0x13, 6, 0, 5, 0, 0]);
        }

        #[test]
        fn display_mode_changes_at_five_second_boundaries() {
            assert!(show_temperature(Duration::from_secs(0)));
            assert!(show_temperature(Duration::from_secs(4)));
            assert!(!show_temperature(Duration::from_secs(5)));
            assert!(!show_temperature(Duration::from_secs(9)));
            assert!(show_temperature(Duration::from_secs(10)));
        }

        #[test]
        fn cpu_delta_returns_expected_load() {
            let old = CpuTimes { idle: 400, total: 1000 };
            let new = CpuTimes { idle: 440, total: 1100 };
            assert_eq!(cpu_usage(old, new), Some(60.0));
        }
    }
}

#[cfg(windows)]
fn main() {
    if let Err(error) = app::run() {
        eprintln!("Error: {error}");
        std::process::exit(1);
    }
}
