# Todo Studio

一个基于 `Vue 3 + Vite + Tauri 2 + Rust` 的桌面待办应用。

## 技术栈

- 前端：`Vue 3`、`Vite`、`TypeScript`
- 桌面端：`Tauri 2`
- 后端能力：`Rust`
- 数据存储：`rusqlite`

## 本地开发

### 1. 安装依赖

```bash
npm install
```

### 2. 启动开发环境

```bash
npm run tauri dev
```

该命令会同时启动：

- Vite 开发服务
- Tauri 桌面应用

## 生产构建

### 仅构建前端静态资源

```bash
npm run build
```

### 构建当前机器对应平台的桌面安装包

```bash
npm run build:desktop
```

或：

```bash
npm run tauri build
```

构建完成后，产物通常位于：

```bash
src-tauri/target/release/bundle/
```

## 编译安装包所需基础环境

所有平台通用的基础环境：

- `Node.js`（建议使用 LTS 版本）
- `npm`
- `Rust`
- `Cargo`
- `Tauri CLI`（当前项目已通过 npm 依赖提供）

建议先检查以下命令是否可用：

```bash
node -v
npm -v
rustc -V
cargo -V
```

如需安装 Rust：

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

安装项目依赖：

```bash
npm install
```

### macOS 构建环境

适用于：

- `aarch64-apple-darwin`
- `x86_64-apple-darwin`

需要具备：

- macOS 系统
- Xcode Command Line Tools
- Rust macOS 对应 target

检查 Xcode Command Line Tools：

```bash
xcode-select -p
```

如未安装：

```bash
xcode-select --install
```

安装 Rust target：

```bash
rustup target add aarch64-apple-darwin
rustup target add x86_64-apple-darwin
```

如果要正式分发给其他用户，额外建议准备：

- Apple Developer 账号
- Developer ID 签名证书
- notarization（公证）能力

### macOS 图标文件生成说明

macOS 安装包和应用本体通常会使用 `icon.icns` 作为应用图标文件。

`icon.icns` 不是普通单张图片，而是一个包含多种尺寸图标的 macOS 图标容器文件。通常做法是：

1. 先准备一个 `icon.iconset` 目录
2. 将不同尺寸的 PNG 图标放进去
3. 再使用下面命令打包生成 `icon.icns`

```bash
iconutil -c icns icon.iconset -o icon.icns
```

这个命令的作用是：

- 读取 `icon.iconset/` 目录中的多尺寸 PNG 图标
- 将它们打包为 macOS 可识别的 `icon.icns`
- 供 Tauri 在构建 `.app` 和 `.dmg` 时使用

如果你修改了主图标但 macOS 打包后仍显示旧图标，请确认已经重新生成过 `icon.icns`，再重新执行打包。

### Windows 构建环境

适用于：

- `x86_64-pc-windows-msvc`
- `aarch64-pc-windows-msvc`

推荐在 Windows 原生环境下构建，并准备：

- Windows 10 / 11
- Microsoft Visual Studio Build Tools 或 Visual Studio
- MSVC C++ 编译工具链
- Windows SDK
- Rust 对应的 MSVC target
- WebView2 运行时（大多数 Windows 11 已内置，部分 Windows 10 需要单独安装）

安装 Rust target：

```bash
rustup target add x86_64-pc-windows-msvc
rustup target add aarch64-pc-windows-msvc
```

如果需要生成 `.msi`，通常还需要 Windows 打包相关组件；在 Windows 原生环境下最稳妥。

如果要正式分发给其他用户，建议准备：

- 代码签名证书

### Linux 构建环境

适用于：

- `x86_64-unknown-linux-gnu`
- `aarch64-unknown-linux-gnu`

推荐在 Linux 原生环境下构建，并准备：

- GCC / Clang
- `pkg-config`
- `libwebkit2gtk`
- `libgtk-3`
- `libsoup`
- `librsvg2`
- `patchelf`（某些包格式需要）
- Rust 对应 Linux target

Debian / Ubuntu 常见安装方式示例：

```bash
sudo apt update
sudo apt install -y \
  build-essential \
  pkg-config \
  libgtk-3-dev \
  libwebkit2gtk-4.1-dev \
  libsoup-3.0-dev \
  librsvg2-dev \
  patchelf
```

安装 Rust target：

```bash
rustup target add x86_64-unknown-linux-gnu
rustup target add aarch64-unknown-linux-gnu
```

不同发行版的包名可能略有差异，请根据实际系统调整。

## 各平台安装包打包命令

当前项目已启用 Tauri bundling，配置文件为 `src-tauri/tauri.conf.json`，支持按目标平台打包。

> 注意：跨平台打包通常需要在对应操作系统上进行，或提前配置对应的交叉编译环境。

---

## macOS

### macOS Apple Silicon（M1 / M2 / M3 / M4）

基础环境：

- macOS
- Xcode Command Line Tools
- `rustup target add aarch64-apple-darwin`

打包命令：

```bash
npm run build:mac:arm
```

等价命令：

```bash
npm run tauri build -- --target aarch64-apple-darwin
```

产物目录：

```bash
src-tauri/target/aarch64-apple-darwin/release/bundle/
```

### macOS Intel

基础环境：

- macOS
- Xcode Command Line Tools
- `rustup target add x86_64-apple-darwin`

打包命令：

```bash
npm run build:mac:intel
```

等价命令：

```bash
npm run tauri build -- --target x86_64-apple-darwin
```

产物目录：

```bash
src-tauri/target/x86_64-apple-darwin/release/bundle/
```

---

## Windows

### Windows x64

基础环境：

- Windows
- Visual Studio Build Tools / MSVC
- Windows SDK
- `rustup target add x86_64-pc-windows-msvc`

打包命令：

```bash
npm run build:win:x64
```

等价命令：

```bash
npm run tauri build -- --target x86_64-pc-windows-msvc
```

产物目录：

```bash
src-tauri/target/x86_64-pc-windows-msvc/release/bundle/
```

### Windows ARM64

基础环境：

- Windows
- Visual Studio Build Tools / MSVC
- Windows SDK
- `rustup target add aarch64-pc-windows-msvc`

打包命令：

```bash
npm run build:win:arm
```

等价命令：

```bash
npm run tauri build -- --target aarch64-pc-windows-msvc
```

产物目录：

```bash
src-tauri/target/aarch64-pc-windows-msvc/release/bundle/
```

---

## Linux

### Linux x64

基础环境：

- Linux
- `build-essential`
- `pkg-config`
- `libgtk-3-dev`
- `libwebkit2gtk`
- `libsoup`
- `librsvg2-dev`
- `patchelf`
- `rustup target add x86_64-unknown-linux-gnu`

打包命令：

```bash
npm run build:linux:x64
```

等价命令：

```bash
npm run tauri build -- --target x86_64-unknown-linux-gnu
```

产物目录：

```bash
src-tauri/target/x86_64-unknown-linux-gnu/release/bundle/
```

### Linux ARM64

基础环境：

- Linux
- `build-essential`
- `pkg-config`
- `libgtk-3-dev`
- `libwebkit2gtk`
- `libsoup`
- `librsvg2-dev`
- `patchelf`
- `rustup target add aarch64-unknown-linux-gnu`

打包命令：

```bash
npm run build:linux:arm
```

等价命令：

```bash
npm run tauri build -- --target aarch64-unknown-linux-gnu
```

产物目录：

```bash
src-tauri/target/aarch64-unknown-linux-gnu/release/bundle/
```

---

## 发布流程建议

### macOS 发布

如果只是本机测试：

- 可直接执行 `npm run build:mac:arm`
- 双击 `.app` 或 `.dmg` 测试
- 如系统拦截，可右键后选择“打开”

如果要分发给其他用户，建议补全以下流程：

1. 使用 Apple Developer 账号进行签名
2. 对应用进行 notarization（公证）
3. 上传 `.dmg` 作为最终分发文件

推荐分发产物：

- `Apple Silicon` 用户：`.dmg`
- 如需兼容 Intel：额外再构建一个 `x86_64` 版本

### Windows 发布

建议在 Windows 原生环境下构建：

- `npm run build:win:x64`
- 优先分发 `.msi`
- 如需便携版，可同时保留 `.exe`

### Linux 发布

建议在 Linux 原生环境下构建：

- `npm run build:linux:x64`
- 通常优先分发：`.AppImage`
- 如果目标用户明确使用 Debian/Ubuntu，可提供 `.deb`
- 如果目标用户明确使用 Fedora/RHEL，可提供 `.rpm`

## 常见产物类型

由于当前配置中 `bundle.targets` 为 `all`，构建时会尽量生成当前平台支持的所有安装包格式。

不同平台常见产物包括：

- macOS：`.app`、`.dmg`
- Windows：`.msi`、`.exe`
- Linux：`.deb`、`.rpm`、`.AppImage`

最终实际生成哪些文件，取决于：

- 当前操作系统
- 已安装的系统打包依赖
- Tauri bundler 对该平台的支持情况

## 常见问题

### 1. macOS 提示应用“已损坏”或“无法验证开发者”

这是因为应用未签名或未公证。测试时可以右键应用后选择“打开”，正式分发建议配置：

- Apple Developer 签名
- Notarization 公证

### 2. Windows 或 Linux 能不能在 macOS 上直接打包？

理论上可以通过交叉编译尝试，但桌面安装包通常更推荐在目标平台原生构建，兼容性和依赖处理更稳妥。

### 3. 如何查看 Tauri 支持的 target？

可以执行：

```bash
rustup target list
```

如需安装某个 target，例如 macOS ARM64：

```bash
rustup target add aarch64-apple-darwin
```

## 项目脚本

`package.json` 中当前可用脚本：

```bash
npm run dev
npm run build
npm run preview
npm run tauri
npm run build:desktop
npm run build:mac:arm
npm run build:mac:intel
npm run build:win:x64
npm run build:win:arm
npm run build:linux:x64
npm run build:linux:arm
```

## 目录说明

```text
src/          前端 Vue 应用
src-tauri/    Tauri / Rust 桌面端代码与打包配置
dist/         前端构建产物
```
