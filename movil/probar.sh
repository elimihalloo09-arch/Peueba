#!/usr/bin/env bash
# Revisa que el codigo este sano antes de dar por bueno un cambio.
set -euo pipefail
export PATH="$HOME/flutter/bin:$PATH"
cd "$(dirname "$0")"
flutter analyze
flutter test
