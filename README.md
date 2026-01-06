# tiny-time-tracker (TTT)
a very tiny time tracker for any executable binary on Windows.

<img width="1604" height="552" alt="image" src="https://github.com/user-attachments/assets/7f7de691-5de1-4ea9-b6f3-20760e216287" />

# Notes
TTT does not poll target processes and instead relies on Windows security and Task Scheduler to track when a process starts and stops. Because of this TTT has near 0 footprint, only running once when a tracked program is launched and once when the tracked program stops.

Windows Home edition does not come with GroupPolicy tools installed, they need to be installed seperately. Running `activate.bat` will install these necessary tools directly from Microsoft and enable the necessary policies.

# Implementation
- `tiny-time-tracker.exe` is used to host the WebUI. This is for adding/editing/deleting tracked tasks as well as viewing collected data
- `trigger.exe` writes playtime to the database. It will run when a tracked app starts/stops

Essentially a Scheduled Task is created when you add a game to the WebUI to run `trigger.exe` when you run something, and then run it again when it stop. The `trigger.exe` application then writes this data to the database, which is later read back by `tiny-time-tracker.exe`.

`trigger.exe` only runs for a single moment to write to the database then terminates. You also do not need to run the web-ui unless you are viewing/editing, tracking will continue to work. As such there is negligible performance impact all around.

# Building
```
cargo build --release
```

# Usage
1. Enable GroupPolicy tool. Run `activate.bat`
2. Run `tiny-time-tracker.exe` as Administrator, then close it. A .db file will be created
3. Make a copy of `.env.template`, name it `.env`
5. Fill in the path to `trigger.exe` as well as the new `ttt.db` file. (You can relocate these if you want)

After this you can launch `tiny-time-tracker.exe` and visit `http://localhost:3000` to add a new tracked app.
