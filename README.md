# XDG Background Desktop Portal

> Note: this interface is early in development and is not stable yet.

This application provides a desktop agnostic implementation to register Autostart applications.
To do that, this program implements the `org.freedesktop.impl.portal.Background` portal, normally provided by GNOME or KDE.

# Usage

> **Important:** for an installation guide see [Wiki](TODO)

This portal can be enabled in your current setup by adding `org.freedesktop.impl.portal.Background=background` to your `<de>-portals.conf`.

For Hyprland this would look like:

```conf
# File: ~/.config/xdg-desktop-portal/hyprland-portals.conf or /usr/share/xdg-desktop-portal/hyprland-portals.conf
[preferred]
default=hyprland;gtk
org.freedesktop.impl.portal.Background=background
```

# Limitations

Can not track if an application has a visible or open window.
As a result it will always report applications as running in background.
In addition, this application does not start registered applications on startup.
This can be done by using [dex](https://github.com/jceb/dex) or similar applications.

# To-Do

- [ ] Send system notifications to request background execution.
- [x] Use autostart directory
- [ ] Handle D-Bus activatable applications (Handle Autostart flags)
