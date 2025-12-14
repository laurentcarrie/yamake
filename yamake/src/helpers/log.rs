use std::time::SystemTime;

pub fn setup_logger() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}:{} {} {} {}] {}",
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                humantime::format_rfc3339_seconds(SystemTime::now()),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout())
        .chain(fern::log_file("yamake.log")?)
        .apply()?;
    Ok(())
}
