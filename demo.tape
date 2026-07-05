# git-rewind demo tape
# Run from the git-rewind repo root:
#   vhs demo.tape

Output assets/demo.gif

Set Shell "powershell"
Set FontSize 14
Set Width 1200
Set Height 650
Set Theme "Dracula"
Set Padding 20
Set TypingSpeed 80ms
Set PlaybackSpeed 0.8

# ── Start in the repo ──────────────────────────────────────────────────────────
Type "cd C:\Users\Gagandeep\project\git-rewind"
Enter
Sleep 500ms

# ── Show what we're about to launch ────────────────────────────────────────────
Type ".\target\debug\git-rewind-ui.exe"
Enter
Sleep 1.5s          # wait for TUI to paint

# ── Navigate down the timeline ─────────────────────────────────────────────────
Down
Sleep 600ms
Down
Sleep 600ms
Down
Sleep 600ms

# ── Jump to first entry ────────────────────────────────────────────────────────
Type "g"
Sleep 700ms

# ── Jump to last entry ─────────────────────────────────────────────────────────
Type "G"
Sleep 700ms

# ── Navigate back up a few steps ───────────────────────────────────────────────
Up
Sleep 400ms
Up
Sleep 400ms

# ── Trigger a rewind (will show dirty warning or mode picker) ──────────────────
Type "r"
Sleep 1s

# ── Cancel cleanly — don't actually rewind in the demo ────────────────────────
Type "c"
Sleep 700ms

# ── Quit ───────────────────────────────────────────────────────────────────────
Type "q"
Sleep 500ms
