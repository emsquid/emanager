# emanager

This is a simple Rust CLI that manages a few things for me, 
you probably don't want to use it except if you are planning to replicate my [dotfiles](https://github.com/emsquid/dotfiles)

## Requirements

- [Git](https://git-scm.com/downloads)
- [Rust toolchain](https://www.rust-lang.org/tools/install)
- [Hyprland](https://hyprland.org)
- [Eww](https://elkowar.github.io/eww)
- [Brightnessctl](https://github.com/Hummer12007/brightnessctl)
- [WirePlumber](https://pipewire.pages.freedesktop.org/wireplumber)
- [NetworkManager](https://wiki.archlinux.org/title/NetworkManager)
- [Libnotify](https://gitlab.gnome.org/GNOME/libnotify)
- [Acpid](https://wiki.archlinux.org/title/Acpid)

## Usage

```
Usage: emanager <COMMAND>

Commands:
  daemon      Launch manager daemon
  system      Commands to manage systemd
  brightness  Commands to manage backlight
  volume      Commands to manage volume
  layout      Change layout
  help        Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```
