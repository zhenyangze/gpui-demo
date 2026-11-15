#!/bin/zsh
# 补采缺失截图:仅当屏幕解锁(存在非 loginwindow 的应用窗口)时执行
cd "$(dirname "$0")/.."
# 屏幕锁定检测:若只有 Window Server/loginwindow/Bartender 等,视为锁定
LOCKED=$(swift scripts/winid.swift 2>/dev/null | grep -v -E "Window Server|loginwindow|Bartender|Item-0|Menubar|MenuBarCover|BentoBox|Clock\|" | grep -c "|")
if [ "$LOCKED" -lt 3 ]; then
  echo "[capture] screen still locked, skip"
  exit 0
fi
echo "[capture] screen unlocked, capturing..."

shoot() {
  local APP=$1 ARG=$2 OUT=$3
  [ -s "shots/$OUT" ] && { echo "[capture] $OUT exists, skip"; return; }
  (./target/debug/$APP $ARG &)
  sleep 4
  osascript -e "tell application \"System Events\" to set frontmost of first process whose name is \"$APP\" to true" 2>/dev/null
  sleep 1
  WID=$(swift scripts/winid.swift $APP | head -1 | cut -d'|' -f1)
  screencapture -o -x -l "$WID" "shots/$OUT" 2>/dev/null && echo "[capture] $OUT done" || echo "[capture] $OUT failed"
  pkill -f "target/debug/$APP" 2>/dev/null
  sleep 0.6
}

shoot demo-game2048 "over"  game_over.png
shoot demo-game2048 "win"   game_win.png
shoot demo-devtoolbox ""    toolbox_json.png
shoot demo-devtoolbox regex toolbox_regex.png
shoot demo-devtoolbox color toolbox_color.png
shoot demo-devtoolbox hash  toolbox_hash.png
shoot demo-gallery  "feedback" gallery_feedback.png
shoot demo-gallery  "about"    gallery_about.png
echo "[capture] all done"
