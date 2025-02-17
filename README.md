# tiny-time-tracker (TTT)
a very tiny time tracker for any executable binary on Windows.

TTT does not poll target processes and instead relies on Windows security and Task Scheduler to track when a process starts and stops. Because of this TTT has near 0 footprint, only running once when a tracked program is launched and once when the tracked program stops.

Windows Home edition does not come with GroupPolicy tools installed, they need to be installed seperately. Running `activate.bat` will install these necessary tools directly from Microsoft and enable the necessary policies.