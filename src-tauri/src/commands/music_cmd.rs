use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;

/// MIDI 分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MidiAnalysisResult {
    /// 调式信息
    pub mode: String,
    /// BPM (每分钟节拍数)
    pub bpm: f64,
    /// 拍号
    pub time_signature: String,
    /// 音轨信息
    pub tracks: Vec<TrackInfo>,
    /// 旋律特征
    pub melody_features: MelodyFeatures,
}

/// 音轨信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackInfo {
    /// 音轨索引
    pub index: usize,
    /// 音轨名称
    pub name: String,
    /// 乐器名称
    pub instrument: String,
    /// 音符数量
    pub note_count: usize,
    /// 是否为人声音轨
    pub is_vocal: bool,
}

/// 旋律特征
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MelodyFeatures {
    /// 音域范围 (半音数)
    pub range: i32,
    /// 平均音高
    pub avg_pitch: f64,
    /// 音程跳跃频率
    pub interval_jumps: f64,
    /// 节奏复杂度
    pub rhythm_complexity: f64,
}

/// Python 环境检测结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythonEnvInfo {
    /// 是否已安装 Python
    pub python_installed: bool,
    /// Python 版本
    pub python_version: Option<String>,
    /// 缺失的依赖包
    pub missing_packages: Vec<String>,
}

/// 检查 Python 环境
#[tauri::command]
pub async fn check_python_env() -> Result<PythonEnvInfo, String> {
    // 检查 Python 是否安装
    let python_check = Command::new("python3").arg("--version").output();

    let (python_installed, python_version) = match python_check {
        Ok(output) => {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            (true, Some(version))
        }
        Err(_) => (false, None),
    };

    if !python_installed {
        return Ok(PythonEnvInfo {
            python_installed: false,
            python_version: None,
            missing_packages: vec![],
        });
    }

    // 检查必需的 Python 包
    let required_packages = vec!["mido", "music21", "numpy", "demucs", "basic-pitch"];
    let mut missing_packages = Vec::new();

    for package in required_packages {
        let check = Command::new("python3")
            .arg("-c")
            .arg(format!("import {}", package.replace("-", "_")))
            .output();

        if check.is_err() || !check.unwrap().status.success() {
            missing_packages.push(package.to_string());
        }
    }

    Ok(PythonEnvInfo {
        python_installed,
        python_version,
        missing_packages,
    })
}

/// 分析 MIDI 文件
#[tauri::command]
pub async fn analyze_midi(midi_path: String) -> Result<MidiAnalysisResult, String> {
    // 获取 Python 脚本路径
    let script_path = get_resource_path("scripts/midi_analyzer.py")?;

    // 调用 Python 脚本
    let output = Command::new("python3")
        .arg(&script_path)
        .arg(&midi_path)
        .output()
        .map_err(|e| format!("Failed to execute Python script: {e}"))?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(format!("MIDI analysis failed: {error}"));
    }

    // 解析 JSON 输出
    let result_json = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(&result_json).map_err(|e| format!("Failed to parse analysis result: {e}"))
}

/// 将 MP3 转换为 MIDI
#[tauri::command]
pub async fn convert_mp3_to_midi(mp3_path: String, output_path: String) -> Result<String, String> {
    // 获取 Python 脚本路径
    let script_path = get_resource_path("scripts/audio_to_midi.py")?;

    // 调用 Python 脚本
    let output = Command::new("python3")
        .arg(&script_path)
        .arg(&mp3_path)
        .arg(&output_path)
        .output()
        .map_err(|e| format!("Failed to execute Python script: {e}"))?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(format!("MP3 to MIDI conversion failed: {error}"));
    }

    Ok(output_path)
}

/// 加载资源文件
#[tauri::command]
pub async fn load_music_resource(resource_name: String) -> Result<String, String> {
    let resource_path = get_resource_path(&format!("music/{resource_name}"))?;

    std::fs::read_to_string(&resource_path)
        .map_err(|e| format!("Failed to read resource file: {e}"))
}

/// 获取资源文件路径
fn get_resource_path(relative_path: &str) -> Result<PathBuf, String> {
    // 在开发环境中，资源文件在 src-tauri/resources/
    // 在生产环境中，资源文件会被打包到应用程序包中
    let mut path =
        std::env::current_exe().map_err(|e| format!("Failed to get executable path: {e}"))?;

    path.pop(); // 移除可执行文件名

    #[cfg(target_os = "macos")]
    {
        // macOS: 资源在 .app/Contents/Resources/
        path.pop(); // 移除 MacOS
        path.push("Resources");
    }

    #[cfg(not(target_os = "macos"))]
    {
        // Windows/Linux: 资源在可执行文件同级目录
        path.push("resources");
    }

    path.push(relative_path);

    if !path.exists() {
        // 尝试开发环境路径
        let dev_path = PathBuf::from("src-tauri/resources").join(relative_path);
        if dev_path.exists() {
            return Ok(dev_path);
        }
        return Err(format!("Resource not found: {relative_path}"));
    }

    Ok(path)
}

/// 安装 Python 依赖
#[tauri::command]
pub async fn install_python_dependencies() -> Result<String, String> {
    let packages = vec!["mido", "music21", "numpy", "demucs", "basic-pitch"];

    let output = Command::new("pip3")
        .arg("install")
        .args(&packages)
        .output()
        .map_err(|e| format!("Failed to install packages: {e}"))?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Installation failed: {error}"));
    }

    Ok("Dependencies installed successfully".to_string())
}
