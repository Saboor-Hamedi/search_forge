# SearchForge — Future Work & Strategic Roadmap

This document outlines **20 robust, high-impact suggestions** for the future engineering and product roadmap of **SearchForge**. These suggestions are designed to elevate SearchForge from a desktop search utility into an ultra-fast, world-class productivity command center on Windows and cross-platform desktop environments.

---

## I. Search Engine & Low-Level Indexing Performance

### 1. Instant Windows NTFS USN Journal & Master File Table (MFT) Scanner
- **Description**: Replace or supplement the recursive directory walker (`walkdir`) on Windows with direct NTFS Master File Table (MFT) / Update Sequence Number (USN) journal reading.
- **Technical Architecture**:
  - Implement a privileged reader using `DeviceIoControl` with `FSCTL_ENUM_USN_DATA` and `FSCTL_READ_USN_JOURNAL` for NTFS volumes.
  - Drop full-drive initial indexing time from minutes to under 2 seconds across millions of files (matching the speed of Voidtools Everything).
  - Gracefully fallback to the user-space directory walker for network shares, exFAT/FAT32 drives, and non-Windows targets.
- **Impact**: Zero perceptible scan latency on fresh boots or large multi-terabyte drives.

### 2. SQLite FTS5 Full-Text Content Search for Documents & Code
- **Description**: Enable users to search *inside* file contents (plain text, markdown, source code, docx, and PDFs) using SQLite’s native `FTS5` (Full-Text Search) virtual table extension with BM25 ranking.
- **Technical Architecture**:
  - Add an asynchronous background indexing worker that extracts text in chunks and inserts tokens into an FTS5 table with Porter stemmer and unicode61 tokenizer.
  - Introduce prefix syntax queries like `content:"search query"` or `in:file fn main`.
  - Store token offsets to render matching line snippets with highlighted matches in search results and preview panels.
- **Impact**: Transforms SearchForge from a filename lookup tool into an indispensable code and document research tool.

### 3. Asymmetric Trigram & Modified Smith-Waterman Fuzzy Scoring
- **Description**: Upgrade the substring/prefix search matcher to a tiered scoring engine that prioritizes exact acronyms, path initials, CamelCase word boundaries, and typographical error tolerance.
- **Technical Architecture**:
  - Implement an in-memory modified Smith-Waterman or Fzy/Fzf algorithm in Rust with SIMD vectorization (`wide` or `std::simd`).
  - Score boosts based on: exact filename match > basename prefix > CamelCase acronym (`VSCode` matches `Visual Studio Code`) > path segment match > loose fuzzy.
  - Cache normalized search tokens in an immutable snapshot swapped via `arc-swap`.
- **Impact**: Keyboard-driven ergonomics where typing 2–3 letters immediately yields the desired application or file.

### 4. Dynamic Multi-Root Workspace & External Drive Mount Detection
- **Description**: Automatically detect when external drives, USB keys, secondary SSDs, or WSL2 virtual hard disks (`\\wsl$\`) are attached or detached.
- **Technical Architecture**:
  - Listen for Win32 `WM_DEVICECHANGE` window messages in the background tray message loop.
  - Maintain independent index partitions per drive root (e.g. `C:`, `D:`, `E:`).
  - Mark index shards offline when a drive is ejected without corrupting the main SQLite cache.
- **Impact**: Flawless multi-drive workflows without requiring manual index resets or crashing on missing paths.

---

## II. Spotlight Command Center & Instant Actions

### 5. Inline Natural Language Calculator & Unit / Currency Converter
- **Description**: Execute mathematical expressions, currency conversions, timezone calculations, and unit transforms directly in the Spotlight search input.
- **Technical Architecture**:
  - Integrate a lightweight, zero-dependency parser using a Rust Pratt parser (e.g., `evalexpr` or a custom AST evaluator).
  - Support inputs like `= (45 * 12) / 8`, `120 USD to EUR`, `16GB in MB`, `timestamp 1700000000`, `now in Tokyo`.
  - Display the computed result as the #1 Spotlight candidate with an instant "Copy Result" badge upon pressing `Enter`.
- **Impact**: Eliminates the constant need to open a browser tab or separate calculator app for quick day-to-day calculations.

### 6. System Power & Developer Quick Actions
- **Description**: Add built-in command runners for common OS administrative tasks and developer commands.
- **Technical Architecture**:
  - Register reserved commands such as `> lock`, `> sleep`, `> restart`, `> shutdown`, `> empty recycle bin`, `> ip`, `> flushdns`.
  - For developers: `> kill <process>`, `> port <number>` (finds and terminates process listening on port), `> env <var>`.
  - Execute via Windows API (`ExitWindowsEx`, `SHEmptyRecycleBinW`) or asynchronous elevated PowerShell subprocess.
- **Impact**: Powers users to manage their machine rapidly without reaching for Task Manager or Start Menu.

### 7. Universal Clipboard History Manager with Image & Text Pinning
- **Description**: Add a clipboard history viewer accessible via shortcut (e.g., `Alt + V` or `clip:`) integrated seamlessly into the Spotlight UI.
- **Technical Architecture**:
  - Implement a Windows clipboard listener using `AddClipboardFormatListener`.
  - Store the last 200 clipboard entries (encrypted or local SQLite) including text, formatted code snippets, hex colors, and PNG thumbnails.
  - Enable fuzzy search through clipboard history with instantaneous paste back into the previous active window via `SendInput`.
- **Impact**: Replaces third-party clipboard utilities with an integrated, secure, native Rust solution.

### 8. Direct Shell & Terminal Command Runner (`>` Prefix Mode)
- **Description**: Allow prefixing any query with `>` to run shell commands in PowerShell, Windows Terminal, CMD, or WSL.
- **Technical Architecture**:
  - Detect `>` or `$ ` prefix in `spotlight.rs`.
  - Display options to "Run in Background", "Run in Windows Terminal", or "Run as Administrator".
  - Show live command preview and pipe output back into the Spotlight result area or launch the user's preferred terminal emulator.
- **Impact**: Turns SearchForge into a universal runner for developers and power users.

---

## III. File Previews & Media Capabilities

### 9. High-Performance Syntax Highlighting with `syntect` & Custom Themes
- **Description**: Upgrade the preview panel's plain-text source code viewer to full Sublime-compatible syntax highlighting.
- **Technical Architecture**:
  - Integrate `syntect` with bundled two-pass syntaxes for Rust, Python, JavaScript, TypeScript, C/C++, Go, HTML, CSS, JSON, TOML, YAML, SQL, Shell.
  - Use `syntect::highlighting::ThemeSet` mapped directly to SearchForge’s dark and glass UI palette.
  - Render colored text spans in `egui` using `JobBuilder` or layout galleys for smooth scrolling.
- **Impact**: Professional-grade previewing of code files without having to launch an IDE.

### 10. Multi-Page Native PDF Hardware-Accelerated Rendering & Navigation
- **Description**: Enhance the PDF previewer with smooth page flipping, zoom controls, and text selection.
- **Technical Architecture**:
  - Expand the existing `pdfium-render` implementation with an asynchronous multi-page LRU rasterization cache.
  - Support mouse-wheel zoom (`Ctrl + Wheel`), page jump slider, and text extraction overlay.
  - Add fallback vector rendering via `lopdf` when native dynamic libraries are not present.
- **Impact**: Turns SearchForge into an instant document reader for research papers, invoices, and manuals.

### 11. Archive Inspector & Quick Extraction (ZIP, TAR, GZ, 7Z, RAR)
- **Description**: Preview compressed archive contents directly within the preview panel without extracting them to disk.
- **Technical Architecture**:
  - Use `zip-rs` and `sevenz-rust` to parse central directory headers asynchronously.
  - Render an interactive tree view of files and folder structures inside the archive.
  - Provide a one-click "Extract Here" and "Extract to Subfolder" button.
- **Impact**: Drastically reduces clutter and time spent opening archive utility applications.

### 12. Audio, Video, and Image EXIF Metadata Inspector
- **Description**: Display detailed multimedia metadata for audio (`.mp3`, `.wav`, `.flac`), video (`.mp4`, `.mkv`), and photos (`.jpg`, `.png`, `.raw`).
- **Technical Architecture**:
  - Integrate lightweight metadata extractors (such as `id3`, `symphonia-metadata`, `kamadak-exif`).
  - Render album art, resolution, codec, duration, bitrate, camera model, focal length, ISO, and GPS location link.
  - Provide quick audio preview playback using `rodio` or native media playback engine.
- **Impact**: Photographers, musicians, and creators can inspect assets instantly from search.

---

## IV. User Experience, Ergonomics & Customization

### 13. Advanced Keyboard Navigation & Vim Keybindings (`jk`, `/`, `Ctrl+N/P`)
- **Description**: Introduce complete keyboard navigability with optional Vim-style modal navigation for power users.
- **Technical Architecture**:
  - Support `Ctrl + N` / `Ctrl + P` (or `j` / `k` when not focusing an edit text field) for list traversal.
  - Add `Ctrl + O` to open, `Ctrl + Shift + C` to copy path, `Ctrl + Enter` to reveal in File Explorer.
  - Add customizable keybinding mappings in `config.toml`.
- **Impact**: Flawless keyboard accessibility without lifting hands from the home row.

### 14. Contextual File Action Menu (Right-Click & `Alt + Enter`)
- **Description**: Provide a comprehensive context menu of file management actions directly in both Spotlight and Full window modes.
- **Technical Architecture**:
  - Support: "Open with...", "Copy Path", "Copy File", "Show in Explorer", "Open in Terminal", "Open in VS Code", "Delete to Trash", "Properties".
  - Integrate with Windows Recycle Bin via Win32 `SHFileOperationW` / `IFileOperation` to ensure accidental deletions are safe and reversible.
  - Integrate with native Windows Shell context menu for shell extensions (`IContextMenu`).
- **Impact**: Replaces routine trips to Windows File Explorer for daily file operations.

### 15. Intelligent Frequency-Based Ranking (Frecent Learning Algorithm)
- **Description**: Learn user habits by tracking launch frequency and recency ("Frecency"), elevating often-used apps and files to the top.
- **Technical Architecture**:
  - Implement the Mozilla Frecency scoring algorithm:
    $$\text{Score} = \text{Frequency} \times \text{Recency Weight}$$
  - Persist launch events in SQLite with zero user profiling or telemetry.
  - Include an option in Preferences to clear history or disable history-based boosting.
- **Impact**: SearchForge dynamically adapts to each user's unique daily workflow.

### 16. Custom Accent Colors, OLED Pure Black, and Native Windows 11 Mica Backdrop
- **Description**: Expand customization with custom UI accent colors, pure black OLED mode, and Windows 11 DWM Mica / Acrylic backdrops.
- **Technical Architecture**:
  - Expand `ThemeMode` in `config.rs` with `OledBlack`, `Mica`, `Acrylic`, and customizable RGB primary accents.
  - Use `DwmSetWindowAttribute` with `DWMWA_SYSTEMBACKDROP_TYPE` (`DWMSBT_MAINWINDOW` or `DWMSBT_TRANSIENTWINDOW`) on Windows 11 22H2+.
  - Expose a color picker in Preferences modal to change the signature yellow accent to blue, purple, emerald, or orange.
- **Impact**: Premium aesthetic cohesion matching modern Windows 11 visual design.

---

## V. Extensibility & Ecosystem Integration

### 17. Safe WASM / Lua Plugin Architecture for Community Extensions
- **Description**: Allow the developer community to create SearchForge extensions (e.g., GitHub issue search, Spotify controller, Home Assistant, Notion search, Dictionary).
- **Technical Architecture**:
  - Embed `wasmtime` or `mlua` in a sandboxed, low-overhead runtime.
  - Provide a clean JSON-RPC or Rust FFI API: `on_query(query: String) -> Vec<SearchResultItem>` and `on_execute(id: String)`.
  - Include a local plugin directory (`~/.searchforge/plugins/`) with hot-reloading.
- **Impact**: Unlocks an infinite ecosystem of integrations mirroring Raycast, Alfred, and PowerToys Run.

### 18. Local Semantic AI Search via On-Device Embeddings
- **Description**: Find documents by conceptual meaning rather than exact keywords (e.g., searching "expenses last month" matches `receipt_september_2026.pdf`).
- **Technical Architecture**:
  - Integrate a quantized, lightweight on-device embedding model (e.g., MiniLM-L6-v2 via `candle` or `ort` ONNX runtime).
  - Store 384-dimensional vector embeddings in SQLite with `sqlite-vec` or an in-memory HNSW index (`instant-distance`).
  - Execute fully offline without sending a single byte of user data to the cloud.
- **Impact**: Next-generation AI-assisted desktop search that keeps user privacy 100% local.

### 19. Git Repository Status & Quick Branch / Workspace Switcher
- **Description**: Automatically detect Git repositories in indexed paths and show repository status (current branch, dirty state, uncommitted changes).
- **Technical Architecture**:
  - When a selected folder contains a `.git` directory, query its HEAD and status via `git2-rs` or direct loose reference parsing.
  - Show a Git branch pill in the results list and preview panel with commits ahead/behind.
  - Provide one-click actions: "Open in VS Code", "Open in GitKraken/SourceTree", "Copy Git Remote URL".
- **Impact**: Makes SearchForge an indispensable workspace launcher for software engineers.

### 20. Multi-Platform Support: Linux (Wayland/X11) & macOS
- **Description**: Generalize the platform-specific layers so SearchForge runs with native excellence on macOS and Linux desktop environments.
- **Technical Architecture**:
  - Abstract native window effects (`window_effects.rs`), hotkeys, and tray management behind a clean platform abstraction trait (`PlatformBackend`).
  - Linux: Implement global hotkeys via X11 `XGrabKey` or Wayland GlobalShortcuts portal, with GTK/AppIndicator tray.
  - macOS: Implement NSWindow styling with visual effect view (VibrantDark), global event tap, and NSStatusItem tray.
- **Impact**: Expands SearchForge's audience to all major operating systems while keeping 100% shared Rust business logic and UI code.

---

## Roadmap Prioritization Matrix

| Phase | Milestone | Included Suggestions | Target Focus |
|---|---|---|---|
| **Phase 1** | *Immediate Polish & Productivity* | #5 (Calculator/Converter), #6 (System Actions), #13 (Vim/Keyboard shortcuts), #14 (Context Menu), #15 (Frecent Ranking) | Pure UX improvements, instant utility |
| **Phase 2** | *Deep Media & Content Power* | #2 (SQLite FTS5 Content Search), #9 (Syntax Highlighting), #10 (PDF Page Flipping), #11 (Archive Inspector), #12 (EXIF/Audio metadata) | Deep file preview and inspection |
| **Phase 3** | *Performance & Native Windows Excellence* | #1 (NTFS USN/MFT Scanner), #4 (Multi-Drive & WSL Mounts), #7 (Clipboard History), #16 (Mica/OLED Themes) | System-level speed and OS integration |
| **Phase 4** | *Platform Expansion & Extensibility* | #17 (WASM Plugins), #18 (Local AI Semantic Search), #19 (Git Workspace Switcher), #20 (Linux & macOS) | Ecosystem & cross-platform growth |
