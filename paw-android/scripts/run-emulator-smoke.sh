#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REPORT_DIR="${ROOT_DIR}/build/reports/android-smoke"
mkdir -p "${REPORT_DIR}"

DEVICE_SERIAL="${ANDROID_SERIAL:-$(adb devices | awk 'NR>1 && $2 == "device" { print $1; exit }')}"
if [[ -z "${DEVICE_SERIAL}" ]]; then
  echo "No Android emulator/device connected." >&2
  exit 1
fi

echo "Using device: ${DEVICE_SERIAL}" | tee "${REPORT_DIR}/device.txt"

pushd "${ROOT_DIR}" >/dev/null
./gradlew :app:assembleDebug :app:connectedDebugAndroidTest | tee "${REPORT_DIR}/connectedDebugAndroidTest.log"
adb -s "${DEVICE_SERIAL}" install -r app/build/outputs/apk/debug/app-debug.apk | tee "${REPORT_DIR}/install.log"
adb -s "${DEVICE_SERIAL}" shell am start -n dev.paw.android/dev.paw.android.MainActivity | tee "${REPORT_DIR}/am-start.log"
adb -s "${DEVICE_SERIAL}" exec-out screencap -p > "${REPORT_DIR}/emulator-smoke.png"
adb -s "${DEVICE_SERIAL}" shell dumpsys activity activities dev.paw.android > "${REPORT_DIR}/dumpsys-activity.txt"
popd >/dev/null

echo "Smoke artifacts written to ${REPORT_DIR}"
