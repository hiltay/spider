use chrono::{DateTime, Local, TimeZone};
use data_structures::config;
pub use serde_yaml::Value;
use std::fs::File;
use std::io::{self};
/// 将不标准的字符串转换为统一格式的字符串
pub fn strptime_to_string<Tz: TimeZone>(strptime: DateTime<Tz>) -> String {
    format!("{}", strptime.fixed_offset().format("%Y-%m-%d %H:%M:%S"))
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
