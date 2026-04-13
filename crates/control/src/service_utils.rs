use common::models::service_config::ServiceConfig;
use common::utils::log_utils::write_wrapper_log;
use std::error::Error;
use windows::{
    core::{PCWSTR, PWSTR},
    Win32::Security::SC_HANDLE,
    Win32::System::Services::{
        CloseServiceHandle, ControlService, CreateServiceW, DeleteService, OpenSCManagerW,
        OpenServiceW, QueryServiceStatus, StartServiceW, ChangeServiceConfig2W,
        SERVICE_CONTROL_STOP, SERVICE_QUERY_STATUS, SERVICE_STATUS, SERVICE_WIN32_OWN_PROCESS,
        SERVICE_AUTO_START, SERVICE_DEMAND_START, SERVICE_CONFIG_DESCRIPTION, SERVICE_DESCRIPTIONW,
        SC_MANAGER_ALL_ACCESS, SERVICE_ERROR_NORMAL,
    },
};

/// 字符串转 Windows 宽字符
fn to_pcwstr(s: &str) -> Vec<u16> {
    use std::iter::once;
    s.encode_utf16().chain(once(0)).collect()
}

/// 安装服务
pub fn install_service(cfg: &ServiceConfig) {
    unsafe {
        let sc_manager = match OpenSCManagerW(None, None, SC_MANAGER_ALL_ACCESS) {
            Ok(handle) if !handle.is_invalid() => handle,
            _ => {
                write_wrapper_log("错误：无法打开服务控制器，请以管理员身份运行");
                return;
            }
        };

        let exe_path = match std::env::current_exe() {
            Ok(p) => format!("{} -service", p.to_string_lossy()),
            Err(_) => {
                write_wrapper_log("错误：获取程序路径失败");
                let _ = CloseServiceHandle(sc_manager);
                return;
            }
        };

        let start_type = if cfg.start_mode.eq_ignore_ascii_case("automatic") {
            SERVICE_AUTO_START
        } else {
            SERVICE_DEMAND_START
        };

        let service_id = to_pcwstr(&cfg.service_id);
        let service_name = to_pcwstr(&cfg.service_name);
        let bin_path = to_pcwstr(&exe_path);

        let service = match CreateServiceW(
            sc_manager,
            PCWSTR(service_id.as_ptr()),
            PCWSTR(service_name.as_ptr()),
            0xF01FF,
            SERVICE_WIN32_OWN_PROCESS,
            start_type,
            SERVICE_ERROR_NORMAL,
            PCWSTR(bin_path.as_ptr()),
            None,
            None,
            None,
            None,
            None,
        ) {
            Ok(handle) if !handle.is_invalid() => handle,
            _ => {
                write_wrapper_log(&format!("服务 {} 创建失败", cfg.service_id));
                let _ = CloseServiceHandle(sc_manager);
                return;
            }
        };

        if let Some(desc) = &cfg.description {
            let _ = set_service_description(service, desc);
        }

        write_wrapper_log(&format!("服务 {} 安装成功", cfg.service_id));

        let _ = CloseServiceHandle(service);
        let _ = CloseServiceHandle(sc_manager);
    }
}

/// 设置服务描述
fn set_service_description(service: SC_HANDLE, desc: &str) -> windows::core::Result<()> {
    unsafe {
        let desc_w = to_pcwstr(desc);
        let data = SERVICE_DESCRIPTIONW {
            lpDescription: PWSTR(desc_w.as_ptr() as _),
        };

        ChangeServiceConfig2W(
            service,
            SERVICE_CONFIG_DESCRIPTION,
            Some(&data as *const _ as _),
        )
    }
}

/// 卸载服务
pub fn uninstall_service(cfg: &ServiceConfig) {
    stop_service(cfg);

    unsafe {
        let sc_manager = match OpenSCManagerW(None, None, SC_MANAGER_ALL_ACCESS) {
            Ok(h) if !h.is_invalid() => h,
            _ => return,
        };

        let service_id = to_pcwstr(&cfg.service_id);
        if let Ok(service) = OpenServiceW(sc_manager, PCWSTR(service_id.as_ptr()), 0x10001) {
            if !service.is_invalid() {
                let _ = DeleteService(service);
                let _ = CloseServiceHandle(service);
                write_wrapper_log(&format!("服务 {} 已卸载", cfg.service_id));
            }
        }

        let _ = CloseServiceHandle(sc_manager);
    }
}

/// 启动服务
pub fn start_service(cfg: &ServiceConfig) {
    unsafe {
        let sc_manager = match OpenSCManagerW(None, None, 0x1) {
            Ok(h) if !h.is_invalid() => h,
            _ => return,
        };

        let service_id = to_pcwstr(&cfg.service_id);
        if let Ok(service) = OpenServiceW(sc_manager, PCWSTR(service_id.as_ptr()), 0x10) {
            if !service.is_invalid() {
                let _ = StartServiceW(service, None);
                write_wrapper_log(&format!("服务 {} 已启动", cfg.service_id));
                let _ = CloseServiceHandle(service);
            }
        }

        let _ = CloseServiceHandle(sc_manager);
    }
}

/// 停止服务
pub fn stop_service(cfg: &ServiceConfig) {
    unsafe {
        let sc_manager = match OpenSCManagerW(None, None, 0x1) {
            Ok(h) if !h.is_invalid() => h,
            _ => return,
        };

        let service_id = to_pcwstr(&cfg.service_id);
        if let Ok(service) = OpenServiceW(sc_manager, PCWSTR(service_id.as_ptr()), 0x20) {
            if !service.is_invalid() {
                let mut status = SERVICE_STATUS::default();
                let _ = ControlService(service, SERVICE_CONTROL_STOP, &mut status);
                write_wrapper_log(&format!("服务 {} 已停止", cfg.service_id));
                let _ = CloseServiceHandle(service);
            }
        }

        let _ = CloseServiceHandle(sc_manager);
    }
}

/// 查询服务状态
pub fn query_service_status(id: &str) -> Result<String, Box<dyn Error>> {
    unsafe {
        let sc_manager = match OpenSCManagerW(None, None, 0x1) {
            Ok(h) if !h.is_invalid() => h,
            _ => return Ok("未安装".into()),
        };

        let service_id = to_pcwstr(id);
        let _service = OpenServiceW(sc_manager, PCWSTR(service_id.as_ptr()), SERVICE_QUERY_STATUS);
        
        match _service {
            Ok(service) if !service.is_invalid() => {
                let mut status = SERVICE_STATUS::default();
                let _ = QueryServiceStatus(service, &mut status);

                let state = match status.dwCurrentState.0 {
                    1 => "已停止",
                    2 => "启动中",
                    3 => "停止中",
                    4 => "运行中",
                    5 => "恢复中",
                    6 => "暂停中",
                    7 => "已暂停",
                    _ => "未知",
                };

                let _ = CloseServiceHandle(service);
                let _ = CloseServiceHandle(sc_manager);

                Ok(state.to_string())
            }
            _ => {
                let _ = CloseServiceHandle(sc_manager);
                Ok("未安装".into())
            }
        }
    }
}