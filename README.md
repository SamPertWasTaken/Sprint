# Sprint
A featureful wayland search tool for Linux, letting you open applications, do simple calculations and quickly search the web.

![A preview GIF of Sprint working](https://i.imgur.com/pXx97E4.gif)

## Installing
To install, simply clone the repo and install via `cargo install --path .`.  
Ensure that `~/.cargo/bin` is included in your `$PATH`.

## Usage
Just invoke the `sprint` binary by setting it as a keybind in your window manager. For example with Hyprland;
```
bind = SUPER, R, exec, ~/.cargo/bin/sprint
```

## Configuration
Sprint will always ensure a config file exists, either in `$XDG_CONFIG_HOME/sprint.toml` or `$HOME/.config/sprint.toml` if `XDG_CONFIG_HOME` is not set.  
The comments inside the config file should keep you right as you modify it.
