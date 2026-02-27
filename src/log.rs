use std::fs::OpenOptions;
use std::io::Write;
use std::{env, panic};

pub fn setup_panic_hook() {
    panic::set_hook(Box::new(|panic_info| {
        let mut log_path = env::current_exe()
            .map(|p| p.parent().unwrap_or(&p).to_path_buf())
            .unwrap_or_else(|_| env::current_dir().unwrap_or_default());

        log_path.push("crash_report.log");

        let message = panic_info
            .payload()
            .downcast_ref::<&str>()
            .map(|s| s.to_string())
            .or_else(|| {
                panic_info
                    .payload()
                    .downcast_ref::<String>()
                    .map(|s| s.clone())
            })
            .unwrap_or_else(|| "Unknown panic".to_string());

        let location = panic_info
            .location()
            .map(|l| format!("at {}:{}", l.file(), l.line()))
            .unwrap_or_default();

        let log_msg = format!(
            "--- Application Crash ---\nTime: {}\nMessage: {}\nLocation: {}\n\n",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            message,
            location
        );

        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(log_path) {
            let _ = writeln!(file, "{}", log_msg);
        }
    }));
}
