# SearchForge — Premium UI Refinement & Application Search Polish

## Objective

SearchForge is now functionally strong. The next task is **not a feature rewrite**.

This is a focused **premium UI/UX refinement pass**, especially for:

- application search
- application icons
- search result presentation
- preview hierarchy
- footer/status information
- Filter dialog
- shadows/depth
- spacing and visual polish

Use the attached screenshots as the visual reference.

The goal is **not to copy Microsoft's UI**, but to learn from its strengths:

> clean hierarchy, excellent spacing, restrained controls, strong application identity, and almost no unnecessary visual noise.

SearchForge should feel like a **serious native Windows utility**, not a generic egui application.

---

# 1. Important: Understand the `230,024 items indexed`

The current footer displays something like:

```text
SearchForge v0.1.6 · 230,024 items indexed
```

First, inspect the indexing implementation and determine **exactly what this number represents**.

It must be technically accurate.

Determine whether it represents:

- database rows
- files
- folders
- applications
- shortcuts
- indexed filesystem entries
- failed/partial entries
- something else

Do not guess.

If `230,024` means indexed filesystem entries, make that clear.

For example:

```text
230,024 items indexed
```

is acceptable if that is genuinely what it represents.

But if the number is ambiguous or misleading, improve the wording.

Also verify whether:

- excluded paths are counted
- inaccessible files are counted
- duplicate entries are counted
- application shortcuts are counted
- folders are counted

The status must accurately represent the underlying index.

---

# 2. Footer — Reduce Its Visual Importance

The current footer gives too much visual emphasis to:

```text
SearchForge v0.1.6 · 230,024 items indexed
```

The reference UI demonstrates a better principle:

> The search experience should occupy almost all of the user's visual attention.

The footer should be extremely subtle.

Consider:

```text
SearchForge v0.1.6 · 230,024 items indexed
```

in small, low-contrast typography.

Do not put it inside a large visually separated bar.

Do not make it compete with search results.

If appropriate, move status information into a subtle bottom status line.

---

# 3. Application Search Must Become Much Better

The current SearchForge application result:

```text
Windows Terminal
Microsoft Corporation · Application
```

is already heading in the right direction.

But the visual quality should be significantly improved.

Compare the attached Microsoft reference.

When searching:

```text
terminal
```

the application result should immediately communicate:

```text
Windows Terminal
Microsoft Corporation
Application
```

with the **actual Windows Terminal icon**.

---

# 4. Fix Application Icons

This is a major issue.

The application result must display the **real application icon** whenever possible.

For:

```text
Windows Terminal
Visual Studio Code
Sublime Text
NetBeans
DBeaver
Firefox
Chrome
etc.
```

SearchForge should extract and cache the application's actual icon.

Do not display:

```text
APP
```

or a generic colored square when the real icon is available.

---

# 5. Windows Packaged Applications

Pay special attention to applications installed through:

- Microsoft Store
- WindowsApps
- packaged applications
- AppX/MSIX

The screenshot shows:

```text
C:\Users\Saboor\AppData\Local\Microsoft\WindowsApps\wt.exe
```

but the actual application identity is:

```text
Windows Terminal
```

Do not treat:

```text
wt.exe
```

as the application's display identity.

Resolve the proper application metadata and icon.

The application model should distinguish:

```text
Executable:
wt.exe

Display name:
Windows Terminal

Publisher:
Microsoft Corporation

Type:
Packaged application
```

---

# 6. Application Result Design

Application results should feel more like a launcher result than a file result.

Desired hierarchy:

```text
[REAL ICON]   Windows Terminal
              Microsoft Corporation · Application
```

Do not prioritize:

```text
wt.exe
C:\WindowsApps\...
```

in the primary result.

The filesystem path is secondary metadata.

---

# 7. Application Preview

The preview should also become cleaner.

Current preview has too much empty card-like structure.

Create a simple hierarchy:

```text
              [Application Icon]

              Windows Terminal

       Microsoft Corporation
       Installed application

Location
C:\Users\Saboor\AppData\Local\Microsoft\...

                         Launch ↗
```

The icon should be prominent but not huge.

The application name should be the strongest text.

Publisher/type should be secondary.

Location should be tertiary.

---

# 8. Remove Excessive Empty Space

The current application preview has too much dead space and feels somewhat unfinished.

Use the available area intentionally.

Do not fill it with decorative cards.

Use:

- spacing
- alignment
- typography
- subtle grouping

instead.

The preview should feel balanced.

---

# 9. Application Actions

The application preview should have one clear primary action:

```text
Launch ↗
```

Do not duplicate Launch controls.

The result row itself may optionally expose subtle quick actions if the architecture already supports them, but do not clutter every row.

The primary workflow should remain:

```text
Search
 ↓
Select application
 ↓
Enter / Launch
```

---

# 10. Search Result Quick Actions

The Microsoft reference has useful contextual actions.

SearchForge can adopt the **concept**, but keep it minimal.

For an application result, potentially support subtle actions such as:

```text
Launch
Open location
```

Only show these when appropriate.

Do not permanently display a row full of buttons.

Use hover/reveal behavior if it can be implemented without making the UI heavy.

---

# 11. Search Bar

The search bar currently feels too heavy during interaction.

It should become much more refined.

### Normal

Very subtle background.

### Hover

Almost imperceptible change.

### Focused

Clear but restrained yellow accent.

Do not create a thick yellow border.

Do not create a glowing outline.

The search bar should feel like a native command/search field.

---

# 12. Search Result Hover

Current file/application hover states feel too heavy.

Reduce the visual delta.

Use:

```text
Normal
    ↓
very subtle background

Hover
    ↓
slightly lighter background

Selected
    ↓
slightly stronger background
+
small yellow indicator
```

Do not use:

- thick borders
- glow
- large shadows
- dramatic color changes
- scaling animations

The user should barely notice the hover transition.

---

# 13. Selected Result

The existing yellow indicator is useful.

Keep it.

But use it as a **precision accent**.

The selected result should not look like a bright highlighted card.

It should feel similar to:

```text
slightly elevated background
│
yellow selection indicator
│
application/file
```

Very subtle.

---

# 14. Filter Dialog — Major Refinement

The Filter dialog needs a substantial cleanup.

Current structure contains unnecessary explanatory text:

```text
Paths matching any of these patterns will be skipped:
```

Remove that text.

The UI should be self-explanatory.

---

# 15. Move Add Input to the Top

The custom exclusion input currently appears too low.

Move it to the **top of the exclusion list**.

Desired structure:

```text
Exclusion Filters                           ×

[ Enter pattern...                  ] [Add]

☐ node_modules
☐ .git
☐ target
☐ AppData
☐ .cache
☐ $RECYCLE.BIN

Reset Defaults
```

This is much more logical.

The user can immediately add a pattern before reviewing existing exclusions.

---

# 16. Remove the `Done` Button

Remove:

```text
Done
```

The dialog should not require a Done button if changes are applied immediately.

The close `×` should close the dialog.

If the application currently requires explicit confirmation internally, refactor the state handling so the controls apply changes immediately and safely.

Do not keep a redundant Done button.

---

# 17. Filter Input and Add Button

This is important.

The input and Add button must have **exactly the same height**.

For example:

```text
┌────────────────────────────────┐ ┌──────┐
│ Enter exclusion pattern...     │ │ Add  │
└────────────────────────────────┘ └──────┘
```

Both controls:

- same height
- same vertical alignment
- same baseline
- consistent corner radius
- consistent internal padding

No mismatch.

The Add button must not look taller or shorter than the input.

---

# 18. Filter Input Visual Quality

The current input looks ugly.

Redesign it as a polished native input.

Use:

- subtle dark surface
- comfortable horizontal padding
- restrained border
- clear focus state
- proper placeholder typography

Avoid:

- tiny cramped input
- bright outline
- excessive border
- strange internal padding

The input should look like it belongs to the same product as the search bar.

---

# 19. Filter List

The exclusion rows should be clean:

```text
☐ node_modules
☐ .git
☐ target
☐ AppData
☐ .cache
☐ $RECYCLE.BIN
```

No individual cards.

No excessive borders.

No broken icons.

No unnecessary containers.

Use subtle row hover.

---

# 20. Reset Defaults

Keep:

```text
Reset Defaults
```

Make it a secondary action.

It should not visually compete with the exclusion input or list.

Do not remove it.

---

# 21. Filter Dialog Shadow / Depth

The current application feels too flat.

The dialog needs **a subtle sense of elevation**.

Add a restrained shadow/depth effect so the dialog visually separates from the main SearchForge window.

But do NOT create:

- huge shadows
- glowing borders
- neon effects
- excessive blur

Think:

> subtle native popup elevation.

The dialog should feel like it is floating slightly above the main interface.

---

# 22. Main Window Depth

The main SearchForge window can also use extremely subtle depth where appropriate.

The goal is not skeuomorphism.

Use just enough contrast/shadow to establish:

```text
background
    ↓
SearchForge window
    ↓
dialog/popover
```

The visual hierarchy should become clearer.

---

# 23. Color System

Keep the current dark aesthetic.

Keep yellow as the primary accent.

But use it selectively for:

- selected result indicator
- focused search
- primary action
- important active state

Do not turn every interactive element yellow.

The Microsoft reference demonstrates good restraint here.

---

# 24. Typography

Improve hierarchy.

Application:

```text
Windows Terminal
```

should be clearly stronger than:

```text
Microsoft Corporation · Application
```

and:

```text
C:\Users\...
```

Similarly:

```text
SearchForge
```

and important controls should have clear hierarchy without oversized text.

---

# 25. Result Metadata

For applications, prefer:

```text
Microsoft Corporation · Application
```

rather than exposing too much filesystem information in the result row.

For normal files:

```text
filename
path
size
```

remains appropriate.

Different entity types should have different information hierarchy.

---

# 26. Search Suggestions / Empty State

Keep the previous empty-state improvements.

When no search is active:

```text
No files found
```

or:

```text
Select a file to preview
```

should be centered within their **individual panes**.

Do not put them near the top.

Do not create giant cards.

---

# 27. Preview Pane

The preview pane should feel less like a card and more like a clean information canvas.

For an application:

```text
ICON

Application Name
Publisher · Application

Location
...

Launch ↗
```

For PDF:

actual PDF pages.

For Markdown:

full readable document.

For images:

actual image.

The preview architecture should remain type-specific.

---

# 28. Do Not Make Everything a Card

This is a key design rule.

Avoid:

```text
card
  └── card
      └── card
```

Instead use:

```text
spacing
typography
alignment
subtle surfaces
```

Cards should only exist when they actually communicate grouping.

---

# 29. Important: Do Not Copy Microsoft's UI Literally

Use the reference screenshots as a **UX quality benchmark**, not as a design to clone.

SearchForge should have its own identity.

Borrow:

- clean spacing
- strong hierarchy
- restrained hover
- excellent app icons
- minimal chrome
- clear actions
- subtle elevation

Do not copy exact colors, layouts, icons, or branding.

---

# 30. Performance

Do not sacrifice SearchForge's speed for visual polish.

Especially:

- application icons must be cached
- application metadata must be cached
- don't extract icons every search
- don't repeatedly calculate application metadata
- don't introduce expensive animations
- don't create continuous repaint loops

The UI should remain extremely responsive.

---

# 31. Final Application Search Experience

Searching:

```text
terminal
```

should feel approximately like:

```text
┌───────────────────────────────────────────────┐
│ 🔍 terminal                              Filter│
├───────────────────┬───────────────────────────┤
│                   │                           │
│ [Terminal icon]   │       [Terminal icon]    │
│ Windows Terminal  │                           │
│ Microsoft Corp.   │       Windows Terminal   │
│                   │       Microsoft Corp.     │
│ terminal.py       │                           │
│ terminal_theme.py │       Location            │
│                   │       C:\...\wt.exe       │
│                   │                           │
│                   │                   Launch ↗ │
└───────────────────┴───────────────────────────┘
```

But visually much more refined than this ASCII representation.

---

# 32. Acceptance Criteria

### Application Search

- [ ] Windows Terminal has the correct icon
- [ ] VS Code has the correct icon
- [ ] Sublime Text has the correct icon
- [ ] NetBeans has the correct icon
- [ ] DBeaver has the correct icon
- [ ] Packaged Windows applications are detected correctly
- [ ] Applications are deduplicated
- [ ] Application names are prioritized over executable names
- [ ] Search ranking feels intelligent
- [ ] Launch works
- [ ] Enter launches selected application

### Main UI

- [ ] Search field feels lightweight
- [ ] Hover is subtle
- [ ] Selected state is clear but restrained
- [ ] Footer/status is visually secondary
- [ ] `230,024 items indexed` is technically accurate
- [ ] Application results have strong visual identity
- [ ] Preview hierarchy is clean
- [ ] No unnecessary cards
- [ ] Subtle depth/elevation exists

### Filter

- [ ] Remove explanatory paragraph
- [ ] Move custom input to top
- [ ] Input and Add button have exactly equal height
- [ ] Remove Done button
- [ ] Keep Reset Defaults
- [ ] Checkbox icons render correctly
- [ ] Filter rows are clean
- [ ] Input looks polished
- [ ] Dialog has subtle shadow/elevation
- [ ] Dialog remains compact

### Preview

- [ ] Application preview is clean
- [ ] Real icon is displayed
- [ ] No giant colored hero container
- [ ] One clear Launch action
- [ ] PDF remains real rendered PDF
- [ ] Markdown remains fully scrollable
- [ ] No content clipping

---

# Final Instruction

Do **not** rewrite SearchForge.

Do **not** add random features.

Do **not** make the interface more complicated.

Perform a disciplined **premium UI refinement pass**.

The guiding principle is:

> **Make every element quieter, clearer, better aligned, and more intentional.**

The biggest priorities are:

**1. Excellent application search with real icons.**
**2. Fix the visual hierarchy of the main search interface.**
**3. Make the Filter dialog genuinely polished.**
**4. Add subtle depth/shadow without making the UI heavy.**
**5. Make the indexed-item status accurate and visually secondary.**

After implementation, run SearchForge and manually test:

```text
terminal
vscode
sublime
netbeans
dbeaver
```

Then test:

- application selection
- Launch
- Enter-to-launch
- hover
- search focus
- Filter dialog
- adding exclusions
- removing exclusions
- Reset Defaults
- window resizing

**Do not declare the task complete just because it compiles. The final result should look and feel like a polished native Windows search utility.**
