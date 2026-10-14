-- Test-only: run exclusively on the ephemeral macOS Actions runner.
-- Select a real Save panel for the exact signed process, never bypass its policy.
on run argv
  set appPid to (item 1 of argv) as integer
  set reportDirectory to item 2 of argv
  set expectedName to "global-dandelion-local-report.json"
  tell application "System Events"
    if not UI elements enabled then error "MACOS_UI_AUTOMATION_NOT_CONFIGURED"
    set appProcess to first application process whose unix id is appPid
    tell appProcess
      set frontmost to true
      set saveSheet to missing value
      repeat 100 times
        repeat with candidateWindow in windows
          if exists sheet 1 of candidateWindow then
            set candidateSheet to sheet 1 of candidateWindow
            if exists button "Save" of candidateSheet then
              set saveSheet to candidateSheet
              exit repeat
            end if
          end if
        end repeat
        if saveSheet is not missing value then exit repeat
        delay 0.1
      end repeat
      if saveSheet is missing value then error "EXPECTED_REPORT_SAVE_PANEL_MISSING"
      if (value of text field 1 of saveSheet) is not expectedName then error "REPORT_FILENAME_MISMATCH"
      keystroke "g" using {command down, shift down}
      delay 0.3
      keystroke reportDirectory
      key code 36
      delay 0.3
      click button "Save" of saveSheet
    end tell
  end tell
  return "Exact report save intent completed"
end run
