#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
VIDEO_TOOL="$SCRIPT_DIR/fcm-010-13-macos-session-video.swift"
M2_SCALER_DRIVER='AppleM2ScalerParavirtDriver'
CURRENT_PARAVIRT_DISPLAY='AppleParavirtDisplay'
CAPTURE_INTERVAL_SECONDS="${FCM_MACOS_RECORDING_INTERVAL_SECONDS:-1}"
MIN_VIDEO_BYTES="${FCM_MACOS_RECORDING_MIN_BYTES:-100000}"

usage() {
  echo "usage: $0 preflight|start|capture-loop|assert-live|stop|verify <evidence-dir>" >&2
  exit 64
}

require_macos_tools() {
  test "$(uname -s)" = Darwin
  test -x /usr/sbin/screencapture
  test -x /usr/sbin/system_profiler
  test -x /usr/sbin/ioreg
  command -v swift >/dev/null
  command -v jq >/dev/null
  [[ "$MIN_VIDEO_BYTES" =~ ^[0-9]+$ ]]
  test "$MIN_VIDEO_BYTES" -gt 0
  test -s "$VIDEO_TOOL"
}

frame_count() {
  local frames="$1/session-frames"
  if ! test -d "$frames"; then
    echo 0
    return
  fi
  /usr/bin/find "$frames" -type f -name '*.jpg' -print | wc -l | tr -d ' '
}

validate_video() {
  local video="$1"
  local output="$2"
  if ! test -s "$video"; then
    echo 'video validation failed: movie is missing or empty' > "$output"
    return 61
  fi
  if ! swift "$VIDEO_TOOL" validate "$video" > "$output" 2>&1; then
    return 62
  fi
  if ! grep -Eq '^playable=true duration_seconds=[0-9]+([.][0-9]+)? video_tracks=[1-9][0-9]* first_sample=decoded$' "$output"; then
    echo 'video validation failed: validator did not emit the exact success record' >> "$output"
    return 63
  fi
}

collect_display_environment() {
  local evidence="$1"
  local profiler="$evidence/recorder-system-profiler-display.txt"
  local m2_ioreg="$evidence/recorder-ioreg-m2-scaler-driver.txt"
  local paravirt_ioreg="$evidence/recorder-ioreg-paravirt-display.txt"
  local display_ioreg="$evidence/recorder-ioreg-display-lines.txt"
  local combined="$evidence/recorder-display-environment.txt"

  /usr/sbin/system_profiler SPDisplaysDataType > "$profiler" 2>&1 || true
  /usr/sbin/ioreg -r -c "$M2_SCALER_DRIVER" -l > "$m2_ioreg" 2>&1 || true
  /usr/sbin/ioreg -r -c "$CURRENT_PARAVIRT_DISPLAY" -l > "$paravirt_ioreg" 2>&1 || true
  /usr/sbin/ioreg -lw0 2>&1 \
    | grep -E 'AppleM2ScalerParavirtDriver|AppleParavirtDisplay|AppleParavirtGPU|AppleParavirt|IODisplay|DisplayVendorID|DisplayProductID|framebuffer|Framebuffer' \
    > "$display_ioreg" || true

  {
    echo 'schema=fabushi.macos-session-recorder-environment.v1'
    echo "uname=$(uname -a)"
    echo "runner_os=${RUNNER_OS:-unknown}"
    echo "runner_arch=${RUNNER_ARCH:-unknown}"
    echo '--- sw_vers ---'
    /usr/bin/sw_vers 2>&1 || true
    echo '--- system_profiler SPDisplaysDataType ---'
    cat "$profiler"
    echo '--- ioreg AppleM2ScalerParavirtDriver ---'
    cat "$m2_ioreg"
    echo '--- ioreg AppleParavirtDisplay ---'
    cat "$paravirt_ioreg"
    echo '--- ioreg display-related lines ---'
    cat "$display_ioreg"
  } > "$combined"
  test -s "$combined"
}

native_probe() {
  local evidence="$1"
  local video="$evidence/native-recorder-probe.mov"
  local log="$evidence/native-recorder-probe.log"
  local validation="$evidence/native-recorder-probe-validation.txt"
  rm -f "$video" "$log" "$validation"

  set +e
  /usr/sbin/screencapture -v -V 2 -D 1 "$video" > "$log" 2>&1 &
  local pid=$!
  local finished=false
  for _ in $(seq 1 10); do
    if ! kill -0 "$pid" 2>/dev/null; then
      finished=true
      break
    fi
    sleep 1
  done
  if test "$finished" != true; then
    echo 'native probe exceeded its bounded completion window' >> "$log"
    kill -INT "$pid" 2>/dev/null || true
    sleep 1
    kill -TERM "$pid" 2>/dev/null || true
  fi
  wait "$pid" 2>/dev/null
  local native_exit=$?
  set -e
  printf '%s\n' "$native_exit" > "$evidence/native-recorder-probe-exit.txt"

  if test -s "$video" && validate_video "$video" "$validation"; then
    printf '%s\n' true > "$evidence/native-recorder-probe-playable.txt"
  else
    printf '%s\n' false > "$evidence/native-recorder-probe-playable.txt"
    test -s "$validation" || echo 'playable=false native probe produced no decodable movie' > "$validation"
  fi
}

fallback_probe() {
  local evidence="$1"
  local frames="$evidence/fallback-recorder-probe-frames"
  local video="$evidence/fallback-recorder-probe.mov"
  local validation="$evidence/fallback-recorder-probe-validation.txt"
  rm -rf "$frames"
  rm -f "$video" "$validation"
  mkdir -p "$frames"
  for index in 0 1 2; do
    local frame="$frames/$(printf '%03d' "$index").jpg"
    if ! /usr/sbin/screencapture -x -D 1 -t jpg "$frame" > "$evidence/fallback-recorder-probe-screencapture-$index.log" 2>&1; then
      echo "fallback screenshot capture failed at probe frame $index" >&2
      return 51
    fi
    if ! test -s "$frame"; then
      echo "fallback screenshot probe frame $index is empty" >&2
      return 52
    fi
    sleep 0.2
  done
  if ! swift "$VIDEO_TOOL" encode "$frames" "$video" 1 > "$validation" 2>&1; then
    cat "$validation" >&2
    return 53
  fi
  if ! validate_video "$video" "$evidence/fallback-recorder-probe-revalidation.txt"; then
    cat "$evidence/fallback-recorder-probe-revalidation.txt" >&2
    return 54
  fi
  printf '%s\n' true > "$evidence/fallback-recorder-probe-playable.txt"
  rm -rf "$frames"
}

preflight() {
  local evidence="$1"
  require_macos_tools
  mkdir -p "$evidence"
  collect_display_environment "$evidence"

  local m2_scaler=false current_paravirt=false hosted_paravirt=false profiler_present=false
  if grep -Fq "$M2_SCALER_DRIVER" "$evidence/recorder-ioreg-m2-scaler-driver.txt"; then
    m2_scaler=true
  fi
  if grep -Fq "$CURRENT_PARAVIRT_DISPLAY" "$evidence/recorder-ioreg-paravirt-display.txt"; then
    current_paravirt=true
  fi
  if test "$m2_scaler" = true || test "$current_paravirt" = true; then
    hosted_paravirt=true
  fi
  if test -s "$evidence/recorder-system-profiler-display.txt"; then
    profiler_present=true
  fi
  printf '%s\n' "$m2_scaler" > "$evidence/apple-m2-scaler-paravirt-driver.txt"
  printf '%s\n' "$current_paravirt" > "$evidence/apple-paravirt-display.txt"
  printf '%s\n' "$hosted_paravirt" > "$evidence/hosted-paravirt-display.txt"

  native_probe "$evidence"
  if ! fallback_probe "$evidence"; then
    printf '%s\n' false > "$evidence/fallback-recorder-probe-playable.txt"
    jq -n \
      --arg schema 'fabushi.macos-session-recorder-preflight.v1' \
      --arg runnerOs "${RUNNER_OS:-$(uname -s)}" \
      --arg runnerArch "${RUNNER_ARCH:-$(uname -m)}" \
      --arg nativePlayable "$(cat "$evidence/native-recorder-probe-playable.txt")" \
      --argjson m2Scaler "$m2_scaler" \
      --argjson currentParavirt "$current_paravirt" \
      --argjson hostedParavirt "$hosted_paravirt" \
      --argjson profilerPresent "$profiler_present" \
      '{schema:$schema,runnerOs:$runnerOs,runnerArch:$runnerArch,appleM2ScalerParavirtDriver:$m2Scaler,appleParavirtDisplay:$currentParavirt,hostedParavirtDisplay:$hostedParavirt,systemProfilerDisplayDataPresent:$profilerPresent,nativeRecorderPlayable:($nativePlayable == "true"),fallbackRecorderPlayable:false,selectedMode:"none",failClosed:true}' \
      > "$evidence/recorder-preflight.json"
    echo 'No verified playable recording backend is available on this runner; refusing to continue.' >&2
    return 55
  fi

  local native_playable fallback_playable
  native_playable="$(cat "$evidence/native-recorder-probe-playable.txt")"
  fallback_playable="$(cat "$evidence/fallback-recorder-probe-playable.txt")"
  test "$fallback_playable" = true

  jq -n \
    --arg schema 'fabushi.macos-session-recorder-preflight.v1' \
    --arg runnerOs "${RUNNER_OS:-$(uname -s)}" \
    --arg runnerArch "${RUNNER_ARCH:-$(uname -m)}" \
    --arg nativePlayable "$native_playable" \
    --arg fallbackPlayable "$fallback_playable" \
    --arg selectedMode 'frame-avassetwriter' \
    --argjson m2Scaler "$m2_scaler" \
    --argjson currentParavirt "$current_paravirt" \
    --argjson hostedParavirt "$hosted_paravirt" \
    --argjson profilerPresent "$profiler_present" \
    '{schema:$schema,runnerOs:$runnerOs,runnerArch:$runnerArch,appleM2ScalerParavirtDriver:$m2Scaler,appleParavirtDisplay:$currentParavirt,hostedParavirtDisplay:$hostedParavirt,systemProfilerDisplayDataPresent:$profilerPresent,nativeRecorderPlayable:($nativePlayable == "true"),fallbackRecorderPlayable:($fallbackPlayable == "true"),selectedMode:$selectedMode,failClosed:true}' \
    > "$evidence/recorder-preflight.json"

  jq -e '.failClosed == true and .selectedMode == "frame-avassetwriter" and .fallbackRecorderPlayable == true' \
    "$evidence/recorder-preflight.json" >/dev/null
  if test "$hosted_paravirt" = true; then
    echo "Hosted paravirtual display detected (AppleM2ScalerParavirtDriver=$m2_scaler, AppleParavirtDisplay=$current_paravirt); verified frame/AVAssetWriter recorder selected independently of native video probe result=$native_playable." \
      | tee "$evidence/recorder-selection.txt"
  else
    echo "No known hosted paravirtual display class detected; verified frame/AVAssetWriter recorder selected independently of native video probe result=$native_playable." \
      | tee "$evidence/recorder-selection.txt"
  fi
}

capture_loop() {
  local evidence="$1"
  local frames="$evidence/session-frames"
  mkdir -p "$frames"
  rm -f "$evidence/recorder.failed" "$evidence/recorder.stop"
  local index=0
  while ! test -f "$evidence/recorder.stop"; do
    local frame="$frames/$(printf '%07d' "$index").jpg"
    local captured=false
    local attempt
    for attempt in 1 2 3; do
      rm -f "$frame"
      if /usr/sbin/screencapture -x -D 1 -t jpg "$frame" > "$evidence/session-recorder-last-screencapture.log" 2>&1 && test -s "$frame"; then
        captured=true
        break
      fi
      sleep 0.2
    done
    if test "$captured" != true; then
      echo "screencapture failed or produced an empty frame at index $index after 3 attempts" > "$evidence/recorder.failed"
      cat "$evidence/session-recorder-last-screencapture.log" >> "$evidence/recorder.failed" 2>/dev/null || true
      exit 41
    fi
    index=$((index + 1))
    printf '%s\n' "$index" > "$evidence/recorder-frame-count.txt"
    sleep "$CAPTURE_INTERVAL_SECONDS"
  done
}

assert_live() {
  local evidence="$1"
  test -s "$evidence/session-recorder.pid"
  local pid
  pid="$(cat "$evidence/session-recorder.pid")"
  kill -0 "$pid"
  if test -s "$evidence/recorder.failed"; then
    cat "$evidence/recorder.failed" >&2
    cat "$evidence/session-recorder-last-screencapture.log" >&2 2>/dev/null || true
    exit 1
  fi
  local count
  count="$(frame_count "$evidence")"
  test "$count" -ge 2
  printf 'recorder_live=true pid=%s frames=%s\n' "$pid" "$count"
}

start() {
  local evidence="$1"
  preflight "$evidence"
  rm -rf "$evidence/session-frames"
  rm -f "$evidence/recorder.stop" "$evidence/recorder.failed" "$evidence/macos-session.mov"
  mkdir -p "$evidence/session-frames"
  date +%s > "$evidence/recorder-started-epoch.txt"
  RUNNER_TRACKING_ID= nohup bash "$0" capture-loop "$evidence" \
    > "$evidence/session-recorder-capture.log" 2>&1 &
  local pid=$!
  printf '%s\n' "$pid" > "$evidence/session-recorder.pid"
  sleep 3
  assert_live "$evidence"
}

stop() {
  local evidence="$1"
  require_macos_tools
  test -s "$evidence/session-recorder.pid"
  local pid
  pid="$(cat "$evidence/session-recorder.pid")"
  touch "$evidence/recorder.stop"
  for _ in $(seq 1 30); do
    if ! kill -0 "$pid" 2>/dev/null; then
      break
    fi
    sleep 1
  done
  if kill -0 "$pid" 2>/dev/null; then
    echo 'recorder capture loop did not stop within 30 seconds' > "$evidence/recorder.failed"
    kill -TERM "$pid" 2>/dev/null || true
    exit 43
  fi
  if test -s "$evidence/recorder.failed"; then
    cat "$evidence/recorder.failed" >&2
    cat "$evidence/session-recorder-last-screencapture.log" >&2 2>/dev/null || true
    exit 1
  fi

  local count started ended duration movie_bytes
  count="$(frame_count "$evidence")"
  test "$count" -ge 3
  started="$(cat "$evidence/recorder-started-epoch.txt")"
  ended="$(date +%s)"
  duration=$((ended - started))
  if test "$duration" -lt 1; then duration=1; fi

  if ! swift "$VIDEO_TOOL" encode "$evidence/session-frames" "$evidence/macos-session.mov" "$duration" \
    > "$evidence/macos-session-encode-validation.txt" 2>&1; then
    cat "$evidence/macos-session-encode-validation.txt" >&2
    exit 44
  fi
  if ! validate_video "$evidence/macos-session.mov" "$evidence/macos-session-playability.txt"; then
    cat "$evidence/macos-session-playability.txt" >&2
    exit 45
  fi
  movie_bytes="$(stat -f %z "$evidence/macos-session.mov")"
  test "$movie_bytes" -gt "$MIN_VIDEO_BYTES"

  local m2_scaler current_paravirt hosted_paravirt native_playable
  m2_scaler="$(cat "$evidence/apple-m2-scaler-paravirt-driver.txt")"
  current_paravirt="$(cat "$evidence/apple-paravirt-display.txt")"
  hosted_paravirt="$(cat "$evidence/hosted-paravirt-display.txt")"
  native_playable="$(cat "$evidence/native-recorder-probe-playable.txt")"
  jq -n \
    --arg schema 'fabushi.macos-session-recorder.v1' \
    --arg mode 'frame-avassetwriter' \
    --argjson frames "$count" \
    --argjson durationSeconds "$duration" \
    --argjson movieBytes "$movie_bytes" \
    --argjson minimumMovieBytes "$MIN_VIDEO_BYTES" \
    --arg nativePlayable "$native_playable" \
    --argjson m2Scaler "$m2_scaler" \
    --argjson currentParavirt "$current_paravirt" \
    --argjson hostedParavirt "$hosted_paravirt" \
    --arg movie 'macos-session.mov' \
    '{schema:$schema,mode:$mode,frames:$frames,durationSeconds:$durationSeconds,movieBytes:$movieBytes,minimumMovieBytes:$minimumMovieBytes,appleM2ScalerParavirtDriver:$m2Scaler,appleParavirtDisplay:$currentParavirt,hostedParavirtDisplay:$hostedParavirt,nativeRecorderPlayable:($nativePlayable == "true"),movie:$movie,playable:true,firstSampleDecoded:true,failClosed:true}' \
    > "$evidence/recorder-final.json"
  rm -rf "$evidence/session-frames"
}

verify() {
  local evidence="$1"
  require_macos_tools
  test -s "$evidence/recorder-preflight.json"
  test -s "$evidence/recorder-final.json"
  jq -e '.failClosed == true and .fallbackRecorderPlayable == true and .selectedMode == "frame-avassetwriter"' \
    "$evidence/recorder-preflight.json" >/dev/null
  jq -e '.failClosed == true and .mode == "frame-avassetwriter" and .playable == true and .firstSampleDecoded == true and .frames >= 3 and .durationSeconds > 0 and .movieBytes > .minimumMovieBytes' \
    "$evidence/recorder-final.json" >/dev/null
  test -s "$evidence/macos-session.mov"
  test "$(stat -f %z "$evidence/macos-session.mov")" -gt "$MIN_VIDEO_BYTES"
  if ! validate_video "$evidence/macos-session.mov" "$evidence/macos-session-final-validation.txt"; then
    cat "$evidence/macos-session-final-validation.txt" >&2
    exit 46
  fi
}

command="${1:-}"
evidence="${2:-}"
test -n "$command" && test -n "$evidence" || usage
case "$command" in
  preflight) preflight "$evidence" ;;
  start) start "$evidence" ;;
  capture-loop) capture_loop "$evidence" ;;
  assert-live) assert_live "$evidence" ;;
  stop) stop "$evidence" ;;
  verify) verify "$evidence" ;;
  *) usage ;;
esac
