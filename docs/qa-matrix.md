# QA Matrix

## Mandatory Desktop Matrix

- GNOME on X11
- GNOME on Wayland
- KDE Plasma on X11
- KDE Plasma on Wayland

## Core Manual Checks

- App starts hidden and remains active in the background.
- Tray click opens picker and focus lands in search.
- Closing or blurring the picker hides it without exiting the app.
- Copying text creates one history item and does not churn duplicates every poll cycle.
- Selecting an item copies it back to the clipboard and hides the picker.
- Toggling autostart persists across restart.
- Updating hotkey persists across restart.

## Stress Checks

- Rapid text copy changes do not crash the app.
- Reopening the app does not corrupt the SQLite database.
- History search remains responsive with at least 1,000 text entries.
