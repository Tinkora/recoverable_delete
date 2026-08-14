# Recoverable Delete

<div align="center">
  <a href="https://ko-fi.com/tinkora" target="_blank" rel="noopener noreferrer">
    <img src="https://ko-fi.com/img/githubbutton_sm.svg" alt="在 Ko-fi 上支持 Tinkora" width="520">
  </a>
</div>

Recoverable Delete 是一个 Codex Plugin：它会阻止常见的永久删除路径，并引导 Agent 把明确的文件或目录移入操作系统回收站。

[English](README.md)

## 为什么还需要这个项目？

已有项目解决了相邻问题：

- [`skill-use-trash`](https://github.com/ar1g/skill-use-trash) 是面向 macOS、Linux 的轻量提示型 Skill。
- [`ai-trash`](https://github.com/forethought-studio/ai-trash) 是成熟的 MIT 跨平台回收站 CLI 和 PATH 包装器。

它们都不是同时覆盖 shell 和 `apply_patch` 的 Codex 原生 `PreToolUse` Plugin。本项目只补充 Codex 集成和策略检测层，可以与 `ai-trash` 配合使用，不重新实现一套回收站存储。

## 当前能力

- 阻止 `rm`、`rmdir`、`unlink`、`find -delete`、`git clean`、`rsync --delete`、PowerShell `Remove-Item`、Windows `del` 等常见物理删除命令。
- 识别嵌套 shell、`xargs` 命令、常见命令包装器、绝对可执行文件路径、Python/Node/Ruby 内联删除 API，以及 `apply_patch` 的 `Delete File`。
- 对无法安全分析的 `$()` 和反引号命令替换直接拒绝，采用 fail-closed 策略，不让不透明的嵌套命令通过。
- 放行普通命令和已知的回收站命令。
- Hook 输入损坏或发布包缺少策略二进制文件时 fail closed。

它是一层安全护栏，不是操作系统级安全边界。任意原生程序、特殊工具路径或被主动绕过的 Hook 仍可能删除数据。

## 开发验证

项目通过 `rust-toolchain.toml` 选择共享 Rust 工具链。

```sh
cargo fmt --all -- --check
cargo test --locked
cargo clippy --all-targets --locked -- -D warnings
```

生成当前平台的 Plugin 目录：

```sh
sh scripts/package_plugin.sh
```

Windows 使用：

```powershell
./scripts/package_plugin.ps1
```

端到端运行打包后的 Windows 策略二进制文件和 PowerShell Hook dispatcher：

```powershell
cargo build --release --locked
$outputRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("recoverable_delete_verify_" + [guid]::NewGuid())
./scripts/verify_windows_plugin.ps1 `
  -PolicyBinary .\target\release\recoverable-delete.exe `
  -OutputRoot $outputRoot
```

两个脚本都拒绝覆盖已有发布目录；重新打包前，需要先把旧目录移入废纸篓或回收站。缺少已打包策略二进制文件时，Hook 会有意阻止所有已匹配工具调用。

## 安装发布版本

每个平台归档解压后都是一个名为 `recoverable_delete` 的完整本地 Marketplace 根目录。先验证 SHA-256，再解压到稳定目录，通过 `codex plugin marketplace add` 添加该目录，然后安装 `recoverable-delete@tinkora`。

平台命令、Hook 信任、恢复、升级、卸载和架构限制见 [v0.1.0 发布与安装指南](docs/release_v0_1_0.zh_CN.md)。

将仓库 Marketplace 加入 Codex：

```sh
codex plugin marketplace add /absolute/path/to/recoverable_delete
codex plugin add recoverable-delete@tinkora
```

然后新建 Codex 任务，通过 `/hooks` 审查并信任 Plugin Hook。安装 Plugin 不会自动授予 Hook 信任，这是 Codex 的安全设计。

推送 `v*` tag 后，Release workflow 会分别构建 Linux ARM64、macOS ARM64、Windows x64 原生 Marketplace 压缩包，并把归档、校验和及内容清单附加到 GitHub Release。

## 文档

- [同类方案调研](docs/existing_solutions.md)
- [架构与安全边界](docs/architecture.md)
- [v0.1.0 发布与安装](docs/release_v0_1_0.zh_CN.md)
- [Codex Hooks 官方文档](https://learn.chatgpt.com/docs/hooks)
- [Plugin 打包官方文档](https://developers.openai.com/plugins/build/plugins)

## 许可证

MIT
