#!/bin/zsh
APP_DIR="${0:A:h:h}"
RESOURCES="$APP_DIR/Resources"

export PATH="/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin:$PATH"

PY="/Library/Frameworks/Python.framework/Versions/3.12/bin/python3"

if [ -x "$PY" ]; then
  exec "$PY" "$RESOURCES/veo_upscaler.py"
fi

if command -v python3 >/dev/null 2>&1; then
  exec python3 "$RESOURCES/veo_upscaler.py"
fi

osascript -e 'display dialog "Python 3 не знайдено." buttons {"OK"} default button "OK" with icon caution'
