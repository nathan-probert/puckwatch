# PuckWatch

PuckWatch is a Rust-based command-line tool that keeps you updated on live NHL game scores for your favorite teams.

## How it Works

The script fetches game data from the NHL API and checks the status of games involving your specified favorite teams.

- **Live Game Updates:** If one ofyour favorite teams is currently playing, PuckWatch updates every 10 seconds to provide near real-time score information.
- **No Live Games:** If none of your favorite teams have a game currently live, PuckWatch will check for new game starts every 10 minutes.

This approach ensures you get timely updates during games without unnecessary checks when no relevant games are active. The application maintains its state (e.g., last run time, current game status) in a status file, allowing it to determine when to run next.

Built with Rust, PuckWatch is designed to be efficient, minimizing its impact on system resources.

## Usage with Polybar

You can integrate PuckWatch with Polybar to display NHL scores directly in your status bar. Here's an example of how to configure a Polybar module for PuckWatch:

```ini
[module/puckwatch]
type = custom/script
exec = /path/to/your/puckwatch/target/release/puckwatch # Adjust the path to your PuckWatch executable
interval = 10
```

Place the compiled `puckwatch` binary in a location accessible by your Polybar scripts (e.g., `~/.config/polybar/scripts/` or ensure the path in `exec` is correct) and make it executable (`chmod +x /path/to/your/puckwatch/target/release/puckwatch`).