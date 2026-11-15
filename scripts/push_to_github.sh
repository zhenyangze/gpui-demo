#!/bin/zsh
# 一键完成 GitHub 推送(前置:仓库已建 或 gh 已登录)
set -e
cd "$(dirname "$0")/.."
REPO="git@github.com:zhenyangze/gpui-demo.git"

echo "[1/3] 检查远端仓库是否已存在..."
if git ls-remote "$REPO" HEAD >/dev/null 2>&1 || git ls-remote "$REPO" >/dev/null 2>&1; then
  echo "  ✓ 仓库存在"
else
  echo "  ✗ 仓库不存在;尝试用 gh 创建..."
  if gh auth status >/dev/null 2>&1; then
    VIS="${1:-private}"   # 可传 public / private
    gh repo create zhenyangze/gpui-demo --"$VIS" --source . --remote origin --push
    echo "  ✓ 已创建($VIS)并推送完成"
    exit 0
  else
    echo "  ✗ gh 未登录。两种解决办法(任选其一):"
    echo "    A) 手动建仓:打开 https://github.com/new 创建空仓库 gpui-demo(勿勾选 README),然后重跑本脚本"
    echo "    B) 登录 gh:  gh auth login  后重跑本脚本"
    exit 1
  fi
fi

echo "[2/3] 推送 main 分支..."
git push -u origin main

echo "[3/3] 完成 ✓ 仓库地址: https://github.com/zhenyangze/gpui-demo"
