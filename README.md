<div align="center">

# activate_window

To Find then Activate a specific window under wayland.

</div>

## Usage
```bash
$ activate_window -t "Cloud Music" #Activate the window with title "Cloud Music"
$ activate_window -t "Firefox" -c # Activate the window whose title contains "Firefox" 
$ activate_window -a "Clion" # Activate the window whose app_id is "Clion"
$ activate_window -l # List all app_id of running windows.
```
The most common scenario is to bind a shortcut key to activate a specific window.
