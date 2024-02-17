# y1337 - tui file management

Yet Another Astoundingly Hackable, Keyboard-Controlled, Ultra-Intuitive,
Efficient, Versatile, Interactive, Fast, Elmish, Minimalistic, and Superlative
File Explorer with Vim-Inspired Keybindings, Designed to Revolutionize Terminal
Productivity, and Unleash Unprecedented Efficiency, All While Honoring the Spirit
of Open Source Collaboration and Embracing the Beauty, and Greatness of Rust Programming
Language, Crafted with Passion, Precision, Pain, and Perseverance, Aiming to Reshape
the Way We Navigate and Manipulate Files in the Digital Realm, Pioneering a New
Era of Command-Line Exploration and Empowering Users to Harness the Full Potential
of Their Command Shells, Forever Changing the Landscape of File Interaction, and
Leaving a Legacy Worthy of Legends, Echoing Through the Halls of Cyberspace and
Inspiring Generations of Developers Yet to Come, A Testament to Human Ingenuity
and the Endless Quest for Excellence, a Journey Beyond the Cursor, and a Beacon
of Innovation in the Vast Digital Wilderness, Infused with the Magic of Lua, Allowing
Users to Extend Its Functionality, Shape Its Behavior, and Create Customized Workflows
Tailored to Their Unique Needs, Enabling a Thriving Ecosystem of Community-Driven
Plugins, Where Imagination Meets Practicality, and Every Keystroke Holds the Promise
of Infinite Possibilities.

## architecture overview

### y1337

The main crate is handling frontend and backend and resolves cli arguments to
pass them to the relevant components.

### y1337-frontend

The frontend follows an elm architecture with one exception: The model is
mutable and will not get created every update.

frontend.rs holds the lifecycle of the tui. It starts an event stream to
enable non lockable operations. This stream is implemented in event.rs and
translates multiple event emitter like terminal interaction with crossterm into
AppEvents.

layout.rs defines the overall app layout, which is used by all view functions.

The modules model, update and view represent the elm philosophy. Messages
are defined in y1337-keymap to prevent cycling dependencies.

### y1337-keymap

This crate holds all key relevant features. The MessageResolver uses buffer
and tree to resolve possible messages, which follow the elm architecture to
modify the model.

tree uses the keymap to build a key tree structure. Thus, in keymap all
key combinations are mapped indirectly to messages.

conversion translates crossterm key events to the y1337-keymap
representation.
