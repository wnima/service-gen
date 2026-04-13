use std::fs;
use std::io::Write;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Sender};
use std::thread;

use crate::utils::file_utils;

// ==============================
// 全局异步日志队列
// ==============================
static LOG_CHANNEL: OnceLock<(Sender<LogTask>, thread::JoinHandle<()>)> = OnceLock::new();
static LOG_STARTED: AtomicBool = AtomicBool::new(false);

/// 日志任务
#[derive(Debug)]
enum LogTask {
    Wrapper(String),    // 已经拼好时间戳 + 头的整行
    Exec(String, String), // (日志内容, log_mode)
}

/// 初始化日志后台线程
fn init_log_system() {
    if LOG_STARTED.swap(true, Ordering::SeqCst) {
        return;
    }

    let (tx, rx) = channel::<LogTask>();

    // 后台唯一单线程写文件，绝对不会乱行
    let worker = thread::spawn(move || {
        while let Ok(task) = rx.recv() {
            match task {
                LogTask::Wrapper(line) => {
                    let _ = write_wrapper_line(&line);
                }
                LogTask::Exec(msg, log_mode) => {
                    let _ = write_exec_line(&msg, &log_mode);
                }
            }
        }
    });

    let _ = LOG_CHANNEL.set((tx, worker));
}

/// 写入 wrapper 日志
fn write_wrapper_line(line: &str) -> std::io::Result<()> {
    let exe_dir = file_utils::get_exe_dir().unwrap();
    fs::create_dir_all(&exe_dir)?;
    let path = exe_dir.join("wrapper.log");

    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .write(true)
        .open(path)?;

    writeln!(file, "{}", line)?;
    file.flush()?;
    Ok(())
}

/// 写入 exec 日志
fn write_exec_line(msg: &str, log_mode: &str) -> std::io::Result<()> {
    let exe_dir = file_utils::get_exe_dir().expect("获取可执行文件路径失败！！！");
    fs::create_dir_all(&exe_dir)?;

    let path = match log_mode {
        "rotate" => {
            let date = chrono::Local::now().format("%Y-%m-%d");
            exe_dir.join(format!("exec_{}.log", date))
        }
        _ => exe_dir.join("exec.log"),
    };

    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .write(true)
        .open(path)?;

    writeln!(file, "{}", msg)?;
    file.flush()?;
    Ok(())
}

/// 包装器日志
pub fn write_wrapper_log(msg: &str) {
    init_log_system();

    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
    let line = format!("[{}] WRAPPER | {}", now, msg);

    if let Some((tx, _)) = LOG_CHANNEL.get() {
        let _ = tx.send(LogTask::Wrapper(line));
    }
}

/// 执行程序日志
pub fn write_exec_log(log_mode: &str, msg: &str) {
    if log_mode.eq_ignore_ascii_case("none") {
        return;
    }

    init_log_system();

    if let Some((tx, _)) = LOG_CHANNEL.get() {
        let _ = tx.send(LogTask::Exec(msg.to_string(), log_mode.to_string()));
    }
}