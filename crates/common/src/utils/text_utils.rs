use std::process::Command;
use std::ptr::null_mut;

// ------------------------------
// Win32 常量 & 安全导入
// ------------------------------
const CP_OEMCP: u32 = 1;

unsafe extern "system" {
    fn MultiByteToWideChar(
        CodePage: u32,
        dwFlags: u32,
        lpMultiByteStr: *const u8,
        cbMultiByte: i32,
        lpWideCharStr: *mut u16,
        cchWideChar: i32,
    ) -> i32;
}

// ------------------------------
// 核心：Windows 控制台输出解码（永不失败）
// ------------------------------
pub fn decode_system_output(bytes: &[u8]) -> String {
    if bytes.is_empty() {
        return String::new();
    }

    unsafe {
        // 第一步：计算所需缓冲区大小
        let len = MultiByteToWideChar(
            CP_OEMCP,
            0,      // 控制台输出必须用 0，不能用 WC_NO_BEST_FIT_CHARS
            bytes.as_ptr(),
            bytes.len() as i32,
            null_mut(),
            0,
        );

        if len <= 0 {
            return String::from_utf8_lossy(bytes).into_owned();
        }

        // 第二步：解码为宽字符
        let mut wide = vec![0u16; len as usize];
        let result_len = MultiByteToWideChar(
            CP_OEMCP,
            0,
            bytes.as_ptr(),
            bytes.len() as i32,
            wide.as_mut_ptr(),
            len,
        );

        if result_len <= 0 {
            return String::from_utf8_lossy(bytes).into_owned();
        }

        // 第三步：安全转为 UTF-8
        String::from_utf16_lossy(&wide[..result_len as usize])
    }
}

// ------------------------------
// 执行系统命令（自动正确解码）
// ------------------------------
pub fn run_system_cmd(program: &str, args: &[&str]) -> Result<String, std::io::Error> {
    let output = Command::new(program).args(args).output()?;
    Ok(decode_system_output(&output.stdout))
}