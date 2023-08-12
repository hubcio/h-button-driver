
## Fix

- remove Mutex from callbacks
- reset led value when uC is rebooted (set to 0) DONE
- fix logger - doesn't work from sound module
- add thread, which will poll sound module and update led value
- handle D-BUS error - restart driver
- use alsa capture switch instead of setting volume DONE https://github.com/xkr47/push-to-talk-xcb-alsa/blob/main/src/main.rs
- make interface for sound that it takes value from 0 to 100
- windows drivers
- macos drivers
- tray functionality
- bluetooth OTA

## TODO
- 3d printed case
- pcb
