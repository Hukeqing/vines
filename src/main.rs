use crate::core::service::Service;
use crate::core::Config;
use crate::feature::web;
use colored::Colorize;
use env_logger::Builder;
use std::collections::VecDeque;
use std::io::Write;
use std::{env, thread};

mod feature;
mod core;
mod common;

fn main() {
    init_log();

    start_service();
}

fn start_service() {
    let home = common::file::DirNode::ROOT().next(String::from("Users")).next(String::from("shiroha")).next(String::from("data"));
    let config = Config::new(home, 4096);
    let service = Service::new(config);

    let mut inits = VecDeque::from(vec![web::init]);
    let mut threads = Vec::new();
    while let Some(init) = inits.pop_front() {
        let tmp_service = service.clone();
        let web_thread = thread::spawn(move || init(tmp_service));
        threads.push(web_thread);
    }
    for thread in threads {
        let _ = thread.join().expect("Thread panicked");
    }
}

fn init_log() {
    // 初始化带颜色的日志
    env::set_var("RUST_LOG", "info"); // 设置日志级别
    Builder::from_default_env()
        .format(|buf, record| {
            let level = match record.level() {
                log::Level::Error => "ERROR".red().bold(),
                log::Level::Warn => "WARN".yellow().bold(),
                log::Level::Info => "INFO".green().bold(),
                log::Level::Debug => "DEBUG".blue().bold(),
                log::Level::Trace => "TRACE".purple().bold(),
            };
            writeln!(buf, "{} [{}] - {}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                     level, record.args())
        })
        .init();
}