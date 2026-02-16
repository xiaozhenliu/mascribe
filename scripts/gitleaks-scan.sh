#!/usr/bin/env bash
set -euo pipefail

if command -v gitleaks >/dev/null 2>&1; then
  GITLEAKS_BIN="gitleaks"
elif [ -x "/opt/homebrew/bin/gitleaks" ]; then
  GITLEAKS_BIN="/opt/homebrew/bin/gitleaks"
else
  echo "gitleaks is not installed. Install with: brew install gitleaks" >&2
  exit 127
fi

echo "[gitleaks] using: $GITLEAKS_BIN"
"$GITLEAKS_BIN" git --redact --verbose
