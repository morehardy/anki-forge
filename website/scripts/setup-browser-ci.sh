#!/usr/bin/env bash
set -euo pipefail

# GitHub's Azure HTTP mirror stalled on font packages in run 40337178616.
# Keep the browser dependency set, but use Ubuntu's official HTTPS archive.
if [[ -f /etc/apt/apt-mirrors.txt ]]; then
  printf '%s\n' 'https://archive.ubuntu.com/ubuntu/' | sudo tee /etc/apt/apt-mirrors.txt >/dev/null
fi
npx playwright install --with-deps chromium
sudo apt-get install -y fonts-noto-cjk
