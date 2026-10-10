# SearchForge — Fix Full Window Centering and Spotlight Positioning

## Objective

Fix the window positioning in the existing SearchForge production application.

**Do not redesign the UI. Do not modify the search engine, indexing, previews, or existing visual styling.**

The current Full SearchForge window appears too far toward the bottom-right instead of being centered on the screen.

This must be fixed at the native window-positioning level.

## 1. Full SearchForge — Center on Screen

When Full SearchForge opens, its outer window must be centered horizontally and vertically on the correct monitor.

Calculate the position using the actual monitor work area and the final window dimensions:

- `x = work_area.x + (work_area.width - window.width) / 2`
- `y = work_area.y + (work_area.height - window.height) / 2`

Use the actual monitor coordinates, including the monitor's offset in multi-monitor configurations.

Do not assume the primary monitor starts at `(0, 0)`.

### Important

- Center the actual native window, not its internal egui content.
- Account for the Windows taskbar and monitor work area.
- Apply positioning after the intended window dimensions are established.
- Avoid positioning the window before its dimensions are known.
- Do not compensate with arbitrary pixel offsets.
- Do not repeatedly reposition the window every frame.

## 2. Spotlight — Preserve Its Own Positioning

Spotlight must remain compact and centered on the appropriate screen.

Do not reuse Full SearchForge's window dimensions when positioning Spotlight.

Each mode must use its own intended dimensions and positioning logic.

## 3. Diagnose the Root Cause

Inspect the existing code before changing anything.

Investigate:

- Native window creation and initial placement.
- `eframe` viewport configuration.
- Windows DPI scaling and display coordinates.
- Saved window positions and persisted geometry.
- Startup-on-login behavior.
- Window restoration and mode switching.
- Any manual positioning code or hardcoded offsets.

Determine why the window appears bottom-right and fix the underlying cause.

Do not merely move it slightly toward the center.

## 4. Multi-Monitor and DPI Support

Verify correct positioning when:

- Windows uses display scaling such as 100%, 125%, or 150%.
- The primary monitor is not the monitor where the window should appear.
- Monitors have different resolutions.
- The application starts automatically with Windows.
- The window is reopened after being hidden.
- Spotlight switches to Full SearchForge.

Use consistent coordinate units. Do not mix physical pixels, logical points, and DPI-scaled coordinates.

## 5. Acceptance Criteria

- [ ] Full SearchForge opens exactly centered on the intended monitor.
- [ ] Centering accounts for the Windows work area.
- [ ] Spotlight remains independently centered.
- [ ] No arbitrary positioning offsets.
- [ ] No visible jumping or flickering during startup.
- [ ] No repeated position corrections during rendering.
- [ ] Startup-on-login still works.
- [ ] `Alt + K` still opens Spotlight.
- [ ] `Esc` still dismisses Spotlight without terminating SearchForge.
- [ ] Switching between modes does not break positioning.
- [ ] Existing UI styling and functionality remain unchanged.
- [ ] `cargo check`, `cargo test`, and `cargo clippy` pass.

## Final instruction

Reproduce the positioning bug, inspect the actual window-management code, fix the root cause, and verify the result by launching the production build.

**The requirement is simple: Full SearchForge must open in the center of the screen, not toward the bottom-right.** Do not declare completion based on code inspection alone.
