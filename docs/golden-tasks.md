# Golden tasks v1

## 1. open youtube
Classification: mixed_workflow
Strategy:
- resolve installed browsers locally
- present route choices
- open selected route after Y
Risk: R0
Approval: required only for route selection, not for danger
Inverse: none

## 2. open slack
Classification: app_control
Strategy: local app resolver
Risk: R0
Approval: no
Inverse: none

## 3. open safari / open chrome / open firefox / open brave / open arc
Classification: app_control
Strategy:
- parse explicit browser target locally
- resolve only installed matching browser from machine scan
- launch directly via approved bundle id
Risk: R0
Approval: no
Inverse: none

## 4. mute the mac
Classification: local_system
Strategy: internal rust or applescript template
Risk: R1
Approval: no
Inverse: restore previous volume

## 5. set volume to 30 percent
Classification: local_system
Strategy: internal rust or applescript template
Risk: R1
Approval: no
Inverse: restore previous volume

## 6. open display settings
Classification: local_system
Strategy: known settings deep-link/template
Risk: R0
Approval: no
Inverse: none

## 7. reveal downloads
Classification: filesystem
Strategy: local known path + open/reveal
Risk: R0
Approval: no
Inverse: none

## 8. move file to archive folder
Classification: filesystem
Strategy: validated move template
Risk: R1
Approval: yes if path target is ambiguous
Inverse: move back

## 9. rename png files in folder to snake_case
Classification: filesystem
Strategy: validated batch rename template
Risk: R2
Approval: yes
Inverse: yes, persist original names map

## 10. join zoom meeting
Classification: ui_automation
Strategy:
- activate zoom
- prefer scriptable path
- fallback to System Events selector path
Risk: R2
Approval: yes
Inverse: none

## 11. run tests in current repo
Classification: shell_execution
Strategy: repo-local approved template only
Risk: R1
Approval: yes in v1
Inverse: none
