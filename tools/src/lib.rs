use chrono::{DateTime, FixedOffset, NaiveDateTime, TimeZone, Utc};
use data_structures::config;
use dotenvy;
pub use serde_yaml::Value;
use std::fs::File;
use std::io::{self};
/// 将时间结构转换为统一格式的字符串`%Y-%m-%d %H:%M:%S`，带时分秒
pub fn strptime_to_string_ymdhms<Tz: TimeZone>(strptime: DateTime<Tz>) -> String {
    strptime
        .fixed_offset()
        .format("%Y-%m-%d %H:%M:%S")
        .to_string()
}

/// 将时间结构转换为统一格式的字符串`%Y-%m-%d`，不带时分秒
pub fn strptime_to_string_ymd<Tz: TimeZone>(strptime: DateTime<Tz>) -> String {
    strptime.fixed_offset().format("%Y-%m-%d").to_string()
}

/// 将可能不标准的时间字符串转换为统一格式的字符串`%Y-%m-%d`，不带时分秒
pub fn strftime_to_string_ymd(strftime: &str) -> String {
    let fmts = [
        "%Y-%m-%d",
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%d%H:%M:%S",
        "%Y-%m-%dT%H:%M:%S.000Z", // 2021-11-12T01:24:06.000Z
        "%Y年%m月%d日",           // xxxx年xx月xx日
    ];
    for fmt in fmts {
        if let Ok(v) = NaiveDateTime::parse_from_str(strftime, fmt) {
            return v.format("%Y-%m-%d").to_string();
        };
    }
    strptime_to_string_ymd(Utc::now().with_timezone(&FixedOffset::east_opt(8 * 60 * 60).unwrap()))
}

pub fn get_yaml(path: &str) -> io::Result<Value> {
    let config_file = File::open(path)?;
    match serde_yaml::from_reader(config_file) {
        Ok(config) => Ok(config),
        Err(err) => panic!("{}", err),
    }
}

pub fn get_yaml_settings(path: &str) -> io::Result<config::Settings> {
    let config_file = File::open(path)?;
    match serde_yaml::from_reader(config_file) {
        Ok(config) => Ok(config),
        Err(err) => panic!("{}", err),
    }
}

/// 获取环境变量中的mysql连接
pub fn load_mysql_conn_env() -> Result<String, Box<dyn std::error::Error>> {
    let r = dotenvy::dotenv()?;
    println!("{:?}",r);
    Ok(dotenvy::var("MYSQL_URI")?)
}

/// 获取环境变量中的代理配置
pub fn load_proxy_env() -> Result<String, Box<dyn std::error::Error>> {
    let r = dotenvy::dotenv()?;
    println!("{:?}",r);
    Ok(dotenvy::var("PROXY")?)
}