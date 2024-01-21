# contributing

## architecture overview

### yate

The main crate is handling frontend and backend and resolves cli arguments to
pass them to the relevant components.

### yate-frontend

The frontend follows an elm architecture with one exception: The model is
mutable and will not get created every update.

`frontend.rs` holds the lifecycle of the tui. It starts an event stream to
enable non lockable operations. This stream is implemented in `event.rs` and
translates multiple event emitter like terminal interaction with crossterm into
AppEvents.

`layout.rs` defines the overall app layout, which is used by all view functions.

The modules `model`, `update` and `view` represent the elm philosophy. Messages
are defined in `yate-keymap` to prevent cycling dependencies.

### yate-keymap

This crate holds all key relevant features. The `MessageResolver` uses `buffer`
and `tree` to resolve possible `message`s, which follow the elm architecture to
modify the model.

`tree` uses the `keymap` to build a `key` tree structure. Thus, in `keymap` all
key combinations are mapped indirectly to `messages`.

`conversion` translates crossterm key events to the `yate-keymap`
representation.

### yate-config (not even started..)

`yate-config` will handle all configurations like custom key bindings, themes,
and... Who knows...

### yate-server (not even started..)

The server component will allow to share state with multiple yate instances and
enable asyncronous long running tasks with progress indication.
