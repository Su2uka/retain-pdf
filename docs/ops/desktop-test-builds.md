# 分支测试安装包

在 GitHub Actions 的 **Build Desktop Packages** 中选择 **Run workflow**，
指定修复分支和测试版本号，例如 `4.2.6-continuity.1`。命令行也可以触发：

```bash
gh workflow run build-desktop-packages.yml --ref codex/fix-translation-continuity -f version=4.2.6-continuity.1
```

手动构建并行执行完整 Tests 门禁和 Windows、Linux、macOS 打包，复用正式发版的安装包构建步骤。
确认门禁和目标平台构建都成功后，在该 run 的 Artifacts 中下载
`desktop-<platform>-<commit>`；对应的 `desktop-meta-<platform>-<commit>` 包含版本、
提交和文件 SHA256。产物保留 7 天。这条入口只上传构建产物，不创建 Release 或更新 main。

验证工作流变更时运行 `python -m pytest -q .github/scripts/tests`，并在 Actions 中核对构建提交、
平台结果和产物清单。若需撤销手动入口，回退添加该入口的 CI 提交即可，原有预构建和 tag 发版调用继续保留。
