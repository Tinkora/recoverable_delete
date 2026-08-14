# Recoverable Delete

Recoverable Delete 是一个 Codex Plugin：它会阻止常见的永久删除路径，并引导 Agent 把明确的文件或目录移入操作系统回收站。

[English](README.md)

## 为什么还需要这个项目？

已有项目解决了相邻问题：

- [`skill-use-trash`](https://github.com/ar1g/skill-use-trash) 是面向 macOS、Linux 的轻量提示型 Skill。
- [`ai-trash`](https://github.com/forethought-studio/ai-trash) 是成熟的 MIT 跨平台回收站 CLI 和 PATH 包装器。

它们都不是同时覆盖 shell 和 `apply_patch` 的 Codex 原生 `PreToolUse` Plugin。本项目只补充 Codex 集成和策略检测层，可以与 `ai-trash` 配合使用，不重新实现一套回收站存储。

## 当前能力

- 阻止 `rm`、`rmdir`、`unlink`、`find -delete`、`git clean`、`xargs rm`、PowerShell `Remove-Item`、Windows `del` 等常见物理删除命令。
- 识别嵌套 shell、常见命令包装器、绝对可执行文件路径，以及 `apply_patch` 的 `Delete File`。
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

本地测试时，需要先编译当前平台策略二进制文件，并将其放入 Plugin 副本的 `bin/recoverable-delete`；Windows 文件名为 `bin/recoverable-delete.exe`。缺少二进制文件时，Hook 会有意阻止所有已匹配工具调用。

将仓库 Marketplace 加入 Codex：

```sh
codex plugin marketplace add /absolute/path/to/recoverable_delete
codex plugin add recoverable-delete@tinkora
```

然后新建 Codex 任务，通过 `/hooks` 审查并信任 Plugin Hook。安装 Plugin 不会自动授予 Hook 信任，这是 Codex 的安全设计。

## 文档

- [同类方案调研](docs/existing_solutions.md)
- [架构与安全边界](docs/architecture.md)
- [Codex Hooks 官方文档](https://learn.chatgpt.com/docs/hooks)
- [Plugin 打包官方文档](https://developers.openai.com/plugins/build/plugins)

## 许可证

MIT
