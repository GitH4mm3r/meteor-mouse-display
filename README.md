# meteor-mouse-display

A little mouse movement wrapping display written in bevy. Relative mouse movement is shown in an attractive meteor style. Could be useful for game/design streaming.
Features mouse tracking without the window needing to be in focus. 
Note: It used to use a transparent window but microsoft windows updated something that broke transparency so I'm waiting for bevy or winit to fix. I believe transparency still works in linux.

```sh
# Compile with cargo:
cargo run --bin main
```

Preview: \
![mosuemovegif](https://github.com/GitH4mm3r/meteor-mouse-display/assets/143547743/23719eee-2e5e-4757-9332-ce853043d681)
