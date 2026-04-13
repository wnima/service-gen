# Service Wrapper Generator (service-gen)

<div align="center">

![Rust](https://img.shields.io/badge/Rust-Edition%202024-orange?logo=rust)
![Platform](https://img.shields.io/badge/Platform-Windows-blue)
![License](https://img.shields.io/badge/License-MIT-green)

**基于 Rust 开发的 Windows Service 包装执行文件生成工具**

[功能特性](#-功能特性) • [快速开始](#-快速开始) • [使用指南](#-使用指南) • [项目结构](#-项目结构) • [开发指南](#-开发指南)

</div>

---

## 📖 项目简介

Service Wrapper Generator (service-gen) 是一个专为 Windows 平台设计的 Service 包装执行文件生成工具，采用 Rust 语言开发。它能够将普通可执行程序包装成 Windows 系统服务，并提供配置管理、打包导出等功能，让开发者能够轻松地将应用程序部署为系统服务。

### 💡 核心价值

- **服务包装**: 将任意可执行程序包装为标准 Windows 系统服务
- **配置简化**: 可视化编辑服务配置，自动生成标准化的 JSON 配置文件
- **一键打包**: 将服务包装器、配置和相关资源打包成 ZIP 格式，便于分发和部署
- **高性能可靠**: 基于 Rust 语言，保证内存安全和运行效率

---

## ✨ 功能特性

### 🔧 服务控制模块 (control)
- ✅ 作为 Windows Service 运行的包装器
- ✅ 支持服务模式后台运行
- ✅ 管理和监控被包装的子进程
- ✅ 服务注册与卸载
- ✅ 管理员权限自动检测
- ✅ 子进程生命周期管理（启动、停止、重启）

### ⚙️ 配置生成模块 (gen)
- ✅ 可视化编辑服务包装配置参数
- ✅ 支持 JSON 格式配置文件生成
- ✅ XML 配置文件解析与处理
- ✅ 配置验证与错误提示
- ✅ 一键打包导出为 ZIP 格式
- ✅ 生成完整的服务部署包

### 🛠️ 通用工具库 (common)
- ✅ 统一的数据模型定义
- ✅ 文件操作工具函数
- ✅ 日志记录与管理
- ✅ 文本处理工具
- ✅ GUI 辅助工具

---

## 🚀 快速开始

### 系统要求

- **操作系统**: Windows 10/11
- **Rust 工具链**: 最新稳定版（支持 Edition 2024）
- **Cargo**: 支持 resolver = "3" 的版本
- **权限**: 部分功能需要管理员权限

### 安装 Rust

如果尚未安装 Rust，请访问 [rustup.rs](https://rustup.rs/) 下载安装：

```bash
# Windows PowerShell 或 CMD
winget install Rustlang.Rustup
```

### 克隆项目

```bash
git clone <repository-url>
cd service-gen
```

### 构建项目

```bash
# 构建所有模块（Debug 模式）
cargo build

# 构建发布版本（Release/Optimized）
cargo build --release
```

---

## 📖 使用指南

### 运行服务控制模块（Service 包装器）

#### GUI 配置模式
```bash
cargo run --package control
```

#### 服务模式（作为 Windows Service 运行）
```bash
cargo run --package control -- -service
```

### 运行配置生成模块

```bash
cargo run --package gen
```

### 单独构建模块

```bash
# 构建通用工具库
cargo build --package common

# 构建服务控制模块（Service 包装器）
cargo build --package control

# 构建配置生成模块
cargo build --package gen
```

---

## 🏗️ 项目结构

```
service-gen/
├── crates/
│   ├── common/              # 通用工具库
│   │   ├── src/
│   │   │   ├── models/      # 数据模型定义
│   │   │   │   ├── mod.rs
│   │   │   │   └── service_config.rs    # 服务配置模型
│   │   │   └── utils/       # 工具函数
│   │   │       ├── exec_config_utils.rs # 执行配置工具
│   │   │       ├── file_utils.rs        # 文件操作工具
│   │   │       ├── gui_utils.rs         # GUI 辅助工具
│   │   │       ├── log_utils.rs         # 日志工具
│   │   │       ├── text_utils.rs        # 文本处理工具
│   │   │       └── mod.rs
│   │   └── Cargo.toml
│   │
│   ├── control/             # Service 包装器主应用
│   │   ├── src/
│   │   │   ├── main.rs      # 入口点（GUI/服务模式切换）
│   │   │   ├── gui.rs       # Iced GUI 实现
│   │   │   ├── service.rs   # 核心服务操作逻辑
│   │   │   ├── service_utils.rs    # 服务辅助工具
│   │   │   └── sc_utils.rs         # Windows SC 接口封装
│   │   ├── Cargo.toml
│   │   └── build.rs         # 构建脚本
│   │
│   └── gen/                 # 配置生成模块
│       ├── src/
│       │   ├── main.rs      # 入口点
│       │   ├── app_view.rs  # 应用程序主视图
│       │   ├── config_view.rs      # 配置编辑视图
│       │   └── package_export.rs   # 配置打包导出
│       ├── Cargo.toml
│       └── build.rs         # 构建脚本
│
├── Cargo.toml               # Workspace 配置
└── README.md                # 项目文档
```

---

## 🔍 技术栈

### 核心依赖

| 类别 | 技术 | 用途 |
|------|------|------|
| **语言** | Rust Edition 2024 | 系统级编程语言，保证内存安全 |
| **GUI 框架** | Iced v0.14 | 跨平台 GUI 框架，Elm 架构模式 |
| **异步运行时** | Tokio | 异步任务处理 |
| **序列化** | Serde, serde_json | JSON 数据序列化/反序列化 |
| **Windows API** | windows-service, winreg, winapi, windows | Windows 系统集成 |
| **日志** | tracing, tracing-subscriber | 结构化日志记录 |
| **文件处理** | zip, walkdir, tempfile | 文件压缩与目录遍历 |
| **XML 处理** | quick-xml | XML 配置文件解析 |
| **图像处理** | image | 图像格式转换与处理 |
| **错误处理** | anyhow, thiserror | 统一的错误处理机制 |
| **时间处理** | chrono | 日期时间操作 |

### 架构设计

- **Cargo Workspace**: 单体仓库管理多个 Crate
- **模块化设计**: `common` 提供共享库，`control` 和 `gen` 独立应用
- **Elm 架构**: GUI 组件遵循 Model-Update-View 模式
- **命令模式**: 分离界面逻辑与业务逻辑

---

## 🛠️ 开发指南

### 清理构建缓存

```bash
cargo clean
```

### 更新依赖

```bash
cargo update
```

### 调试模式

#### Windows CMD
```cmd
set RUST_LOG=debug
cargo run --package gen
```

#### 指定模块日志级别
```cmd
set RUST_LOG=gen=debug,common=info
cargo run --package gen
```

#### PowerShell
```powershell
$env:RUST_LOG="debug"
cargo run --package gen
```

### 代码规范

- 遵循 Rust 官方代码风格指南
- 使用 `cargo fmt` 格式化代码
- 使用 `cargo clippy` 检查代码质量
- 所有公共 API 必须添加文档注释

```bash
# 格式化代码
cargo fmt

# 代码检查
cargo clippy

# 运行测试
cargo test
```

---

## ⚠️ 注意事项

### 权限要求
- **服务注册/启动**: 需要管理员权限
- **建议**: 以管理员身份运行终端或应用程序

### 常见问题

#### 1. GUI 无响应
**原因**: Windows API 权限不足或环境问题  
**解决**: 
- 确保以管理员身份运行
- 检查 Windows 服务管理器是否正常运行

#### 2. 服务注册失败
**原因**: 服务名冲突、路径错误或权限不足  
**解决**:
- 检查服务名称是否已存在
- 验证可执行文件路径是否正确
- 确认具有管理员权限

#### 3. 构建错误
**原因**: 依赖冲突或缓存问题  
**解决**:
```bash
cargo clean
cargo update
cargo build
```

### 平台限制
- 主要针对 **Windows 10/11** 开发
- 强依赖 Windows API，跨平台兼容性有限
- 部分功能可能不支持其他操作系统

---

## 📝 许可证

本项目采用 MIT 许可证。详见 [LICENSE](LICENSE) 文件。

---

## 🤝 贡献指南

欢迎提交 Issue 和 Pull Request！

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 开启 Pull Request

---

## 📮 联系方式

- **项目主页**: [GitHub Repository](<repository-url>)
- **问题反馈**: [Issues](<repository-url>/issues)

---

<div align="center">

**Made with ❤️ using Rust**

⭐ 如果这个项目对你有帮助，请给我们一个 Star！

</div>
