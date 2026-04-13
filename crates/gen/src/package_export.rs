use anyhow::Result;
use common::models::service_config::ServiceConfig;
use common::utils::exec_config_utils::write_config_to_exe;
use std::path::{Path, PathBuf};
use tokio::{fs, io::AsyncWriteExt};
use tracing::info;

pub struct PackageExportService {}

impl PackageExportService {
    pub fn new() -> Self {
        Self {}
    }

    /// 导出服务配置包
    pub async fn export_package(
        &self,
        config: &mut ServiceConfig,
        output_dir: &Path,
    ) -> Result<PathBuf> {
        // 创建包目录
        let package_dir = output_dir.join(format!("{}-package", config.service_id));
        fs::create_dir_all(&package_dir).await?;

        info!("开始生成包装器：{}", config.service_id);
        info!("输出目录：{}", package_dir.display());

        // 3. 复制 JAR 包（如果选择包含）
        if config.include_jar {
            self.copy_jar_file(&package_dir, config).await?;
        }

        // 4. 复制 JRE（如果选择打包）
        if config.bundle_jre {
            if let Some(jre_path) = &config.jre_path {
                self.copy_jre_directory(&package_dir, jre_path).await?;
            }
        }

        // 5. 生成可执行命令
        self.create_executable(config).await;

        // 8. 复制 GUI 控制器
        self.copy_gui_controller(&package_dir, config).await?;

        // 9. 打包成 ZIP
        let zip_path = self
            .create_zip_file(&package_dir, config, output_dir)
            .await?;

        Ok(zip_path)
    }

    /// 复制 JAR 文件
    async fn copy_jar_file(&self, package_dir: &Path, config: &ServiceConfig) -> Result<()> {
        let jar_source = Path::new(&config.jar_path);
        if !jar_source.exists() {
            anyhow::bail!("JAR 文件不存在：{}", config.jar_path);
        }

        let jar_file_name = jar_source
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("app.jar");
        let jar_dest = package_dir.join(jar_file_name);

        fs::copy(jar_source, &jar_dest).await?;
        info!("已复制 JAR 文件：{}", jar_file_name);

        Ok(())
    }

    /// 复制 JRE 目录
    async fn copy_jre_directory(&self, package_dir: &Path, jre_path: &str) -> Result<()> {
        let jre_source = Path::new(jre_path);
        if !jre_source.exists() {
            anyhow::bail!("JRE 目录不存在：{}", jre_path);
        }

        // 无论源目录叫什么，都重命名为 jre
        let jre_dest = package_dir.join("jre");
        self.copy_directory(jre_source, &jre_dest)?;
        info!("已复制 JRE 目录：{} -> jre", jre_path);

        Ok(())
    }

    async fn create_executable(&self, config: &mut ServiceConfig) {
        // 组装可执行命令
        // 1. 是否指定 Java 可执行文件
        let exec_path = if let Some(java_exec) = &config.java_executable {
            java_exec.clone()
        }
        // 2. 如果打包了 JRE，优先使用包内的 java
        else if config.bundle_jre {
            "jre/bin/java".to_string()
        } else {
            "java".to_string() // 默认使用系统 PATH 中的 java
        };
        // 添加JVM参数
        let exec_with_args = if let Some(jvm_opts) = &config.jvm_options {
            format!("{} {}", exec_path, jvm_opts.join(" "))
        } else {
            exec_path.clone()
        };
        // 如果启用调试，添加调试参数
        let exec_with_args = if config.enable_debug {
            let debug_port = config.debug_port.unwrap_or(5005);
            let debug_host = config
                .debug_host
                .clone()
                .unwrap_or_else(|| "localhost".to_string());
            format!(
                "{} -agentlib:jdwp=transport=dt_socket,server=y,suspend=n,address={}:{}",
                exec_with_args, debug_host, debug_port
            )
        } else {
            exec_with_args
        };
        // 添加jar包路径和应用参数，如果开启了包含JAR包，则jar包就在当前目录，否则假设在外部路径
        let jar_path = if config.include_jar {
            let jar_file_name = Path::new(&config.jar_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("app.jar");
            jar_file_name.to_string()
        } else {
            config.jar_path.clone()
        };
        let exec_with_args = format!("{} -jar {}", exec_with_args, jar_path);
        let exec_with_args = if let Some(app_args) = &config.app_args {
            format!("{} {}", exec_with_args, app_args.join(" "))
        } else {
            exec_with_args
        };
        info!("生成的可执行命令：{}", exec_with_args);
        config.executable = Some(exec_with_args);
    }

    async fn copy_gui_controller(&self, package_dir: &Path, config: &ServiceConfig) -> Result<()> {
        let text = serde_json::to_string_pretty(config)?;

        let dest = package_dir.join(format!("{}-control.exe", config.service_id));

        // 将 assets/service_control.exe 编译到可执行文件中，运行时直接写出
        #[cfg(all(debug_assertions))]
        const GUI_CONTROLLER_EXE: &[u8] = include_bytes!("../../../target/debug/control.exe");

        #[cfg(all(not(debug_assertions)))]
        const GUI_CONTROLLER_EXE: &[u8] = include_bytes!("../../../target/release/control.exe");

        let mut exe_file = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .write(true)
            .open(&dest)
            .await?;

        exe_file.write_all(GUI_CONTROLLER_EXE).await?;

        write_config_to_exe(&dest, text).await?;

        info!("已写出嵌入的 GUI 控制器可执行文件：{}", dest.display());
        Ok(())
    }

    /// 创建 ZIP 文件
    async fn create_zip_file(
        &self,
        source_dir: &Path,
        config: &ServiceConfig,
        output_dir: &Path,
    ) -> Result<PathBuf> {
        use zip::write::{FileOptions, ZipWriter};

        let zip_file_name = format!("{}-package.zip", config.service_id);
        let zip_path = output_dir.join(zip_file_name);

        let file = std::fs::File::create(&zip_path)?;
        let mut zip = ZipWriter::new(file);
        let options = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        self.add_directory_to_zip(&mut zip, source_dir, source_dir, &options)?;

        info!("已创建 ZIP 文件：{}", zip_path.display());

        Ok(zip_path)
    }

    /// 递归添加目录到 ZIP
    fn add_directory_to_zip(
        &self,
        zip: &mut zip::write::ZipWriter<std::fs::File>,
        dir: &Path,
        base_dir: &Path,
        options: &zip::write::FileOptions,
    ) -> Result<()> {
        for entry in walkdir::WalkDir::new(dir) {
            let entry = entry?;
            let path = entry.path();

            // 跳过基目录本身
            if path == dir {
                continue;
            }

            let entry_name = path
                .strip_prefix(base_dir)?
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("无效路径"))?
                .replace("\\", "/");

            if path.is_dir() {
                zip.add_directory(entry_name, *options)?;
            } else {
                zip.start_file(entry_name, *options)?;
                let mut f = std::fs::File::open(path)?;
                std::io::copy(&mut f, zip)?;
            }
        }

        zip.finish()?;
        Ok(())
    }

    /// 递归复制目录
    fn copy_directory(&self, source: &Path, target: &Path) -> Result<()> {
        std::fs::create_dir_all(target)?;

        for entry in walkdir::WalkDir::new(source).into_iter().flatten() {
            let entry_path = entry.path();
            let relative_path = entry_path.strip_prefix(source)?;
            let target_path = target.join(relative_path);

            if entry.file_type().is_dir() {
                std::fs::create_dir_all(&target_path)?;
            } else {
                std::fs::copy(entry_path, &target_path)?;
            }
        }

        Ok(())
    }
}
