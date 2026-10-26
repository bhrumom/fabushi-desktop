-- Test-only: run exclusively on the ephemeral macOS Actions runner.
-- Select a real Save panel for the exact signed process, never bypass its policy.
on run argv
  set appPid to (item 1 of argv) as integer
  set reportDirectory to item 2 of argv
  set expectedName to "global-dandelion-session.json"
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
              set reportWindowName to name of candidateWindow
              set saveSheet to sheet 1 of window reportWindowName
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
      set goSheet to missing value
      repeat 100 times
        repeat with candidateElement in UI elements
          try
            if (value of attribute "AXIdentifier" of candidateElement) is "GoToWindow" then
              set goSheet to contents of candidateElement
              exit repeat
            end if
          end try
        end repeat
        if goSheet is not missing value then exit repeat
        if exists sheet 1 of saveSheet then
          set goSheet to sheet 1 of saveSheet
          if exists combo box 1 of goSheet then exit repeat
          if exists text field 1 of goSheet then exit repeat
        end if
        delay 0.1
      end repeat
      if goSheet is missing value then error "REPORT_DIRECTORY_PANEL_MISSING"
      -- A second Save panel may remember the first destination. Replace it;
      -- never append to a previous path or type before the nested panel exists.
      set pathField to text field 1 of goSheet
      if (value of attribute "AXIdentifier" of pathField) is not "PathTextField" then error "REPORT_DIRECTORY_FIELD_MISMATCH"
      click pathField
      set value of pathField to reportDirectory
      set directoryInputReady to false
      repeat 100 times
        set enteredDirectory to value of pathField
        if enteredDirectory is reportDirectory or enteredDirectory is (reportDirectory & "/") then
          set directoryInputReady to true
          exit repeat
        end if
        delay 0.1
      end repeat
      if not directoryInputReady then
        set observedPath to enteredDirectory as text
        if observedPath starts with "/Users/runner/" and (count characters of observedPath) < 512 then
          error "REPORT_DIRECTORY_INPUT_MISMATCH observed=" & observedPath
        end if
        error "REPORT_DIRECTORY_INPUT_MISMATCH length=" & (count characters of observedPath)
      end if
      key code 36
      set directoryChosen to false
      repeat 100 times
        if not (exists goSheet) then
          set directoryChosen to true
          exit repeat
        end if
        -- macOS resolves Go-to-folder suggestions asynchronously; Return while
        -- resolution is pending can be ignored. Confirm only while this nested
        -- navigation panel still exists, never press Return on the Save panel.
        key code 36
        delay 0.1
      end repeat
      if not directoryChosen then error "REPORT_DIRECTORY_SELECTION_INCOMPLETE"
      if (value of text field 1 of saveSheet) is not expectedName then error "REPORT_FILENAME_CHANGED"
      click button "Save" of saveSheet
      repeat 100 times
        if not (exists saveSheet) then return "Exact report Save panel dismissed"
        delay 0.1
      end repeat
      error "REPORT_SAVE_PANEL_NOT_DISMISSED"
    end tell
  end tell
  return "Exact report save intent completed"
end run
