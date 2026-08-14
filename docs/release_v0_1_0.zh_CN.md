# v0.1.0 发布与安装

## 发布产物

`v0.1.0` 为每个已支持架构发布一个完整的本地 Marketplace 归档：

| 主机 | 归档 |
| --- | --- |
| macOS ARM64 | `recoverable-delete-macos-arm64.tar.gz` |
| Linux ARM64 | `recoverable-delete-linux-arm64.tar.gz` |
| Windows x64 | `recoverable-delete-windows-x86_64.zip` |

这个发布候选暂不包含 macOS x64、Linux x64 和 Windows ARM64。安装错误操作系统或架构的归档后，Hook 无法运行策略二进制文件，dispatcher 会 fail closed。

每个归档都有两个配套文件：

- `.sha256` 使用标准校验格式记录归档 SHA-256。
- `.contents.txt` 记录版本、Marketplace 名称、归档哈希和精确文件清单。

Release workflow 会在创建归档前拒绝额外文件、缺失文件、符号链接以及 Cargo/Plugin 版本不一致。

## 验证下载

macOS：

```sh
shasum -a 256 -c recoverable-delete-macos-arm64.tar.gz.sha256
```

Linux：

```sh
sha256sum -c recoverable-delete-linux-arm64.tar.gz.sha256
```

Windows PowerShell：

```powershell
$archive = "recoverable-delete-windows-x86_64.zip"
$expected = ((Get-Content -LiteralPath "$archive.sha256") -split "\s+")[0]
$actual = (Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash.ToLowerInvariant()
if ($actual -ne $expected) { throw "SHA-256 mismatch" }
```

校验和或内容清单缺失、不匹配时，不要安装该归档。

## 安装

解压后的 `recoverable_delete` 是完整的本地 Marketplace 根目录，不只是 Plugin 文件夹。Codex 会跟踪这个 Marketplace 源，因此需要把它保存在稳定路径。

macOS 和 Linux 示例：

```sh
install_root="$HOME/.local/share/recoverable_delete/v0.1.0"
mkdir -p "$install_root"
tar -xzf recoverable-delete-macos-arm64.tar.gz -C "$install_root"
codex plugin marketplace add "$install_root/recoverable_delete"
codex plugin add recoverable-delete@tinkora
```

Linux 上请换用 Linux 归档名称。

Windows PowerShell 示例：

```powershell
$installRoot = Join-Path $env:LOCALAPPDATA "recoverable_delete\v0.1.0"
New-Item -ItemType Directory -Path $installRoot -Force | Out-Null
Expand-Archive -LiteralPath "recoverable-delete-windows-x86_64.zip" -DestinationPath $installRoot
codex plugin marketplace add "$installRoot\recoverable_delete"
codex plugin add recoverable-delete@tinkora
```

使用新的空版本目录，不要覆盖旧的解压版本。

重启 ChatGPT/Codex 桌面端，新建任务，打开 `/hooks`，检查 `recoverable-delete` 的 `PreToolUse` 命令并明确授予信任。安装 Plugin 不会自动信任 Hook。

## 恢复行为

Hook 负责阻止已识别的物理删除路径，不维护私有备份，也不会自行移动文件。Agent 必须改用操作系统废纸篓或回收站重试。

- macOS：使用 `/usr/bin/trash <精确路径>`。系统命令不接受 `--` 作为参数终止符。
- Linux：优先使用 `gio trash <精确路径>`，否则使用 `trash-put <精确路径>`。
- Windows：使用可信的回收站工具或 `Microsoft.VisualBasic.FileIO` API。

通过 Finder 废纸篓、Linux 桌面废纸篓或 Windows 回收站恢复文件。不要自动清空这些位置。

## 升级

1. 验证新版本并解压到新的版本目录。
2. 运行 `codex plugin remove recoverable-delete@tinkora`。
3. 运行 `codex plugin marketplace remove tinkora`。
4. 添加新解压的 Marketplace 路径并重新安装 Plugin。
5. 重启应用，重新检查 Hook 信任并验证新版本。
6. 验证完成后，把旧版本解压目录移入操作系统废纸篓或回收站。

## 卸载

```sh
codex plugin remove recoverable-delete@tinkora
codex plugin marketplace remove tinkora
```

重启应用并确认 Hook 已消失，然后把解压的 Marketplace 目录移入废纸篓或回收站。卸载 Plugin 不会清空操作系统回收位置，也不会影响之前移入其中的文件。

## 安全边界

这个 Plugin 是安全护栏，不是操作系统级安全边界。它能识别常见 shell 命令、包装器、内联运行时删除 API 和 `apply_patch` 文件删除。任意原生程序、特殊工具路径、Hook opt-out、Codex 之外的操作或主动绕过 Hook，仍可能永久删除数据。必须继续保留备份和正常的操作系统权限控制。

Marketplace 本地安装模型见 [OpenAI 官方 Plugin 打包文档](https://developers.openai.com/plugins/build/plugins)。
