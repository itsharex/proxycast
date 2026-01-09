#!/bin/bash
# 同步资源文件到 target/debug 目录

echo "正在同步资源文件..."

# 创建目标目录
mkdir -p src-tauri/target/debug/resources/models/providers
mkdir -p src-tauri/target/debug/resources/models/aliases

# 复制模型文件
cp -v src-tauri/resources/models/index.json src-tauri/target/debug/resources/models/
cp -v src-tauri/resources/models/providers/*.json src-tauri/target/debug/resources/models/providers/
cp -v src-tauri/resources/models/aliases/*.json src-tauri/target/debug/resources/models/aliases/ 2>/dev/null || true

echo "✅ 资源文件同步完成！"
