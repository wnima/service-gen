use anyhow::Result;
use common::models::service_config::ServiceConfig;
use std::thread;
use winapi::shared::minwindef::{FALSE, LPARAM, LRESULT, TRUE, UINT, WPARAM};
use winapi::shared::windef::{HMENU, HWND};
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::winuser::*;
use winapi::um::winuser::{COLOR_BTNFACE, GetSysColorBrush};

use crate::service_utils::{
    install_service, query_service_status, start_service, stop_service, uninstall_service,
};
use common::utils::exec_config_utils::read_config_from_exe;
use common::utils::log_utils::write_wrapper_log;

use common::utils::gui_utils::show_error_box;
use common::utils::gui_utils::to_wstring;

// ==============================
// GUI 部分
// ==============================
const ID_INSTALL: i32 = 101;
const ID_UNINSTALL: i32 = 102;
const ID_START: i32 = 103;
const ID_STOP: i32 = 104;
const ID_STATUS: i32 = 201;


/// 窗口过程
unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: UINT,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe {
        match msg {
            WM_CREATE => {
                let cs = &*(lparam as *const CREATESTRUCTW);
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, cs.lpCreateParams as isize);
                let cfg = &*(cs.lpCreateParams as *const ServiceConfig);

                // 创建控件
                CreateWindowExW(
                    0,
                    to_wstring("STATIC").as_ptr(),
                    to_wstring(&format!("服务: {}", cfg.service_id)).as_ptr(),
                    WS_CHILD | WS_VISIBLE,
                    20,
                    20,
                    360,
                    20,
                    hwnd,
                    std::ptr::null_mut(),
                    GetModuleHandleW(std::ptr::null()),
                    std::ptr::null_mut(),
                );
                CreateWindowExW(
                    0,
                    to_wstring("BUTTON").as_ptr(),
                    to_wstring("安装").as_ptr(),
                    WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON,
                    20,
                    60,
                    160,
                    40,
                    hwnd,
                    ID_INSTALL as HMENU,
                    GetModuleHandleW(std::ptr::null()),
                    std::ptr::null_mut(),
                );
                CreateWindowExW(
                    0,
                    to_wstring("BUTTON").as_ptr(),
                    to_wstring("卸载").as_ptr(),
                    WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON,
                    220,
                    60,
                    160,
                    40,
                    hwnd,
                    ID_UNINSTALL as HMENU,
                    GetModuleHandleW(std::ptr::null()),
                    std::ptr::null_mut(),
                );
                CreateWindowExW(
                    0,
                    to_wstring("BUTTON").as_ptr(),
                    to_wstring("启动").as_ptr(),
                    WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON,
                    20,
                    120,
                    160,
                    40,
                    hwnd,
                    ID_START as HMENU,
                    GetModuleHandleW(std::ptr::null()),
                    std::ptr::null_mut(),
                );
                CreateWindowExW(
                    0,
                    to_wstring("BUTTON").as_ptr(),
                    to_wstring("停止").as_ptr(),
                    WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON,
                    220,
                    120,
                    160,
                    40,
                    hwnd,
                    ID_STOP as HMENU,
                    GetModuleHandleW(std::ptr::null()),
                    std::ptr::null_mut(),
                );
                CreateWindowExW(
                    0,
                    to_wstring("STATIC").as_ptr(),
                    to_wstring("状态: 查询中...").as_ptr(),
                    WS_CHILD | WS_VISIBLE | SS_LEFT,
                    20,
                    180,
                    360,
                    40,
                    hwnd,
                    ID_STATUS as HMENU,
                    GetModuleHandleW(std::ptr::null()),
                    std::ptr::null_mut(),
                );

                // 立即刷新一次
                update_status_ui(hwnd, cfg);

                0
            }

            WM_COMMAND => {
                let id = (wparam & 0xFFFF) as i32;
                let cfg = get_config(hwnd).unwrap();
                thread::spawn(move || match id {
                    ID_INSTALL => install_service(&cfg),
                    ID_UNINSTALL => uninstall_service(&cfg),
                    ID_START => start_service(&cfg),
                    ID_STOP => stop_service(&cfg),
                    _ => {}
                });
                // 立即刷新一次
                update_status_ui(hwnd, cfg);
                0
            }

            WM_TIMER => {
                let cfg = get_config(hwnd).unwrap();
                update_status_ui(hwnd, cfg);
                0
            }

            WM_DESTROY => {
                PostQuitMessage(0);
                0
            }

            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

/// 获取配置
unsafe fn get_config(hwnd: HWND) -> Option<&'static ServiceConfig> {
    let ptr = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) } as *const ServiceConfig;
    if ptr.is_null() {
        None
    } else {
        unsafe { Some(&*ptr) }
    }
}

/// UI线程更新状态
fn update_status_ui(hwnd: HWND, cfg: &ServiceConfig) {
    let new_status = query_service_status(&cfg.service_id).unwrap_or_default();
    set_status_text(hwnd, &new_status);
    update_button_states(hwnd, &cfg.service_id);
}

/// 运行 GUI
pub fn run_gui() -> Result<()> {
    write_wrapper_log("===== GUI 模式启动 =====");
    let config_result = read_config_from_exe();
    let config = match config_result {
        Ok(cfg) => cfg,
        Err(e) => {
            show_error_box("启动失败", &format!("读取配置失败: {}", e));
            return Err(e);
        }
    };
    let cfg_ptr = Box::into_raw(Box::new(config)) as *mut _;

    unsafe {
        let hinstance = GetModuleHandleW(std::ptr::null());
        let class_name = to_wstring("ServiceWrapperUI");
        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wnd_proc),
            hInstance: hinstance,
            hCursor: LoadCursorW(std::ptr::null_mut(), IDC_ARROW),
            hbrBackground: GetSysColorBrush(COLOR_BTNFACE),
            lpszClassName: class_name.as_ptr(),
            hIcon: LoadIconW(hinstance, MAKEINTRESOURCEW(1)),
            hIconSm: LoadIconW(hinstance, MAKEINTRESOURCEW(1)),
            ..std::mem::zeroed()
        };
        RegisterClassExW(&wc);

        let hwnd = CreateWindowExW(
            0,
            class_name.as_ptr(),
            to_wstring("服务管理").as_ptr(),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            420,
            260,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            hinstance,
            cfg_ptr,
        );

        // 启动定时器（最安全、永不报错、永不卡顿）
        SetTimer(hwnd, 999, 1000, None);

        let mut msg = std::mem::zeroed();
        while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
    Ok(())
}

/// 更新按钮
fn update_button_states(hwnd: HWND, id: &str) {
    let installed = query_service_status(id)
        .map(|s| s != "未安装")
        .unwrap_or(false);
    let running = installed
        && query_service_status(id)
            .map(|s| s.contains("运行中"))
            .unwrap_or(false);

    unsafe {
        EnableWindow(
            GetDlgItem(hwnd, ID_INSTALL),
            if installed { FALSE } else { TRUE },
        );
        EnableWindow(
            GetDlgItem(hwnd, ID_UNINSTALL),
            if installed { TRUE } else { FALSE },
        );
        EnableWindow(
            GetDlgItem(hwnd, ID_START),
            if installed && !running { TRUE } else { FALSE },
        );
        EnableWindow(
            GetDlgItem(hwnd, ID_STOP),
            if installed && running { TRUE } else { FALSE },
        );
    }
}

/// 设置文本
fn set_status_text(hwnd: HWND, text: &str) {
    unsafe {
        let txt = to_wstring(&format!("状态: {}", text));
        SetWindowTextW(GetDlgItem(hwnd, ID_STATUS), txt.as_ptr());
    }
}