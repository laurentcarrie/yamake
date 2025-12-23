use env_logger::Builder;
use log::debug;
use log::info;
use std::io::Write;
use std::time::SystemTime;

pub fn setup_logger2(_with_time: bool, level: log::LevelFilter) -> Result<(), fern::InitError> {
    // simple_logging::log_to_file("yamake.log", level).unwrap_or(());
    let mut builder = Builder::from_default_env();

    let _ = builder
        .format(|buf, record| {
            writeln!(
                buf,
                "{}:{} - {} - CCCCCCC {}",
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                record.level(),
                record.args()
            )
        })
        .write_style(env_logger::WriteStyle::Always)
        .format_target(true)
        .format_file(true)
        .format_line_number(true)
        .filter_level(level)
        .is_test(true)
        .try_init();
    debug!("Logger setup at level {:?}", level);
    info!("Logger initialized");

    // let ret = match with_time {
    //     true => fern::Dispatch::new()
    //         .format(|out, message, record| {
    //             out.finish(format_args!(
    //                 "[{}:{} {} {} {}] {}",
    //                 record.file().unwrap_or("unknown"),
    //                 record.line().unwrap_or(0),
    //                 humantime::format_rfc3339_seconds(SystemTime::now()),
    //                 record.level(),
    //                 record.target(),
    //                 message
    //             ))
    //         })
    //         .level(level)
    //         .chain(std::io::stdout())
    //         .chain(fern::log_file("yamake.log")?)
    //         .apply()?,
    //     false => fern::Dispatch::new()
    //         .format(|out, message, record| {
    //             out.finish(format_args!(
    //                 "{}:{} {} {}",
    //                 record.file().unwrap_or("unknown"),
    //                 record.line().unwrap_or(0),
    //                 record.level(),
    //                 message
    //             ))
    //         })
    //         .level(level)
    //         .chain(std::io::stdout())
    //         .chain(fern::log_file("yamake.log")?)
    //         .apply()?,
    // };
    // dbg!(&ret);
    Ok(())
}

pub fn setup_logger(with_time: bool, level: log::LevelFilter) -> Result<(), fern::InitError> {
    let ret = match with_time {
        true => fern::Dispatch::new()
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
            .level(level)
            .chain(std::io::stdout())
            .chain(fern::log_file("yamake.log")?)
            .apply(),
        false => fern::Dispatch::new()
            .format(|out, message, record| {
                out.finish(format_args!(
                    "{}:{} {} {}",
                    record.file().unwrap_or("unknown"),
                    record.line().unwrap_or(0),
                    record.level(),
                    message
                ))
            })
            .level(level)
            .chain(std::io::stdout())
            .chain(fern::log_file("yamake.log")?)
            .apply(),
    };
    let ret = ret.unwrap_or(());
    dbg!(&ret);
    Ok(())
}
