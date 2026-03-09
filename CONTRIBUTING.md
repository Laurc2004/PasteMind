# Contributing

## 中文

### 开发流程

1. Fork 并创建功能分支。
2. 在本地运行 `pnpm install`。
3. 开发时运行 `pnpm dev`。
4. 提交前执行 `pnpm test` 与 `pnpm lint`。
5. 提交 PR，描述问题、方案和验证结果。

### 代码约定

- TypeScript 和 Rust 代码保持可读性优先。
- 仅在复杂逻辑处写注释，避免噪音。
- 新增功能必须补充对应测试（至少单元测试）。
- 涉及权限或隐私行为，必须更新文档。

## English

### Development workflow

1. Fork and create a feature branch.
2. Run `pnpm install` locally.
3. Start with `pnpm dev`.
4. Before pushing, run `pnpm test` and `pnpm lint`.
5. Open a PR with problem statement, implementation notes, and test evidence.

### Code standards

- Optimize for readability in both TypeScript and Rust.
- Keep comments for non-obvious logic only.
- Add tests for new behavior.
- Update docs whenever permissions/privacy behavior changes.
