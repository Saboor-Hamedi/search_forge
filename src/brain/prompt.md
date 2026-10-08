


### 1. Fix File Type Detection (The "PPT" Issue)
Currently, PowerPoint files (`.ppt`, `.pptx`) are showing a generic grey "FILE" badge.
*   **Requirement:** Implement logic to parse the file extension from the filename.
*   **Action:** Create a helper function or enum that maps extensions to specific colors and labels.
    *   `.pdf` -> Red badge "PDF"
    *   `.doc`, `.docx` -> Blue badge "DOC"
    *   `.ppt`, `.pptx` -> Orange badge "PPT" (This is currently missing/broken).
    *   `.png`, `.jpg` -> Green badge "IMG"
    *   Others -> Grey badge "FILE"

### 2. Fix Layout Overlap (The "Size Badge" Issue)
In the compact view, the file size badge (e.g., "2.1 MB") is overlapping or sitting too close to the file title, making it unreadable.
*   **Requirement:** The layout must be robust against window resizing.
*   **Action:**
    *   Do not float the size badge over the text.
    *   Use a `ui.horizontal` layout for each row.
    *   Place the File Type Badge on the far left.
    *   Place the Title and Path in the middle. **Crucial:** Use `.truncate()` on the title label so it adds "..." if the text gets too long, preventing it from pushing into the size badge.
    *   Place the Size Badge on the far right, aligned to the right edge. Alternatively, move the size text to the subtitle line (next to the path) in a smaller, muted font color to save vertical space.

### 3. Improve Visual Subtlety and Aesthetics
The current UI looks a bit "default" and cluttered. I want a modern, dark-mode look similar to macOS Spotlight or Raycast.
*   **Styling Requirements:**
    *   **Backgrounds:** Use a very dark grey (e.g., `#1e1e1e` or similar) for the window background, and a slightly lighter grey (e.g., `#2b2b2b`) for the list items.
    *   **Hover/Selection:** When a user hovers over a row, change the background color slightly to indicate interactivity. The currently selected row should have a distinct accent color (like the yellow bar in the screenshot, but maybe a full subtle background highlight).
    *   **Rounding:** Use `egui::Rounding` to make the badges and the main window corners rounded (e.g., `Rounding::same(4.0)` for badges, `Rounding::same(8.0)` for the window).
    *   **Spacing:** Increase the padding inside the list items. The text feels too cramped. Use `ui.add_space()` or `Frame::none().inner_margin(...)` to give elements breathing room.
    *   **Typography:** Make the file title white/bright and bold. Make the file path and size a muted grey (e.g., `Color32::from_gray(150)`).

### 4. Code Structure
Please provide the refactored `egui` code for the list rendering loop.
*   Assume I have a struct `SearchResult` with fields: `title: String`, `path: String`, `size: u64` (or String), and `extension: String`.
*   Show me how to use `egui::ScrollArea` effectively here.
*   Show me how to use `ui.allocate_ui_at_rect` or `ui.with_layout` if necessary to keep the size badge pinned to the right.

**Goal:** The final result should look clean, professional, and handle long filenames gracefully without breaking the layout.
