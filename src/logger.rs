use fern::colors::{Color, ColoredLevelConfig};

#[derive(Debug)]
pub struct Logger {
    has_init: bool,
}

impl Logger {
    pub fn new() -> Self {
        Self { has_init: false }
    }

    pub fn init(&mut self) -> crate::TimersetResult<()> {
        if self.has_init {
            return Ok(());
        }

        fern::Dispatch::new()
            .chain(stdout()?)
            .chain(filelog()?)
            .apply()?;

        self.has_init = true;

        Ok(())
    }
}

fn stdout() -> crate::TimersetResult<fern::Dispatch> {
    let colors = ColoredLevelConfig::new()
        .error(Color::Red)
        .warn(Color::Yellow)
        .info(Color::White)
        .debug(Color::White)
        .trace(Color::BrightBlack)
        .info(Color::Green);

    let time_format = time::format_description::well_known::Rfc3339;

    let dispatcher = fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[{time}][{target}] {level} > {message}",
                time = time::OffsetDateTime::now_utc()
                    .format(&time_format)
                    .unwrap(),
                target = record.target(),
                level = colors.color(record.level()),
                message = message,
            ))
        })
        .level(log::LevelFilter::Info)
        .chain(std::io::stdout());

    Ok(dispatcher)
}

fn filelog() -> crate::TimersetResult<fern::Dispatch> {
    let mut logpath = std::env::current_exe()?;
    let _ = logpath.set_extension("log");

    let time_format = time::format_description::well_known::Rfc3339;

    let dispatcher = fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[{time}][{level}][{target}] > {message}",
                time = time::OffsetDateTime::now_utc()
                    .format(&time_format)
                    .unwrap(),
                level = record.level(),
                target = record.target(),
                message = message,
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(fern::log_file(logpath)?);

    Ok(dispatcher)
}
