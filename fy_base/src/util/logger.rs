use std::result::Result;
use std::str::FromStr;

use log::{LevelFilter, ParseLevelError};
use log4rs::config::Logger;
use log4rs::{
    append::{
        console::{ConsoleAppender, Target},
        file::FileAppender,
    },
    config::{Appender, Config, Root},
    encode::pattern::PatternEncoder,
    filter::threshold::ThresholdFilter,
};

/**
2020-10-12T10:39:23.184575+08:00 WARN bm_worker - it's warn
2020/10/12 11:22:09.444 [DEBU] -
 */
#[derive(Debug)]
pub struct InitLoggerErr;

impl From<log::ParseLevelError> for InitLoggerErr {
    fn from(_: ParseLevelError) -> Self {
        InitLoggerErr
    }
}

pub fn init_logger_str(path: &str, level: &str) -> Result<(), InitLoggerErr> {
    init_logger(path, LevelFilter::from_str(level)?)
}

pub fn init_logger(path: &str, level_filter: LevelFilter) -> Result<(), InitLoggerErr> {
    let pattern = "{d(%Y/%m/%d %H:%M:%S%.3f)} {l} - {m}{n}";

    let stdout = ConsoleAppender::builder()
        .target(Target::Stdout)
        .encoder(Box::new(PatternEncoder::new(pattern)))
        .build();

    let stderr = ConsoleAppender::builder()
        .target(Target::Stderr)
        .encoder(Box::new(PatternEncoder::new(pattern)))
        .build();

    let file_append = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(pattern)))
        .build(path);

    let file_append = match file_append {
        Ok(v) => v,
        Err(_) => {
            return Err(InitLoggerErr);
        }
    };

    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .appender(
            Appender::builder()
                .filter(Box::new(ThresholdFilter::new(LevelFilter::Error)))
                .build("stderr", Box::new(stderr)),
        )
        .appender(Appender::builder().build("file", Box::new(file_append)))
        .build(
            Root::builder()
                .appenders(vec!["stdout", "stderr", "file"])
                .build(level_filter),
        );

    let config = match config {
        Ok(v) => v,
        Err(_) => {
            return Err(InitLoggerErr);
        }
    };

    match log4rs::init_config(config) {
        Ok(_) => Ok(()),
        Err(_) => Err(InitLoggerErr),
    }
}

//---------------------------------------
pub fn init_console_logger_str(level: &str) -> Result<(), InitLoggerErr> {
    init_console_logger(LevelFilter::from_str(level)?)
}

pub fn init_console_logger(level_filter: LevelFilter) -> Result<(), InitLoggerErr> {
    let pattern = "{d(%Y/%m/%d %H:%M:%S%.3f)} {l} - {m}{n}";

    let stdout = ConsoleAppender::builder()
        .target(Target::Stdout)
        .encoder(Box::new(PatternEncoder::new(pattern)))
        .build();

    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(
            Root::builder()
                .appenders(vec!["stdout"])
                .build(level_filter),
        );

    let config = match config {
        Ok(v) => v,
        Err(_) => {
            return Err(InitLoggerErr);
        }
    };

    match log4rs::init_config(config) {
        Ok(_) => Ok(()),
        Err(_) => Err(InitLoggerErr),
    }
}

//---------------------------------------

pub fn init_app_logger_str(
    path: &str,
    app_target: &str,
    app_level: &str,
    depends_level: &str,
) -> Result<(), InitLoggerErr> {
    init_app_logger(
        path,
        app_target,
        LevelFilter::from_str(app_level)?,
        LevelFilter::from_str(depends_level)?,
    )
}

pub fn init_app_logger_with_web_str(
    app_log_path: &str,
    app_target: &str,
    app_level: &str,
    depends_level: &str,
    web_log_path: &str,
    web_target: &str,
) -> Result<(), InitLoggerErr> {
    init_app_logger_with_web(
        app_log_path,
        app_target,
        LevelFilter::from_str(app_level)?,
        LevelFilter::from_str(depends_level)?,
        web_log_path,
        web_target,
    )
}

//--------------------------------------------------------------------------
pub fn init_app_logger_with_web(
    app_log_path: &str,
    app_target: &str,
    app_level: LevelFilter,
    depends_level: LevelFilter,
    web_log_path: &str,
    web_target: &str,
) -> Result<(), InitLoggerErr> {
    let pattern = "{d(%Y/%m/%d %H:%M:%S%.3f)} {M} {l} - {m}{n}";
    let pattern_web = "{m}{n}";

    let stdout = ConsoleAppender::builder()
        .target(Target::Stdout)
        .encoder(Box::new(PatternEncoder::new(pattern)))
        .build();

    // 标准错误输出
    let stderr = ConsoleAppender::builder()
        .target(Target::Stderr)
        .encoder(Box::new(PatternEncoder::new(pattern)))
        .build();

    let file_append = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(pattern)))
        .build(app_log_path);

    let file_append = match file_append {
        Ok(v) => v,
        Err(_) => {
            return Err(InitLoggerErr);
        }
    };

    let web_append = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(pattern_web)))
        .build(web_log_path);

    let web_append = match web_append {
        Ok(v) => v,
        Err(_) => {
            return Err(InitLoggerErr);
        }
    };

    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .appender(
            Appender::builder()
                // 输出到标准错误的日志，需要Error级别
                .filter(Box::new(ThresholdFilter::new(LevelFilter::Error)))
                .build("stderr", Box::new(stderr)),
        )
        .appender(Appender::builder().build("file", Box::new(file_append)))
        .appender(Appender::builder().build("web", Box::new(web_append)))
        // app log, 继承root的logger
        .logger(Logger::builder().build(app_target, app_level))
        // web access log，不需要从root继承 logger
        .logger(
            Logger::builder()
                .additive(false)
                .appender("web")
                .build(web_target, app_level),
        )
        .build(
            Root::builder()
                .appenders(vec!["stdout", "stderr", "file"])
                .build(depends_level),
        );

    let config = match config {
        Ok(v) => v,
        Err(_) => {
            return Err(InitLoggerErr);
        }
    };

    match log4rs::init_config(config) {
        Ok(_) => Ok(()),
        Err(_) => Err(InitLoggerErr),
    }
}

//--------------------------------------------------------------------------
pub fn init_app_logger(
    app_log_path: &str,
    app_target: &str,
    app_level: LevelFilter,
    depends_level: LevelFilter,
) -> Result<(), InitLoggerErr> {
    let pattern = "{d(%Y/%m/%d %H:%M:%S%.3f)} {M} {l} - {m}{n}";

    let stdout = ConsoleAppender::builder()
        .target(Target::Stdout)
        .encoder(Box::new(PatternEncoder::new(pattern)))
        .build();

    // 标准错误输出
    let stderr = ConsoleAppender::builder()
        .target(Target::Stderr)
        .encoder(Box::new(PatternEncoder::new(pattern)))
        .build();

    let file_append = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(pattern)))
        .build(app_log_path);

    let file_append = match file_append {
        Ok(v) => v,
        Err(_) => {
            return Err(InitLoggerErr);
        }
    };

    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .appender(
            Appender::builder()
                // 输出到标准错误的日志，需要Error级别
                .filter(Box::new(ThresholdFilter::new(LevelFilter::Error)))
                .build("stderr", Box::new(stderr)),
        )
        .appender(Appender::builder().build("file", Box::new(file_append)))
        // app log, 继承root的logger
        .logger(Logger::builder().build(app_target, app_level))
        .build(
            Root::builder()
                .appenders(vec!["stdout", "stderr", "file"])
                .build(depends_level),
        );

    let config = match config {
        Ok(v) => v,
        Err(_) => {
            return Err(InitLoggerErr);
        }
    };

    match log4rs::init_config(config) {
        Ok(_) => Ok(()),
        Err(_) => Err(InitLoggerErr),
    }
}
