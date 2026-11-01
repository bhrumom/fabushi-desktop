-- Disposable Actions only: confirm the exact installed process's native-code prompt.
on run argv
  set appPid to (item 1 of argv) as integer
  set answerLabel to item 2 of argv
  if answerLabel is not "允许" and answerLabel is not "取消" then error "INVALID_NATIVE_CONFIRM_ACTION"
  tell application "System Events"
    if not UI elements enabled then error "MACOS_UI_AUTOMATION_NOT_CONFIGURED"
    set appProcess to first application process whose unix id is appPid
    tell appProcess
      set frontmost to true
      repeat 200 times
        repeat with candidateWindow in windows
          set candidatePanel to candidateWindow
          if exists sheet 1 of candidateWindow then set candidatePanel to sheet 1 of candidateWindow
          if exists button answerLabel of candidatePanel then
            set promptFound to false
            repeat with labelText in static texts of candidatePanel
              if value of labelText contains "小程序请求本机执行权限" then set promptFound to true
            end repeat
            if promptFound then
              click button answerLabel of candidatePanel
              return "native-code-confirmation=" & answerLabel
            end if
          end if
        end repeat
        delay 0.1
      end repeat
      error "EXPECTED_NATIVE_CODE_PERMISSION_PROMPT_MISSING"
    end tell
  end tell
end run
