#!/bin/zsh
# shot.sh <进程名或窗口名关键字> <输出.png>
# 依据 winid.swift 找到窗口 ID,用 screencapture 截取该窗口
set -e
KEY="$1"
OUT="$2"
DIR="$(cd "$(dirname "$0")" && pwd)"
LINE=$("$DIR/../scripts/winid.swift" 2>/dev/null | grep -i "$KEY" | head -1)
if [ -z "$LINE" ]; then
  echo "NOT_FOUND: $KEY"
  exit 2
fi
WID="${LINE%%|*}"
echo "window: $LINE"
screencapture -o -x -l "$WID" "$OUT"
ls -la "$OUT"
