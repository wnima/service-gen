use anyhow::{Result, anyhow};
use common::models::service_config::ServiceConfig;
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use windows_service::{
    define_windows_service,
    service::{
        ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
        ServiceType,
    },
    service_control_handler::{self, ServiceControlHandlerResult},
    service_dispatcher,
};

use common::utils::exec_config_utils::read_config_from_exe;
use common::utils::file_utils::get_exe_dir;
use common::utils::log_utils::{write_exec_log, write_wrapper_log};
use common::utils::gui_utils::show_error_box;

/// 解析命令行
fn parse_command_line(line: &str) -> Result<(String, Vec<String>)> {
    let mut parts = shell_words::split(line).map_err(|e| anyhow!("命令解析失败: {}", e))?;

    if parts.is_empty() {
        return Err(anyhow!("命令为空"));
    }

    let exe = parts.remove(0);
    Ok((exe, parts))
}

/// 启动子进程（非阻塞 + 可终止）
fn run_target_process(cfg: &ServiceConfig, stop_flag: Arc<AtomicBool>) -> Result<()> {
    write_wrapper_log("准备启动目标进程...");

    let cmd_line = cfg
        .executable
        .clone()
        .ok_or_else(|| anyhow!("未配置 executable"))?;

    let (exe, args) = parse_command_line(&cmd_line)?;
    write_wrapper_log(&format!("原始命令：{} {:?}", exe, args));

    let work_dir = if let Some(wd) = &cfg.working_directory {
        std::path::PathBuf::from(wd)
    } else {
        get_exe_dir()?
    };
    write_wrapper_log(&format!("工作目录：{}", work_dir.display()));

    let exe_path_buf = std::path::PathBuf::from(&exe);
    let final_exe: std::path::PathBuf;

    if exe_path_buf.is_absolute() {
        final_exe = exe_path_buf;
        write_wrapper_log(&format!("检测到绝对路径：{}", final_exe.display()));
    } else if exe.contains('.') == false
        && exe.contains('/') == false
        && exe.contains('\\') == false
    {
        final_exe = exe_path_buf;
        write_wrapper_log(&format!("检测到系统PATH命令：{}", exe));
    } else {
        final_exe = work_dir.join(&exe);
        write_wrapper_log(&format!("相对路径 → 绝对路径：{}", final_exe.display()));
    }

    let mut command = Command::new(final_exe);
    command.args(args);
    command.current_dir(&work_dir);
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());

    let mut child = command.spawn()?;
    let pid = child.id();
    write_wrapper_log(&format!("进程已启动，PID={}", pid));

    // 日志线程：非阻塞读取输出
    let log_mode = cfg.log_mode.clone();
    let mut stdout = child.stdout.take().unwrap();
    let mut stderr = child.stderr.take().unwrap();

    thread::spawn(move || {
        use std::io::Read;
        let mut buf = [0; 4096];
        loop {
            match stdout.read(&mut buf) {
                Ok(0) | Err(_) => break,
                Ok(n) => {
                    let s = String::from_utf8_lossy(&buf[..n]);
                    for line in s.lines() {
                        write_exec_log(&log_mode, line);
                    }
                }
            }
        }
    });

    let log_mode_err = cfg.log_mode.clone();
    thread::spawn(move || {
        use std::io::Read;
        let mut buf = [0; 4096];
        loop {
            match stderr.read(&mut buf) {
                Ok(0) | Err(_) => break,
                Ok(n) => {
                    let s = String::from_utf8_lossy(&buf[..n]);
                    for line in s.lines() {
                        let err_line = format!("ERROR: {}", line);
                        write_exec_log(&log_mode_err, &err_line);
                        write_wrapper_log(&format!("子进程错误：{}", line));
                    }
                }
            }
        }
    });

    // 等待子进程，但会监听停止信号
    loop {
        if stop_flag.load(Ordering::SeqCst) {
            write_wrapper_log("收到停止信号，强制终止子进程");
            let _ = child.kill();
            break;
        }
        match child.try_wait() {
            Ok(Some(status)) => {
                write_wrapper_log(&format!("子进程正常退出：{}", status));
                break;
            }
            Ok(None) => {
                thread::sleep(std::time::Duration::from_millis(100));
            }
            Err(e) => {
                write_wrapper_log(&format!("子进程异常：{}", e));
                break;
            }
        }
    }

    Ok(())
}

/// 服务主逻辑
pub fn run_as_service() -> Result<()> {
    write_wrapper_log("===== 服务模式启动 =====");

    let config_result = read_config_from_exe();
    let config = match config_result {
        Ok(cfg) => cfg,
        Err(e) => {
            show_error_box("启动失败", &format!("读取配置失败: {}", e));
            return Err(e);
        }
    };
    let service_id = config.service_id.clone();

    define_windows_service!(ffi_service_main, service_main);

    fn service_main(_args: Vec<std::ffi::OsString>) {
        let config_result = read_config_from_exe();
        let cfg = match config_result {
            Ok(cfg) => cfg,
            Err(e) => {
                show_error_box("启动失败", &format!("读取配置失败: {}", e));
                return;
            }
        };

        write_wrapper_log("服务初始化完成，准备启动...");

        let stop_flag = Arc::new(AtomicBool::new(false));
        let stop_clone = stop_flag.clone();

        let status_handle =
            match service_control_handler::register(&cfg.service_id, move |control| {
                if matches!(control, ServiceControl::Stop) {
                    write_wrapper_log("收到停止指令");
                    stop_clone.store(true, Ordering::SeqCst);
                }
                ServiceControlHandlerResult::NoError
            }) {
                Ok(h) => h,
                Err(e) => {
                    write_wrapper_log(&format!("注册服务控制失败：{}", e));
                    return;
                }
            };

        let _ = status_handle.set_service_status(ServiceStatus {
            service_type: ServiceType::OWN_PROCESS,
            current_state: ServiceState::Running,
            controls_accepted: ServiceControlAccept::STOP,
            exit_code: ServiceExitCode::Win32(0),
            checkpoint: 0,
            wait_hint: std::time::Duration::from_secs(5),
            process_id: None,
        });

        write_wrapper_log("服务已标记为运行中");

        // 主循环
        while !stop_flag.load(Ordering::SeqCst) {
            write_wrapper_log("开始启动目标进程");

            if let Err(e) = run_target_process(&cfg, stop_flag.clone()) {
                write_wrapper_log(&format!("目标进程异常：{}", e));
            }

            if stop_flag.load(Ordering::SeqCst) {
                break;
            }

            write_wrapper_log(&format!("{} 秒后重启", cfg.on_failure_delay));
            thread::sleep(std::time::Duration::from_secs(cfg.on_failure_delay as u64));
        }

        let _ = status_handle.set_service_status(ServiceStatus {
            service_type: ServiceType::OWN_PROCESS,
            current_state: ServiceState::Stopped,
            controls_accepted: ServiceControlAccept::empty(),
            exit_code: ServiceExitCode::Win32(0),
            checkpoint: 0,
            wait_hint: std::time::Duration::default(),
            process_id: None,
        });

        write_wrapper_log("服务已正常停止");
    }

    if let Err(e) = service_dispatcher::start(&service_id, ffi_service_main) {
        write_wrapper_log(&format!("服务调度器启动失败：{}", e));
    }

    Ok(())
}
