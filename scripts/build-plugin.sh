#!/bin/bash
# 插件包构建脚本
# 用法: ./scripts/build-plugin.sh <plugin_name> [version]
# 示例: ./scripts/build-plugin.sh machine-id-tool 0.1.0
#
# 此脚本将插件目录打包为 zip 格式，用于分发和安装
# _需求: 6.1_

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 打印带颜色的消息
info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
    exit 1
}

# 检查参数
if [ -z "$1" ]; then
    echo "用法: $0 <plugin_name> [version]"
    echo "示例: $0 machine-id-tool 0.1.0"
    echo ""
    echo "参数:"
    echo "  plugin_name  插件名称（对应 plugins/ 目录下的文件夹名）"
    echo "  version      可选，覆盖 plugin.json 中的版本号"
    exit 1
fi

PLUGIN_NAME="$1"
VERSION_OVERRIDE="$2"

# 获取脚本和项目目录
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
PLUGINS_DIR="$PROJECT_ROOT/plugins"
PLUGIN_DIR="$PLUGINS_DIR/$PLUGIN_NAME"
OUTPUT_DIR="$PROJECT_ROOT/dist/plugins"

# 检查插件目录是否存在
if [ ! -d "$PLUGIN_DIR" ]; then
    error "插件目录不存在: $PLUGIN_DIR"
fi

# 检查 plugin.json 是否存在
MANIFEST_FILE="$PLUGIN_DIR/plugin.json"
if [ ! -f "$MANIFEST_FILE" ]; then
    error "插件清单文件不存在: $MANIFEST_FILE"
fi

# 读取版本号
if [ -n "$VERSION_OVERRIDE" ]; then
    VERSION="$VERSION_OVERRIDE"
    info "使用覆盖版本: $VERSION"
else
    # 从 plugin.json 读取版本
    VERSION=$(grep -o '"version"[[:space:]]*:[[:space:]]*"[^"]*"' "$MANIFEST_FILE" | head -1 | sed 's/.*"\([^"]*\)"$/\1/')
    if [ -z "$VERSION" ]; then
        error "无法从 plugin.json 读取版本号"
    fi
    info "从 plugin.json 读取版本: $VERSION"
fi

# 创建输出目录
mkdir -p "$OUTPUT_DIR"

# 定义输出文件名
OUTPUT_FILE="$OUTPUT_DIR/${PLUGIN_NAME}-${VERSION}.zip"

info "开始打包插件: $PLUGIN_NAME v$VERSION"
info "源目录: $PLUGIN_DIR"
info "输出文件: $OUTPUT_FILE"

# 如果输出文件已存在，先删除
if [ -f "$OUTPUT_FILE" ]; then
    warn "输出文件已存在，将被覆盖"
    rm -f "$OUTPUT_FILE"
fi

# 创建临时目录用于打包
TEMP_DIR=$(mktemp -d)
TEMP_PLUGIN_DIR="$TEMP_DIR/$PLUGIN_NAME"

# 复制插件文件到临时目录
info "复制插件文件..."
mkdir -p "$TEMP_PLUGIN_DIR"
cp -r "$PLUGIN_DIR"/* "$TEMP_PLUGIN_DIR/"

# 如果指定了版本覆盖，更新 plugin.json
if [ -n "$VERSION_OVERRIDE" ]; then
    info "更新 plugin.json 版本号..."
    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS
        sed -i '' "s/\"version\"[[:space:]]*:[[:space:]]*\"[^\"]*\"/\"version\": \"$VERSION_OVERRIDE\"/" "$TEMP_PLUGIN_DIR/plugin.json"
    else
        # Linux
        sed -i "s/\"version\"[[:space:]]*:[[:space:]]*\"[^\"]*\"/\"version\": \"$VERSION_OVERRIDE\"/" "$TEMP_PLUGIN_DIR/plugin.json"
    fi
fi

# 创建 zip 包
info "创建 zip 包..."
cd "$TEMP_DIR"
zip -r "$OUTPUT_FILE" "$PLUGIN_NAME" -x "*.DS_Store" -x "*__MACOSX*"

# 清理临时目录
rm -rf "$TEMP_DIR"

# 计算校验和
info "计算校验和..."
if command -v sha256sum &> /dev/null; then
    CHECKSUM=$(sha256sum "$OUTPUT_FILE" | awk '{print $1}')
elif command -v shasum &> /dev/null; then
    CHECKSUM=$(shasum -a 256 "$OUTPUT_FILE" | awk '{print $1}')
else
    warn "无法计算校验和：未找到 sha256sum 或 shasum 命令"
    CHECKSUM="N/A"
fi

# 输出结果
echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}插件打包完成！${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo "插件名称: $PLUGIN_NAME"
echo "版本: $VERSION"
echo "输出文件: $OUTPUT_FILE"
echo "文件大小: $(du -h "$OUTPUT_FILE" | cut -f1)"
echo "SHA256: $CHECKSUM"
echo ""
echo "安装方式:"
echo "  1. 在 ProxyCast 中打开插件管理器"
echo "  2. 点击「从文件安装」"
echo "  3. 选择 $OUTPUT_FILE"
