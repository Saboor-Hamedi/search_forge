# SearchForge — Fix the Exclusion Filters Dialog Icons and UI

The **Exclusion Filters** dialog is still visually broken and was not properly fixed.

Focus **only on this dialog**. Do not modify the rest of the application.

## Problem

The exclusion filter rows currently contain broken/misplaced icons or checkbox-looking elements.

For example:

```text
node_modules   [broken icon]
.git           [broken icon]
target         [broken icon]
AppData        [broken icon]
.cache         [broken icon]
$RECYCLE.BIN   [broken icon]
```

These controls do not look like proper checkboxes and appear visually broken.

## Required Fix

Redesign the filter rows so every exclusion pattern has a **proper, consistent checkbox/control**.

Desired structure:

```text
☐ node_modules
☐ .git
☐ target
☐ AppData
☐ .cache
☐ $RECYCLE.BIN
```

When selected:

```text
☑ node_modules
```

The checkbox must:

- render correctly
- have a consistent size
- align vertically with the text
- have adequate spacing from the pattern
- have a clear checked/unchecked state
- respond correctly to mouse clicks
- have a subtle hover state
- use the existing SearchForge visual language
- NOT look like a broken icon

## Important

Do **not** use arbitrary Unicode characters such as:

```text
□
☐
☑
✓
```

if the current font/rendering causes them to appear incorrectly.

Use a proper egui checkbox widget or a correctly rendered custom checkbox using egui primitives.

Do not rely on an icon font that may not exist.

---

## Row Layout

Each row should have a clean structure:

```text
┌─────────────────────────────┐
│ ☐  node_modules              │
│ ☐  .git                      │
│ ☐  target                    │
│ ☐  AppData                   │
│ ☐  .cache                    │
│ ☐  $RECYCLE.BIN              │
└─────────────────────────────┘
```

But **do not add heavy borders around every row**.

The dialog should remain lightweight.

Use spacing and alignment instead of individual cards.

---

## Interaction

Unchecked:

```text
☐ node_modules
```

Checked:

```text
☑ node_modules
```

Hover should only produce a **very subtle background change**.

Do not create:

- glowing controls
- bright borders
- large hover effects
- oversized checkboxes
- colorful boxes

The yellow accent can be used for the checked/active state, but very subtly.

---

## Dialog Polish

Keep the existing:

```text
Exclusion Filters                         ×
```

header.

Improve:

- icon alignment
- checkbox alignment
- row spacing
- text alignment
- input alignment
- Add button alignment
- Reset Defaults placement
- Done button placement

The dialog should feel like part of SearchForge rather than a separate UI component.

Desired hierarchy:

```text
Exclusion Filters                         ×

Paths matching any of these patterns
will be skipped:

☐ node_modules
☐ .git
☐ target
☐ AppData
☐ .cache
☐ $RECYCLE.BIN

[ Enter custom pattern... ] [ Add ]

Reset Defaults                         Done
```

---

## Critical Implementation Requirement

Inspect the **actual existing implementation** of the Exclusion Filters dialog.

Find why the icons/checkboxes are rendering incorrectly.

Do not simply change their color or size.

Determine whether the problem comes from:

- incorrect icon rendering
- unsupported glyph
- wrong font
- incorrect egui widget
- bad layout
- incorrect image/icon asset
- invalid texture
- incorrect checkbox implementation

Then fix the underlying cause.

**Do not replace the broken icon with another potentially unsupported Unicode character.**

Prefer native egui controls/primitives.

---

## Final Verification

After fixing it, launch SearchForge and manually verify:

- [ ] Every exclusion row has a correctly rendered checkbox
- [ ] Checked state is visually obvious
- [ ] Unchecked state is visually obvious
- [ ] Clicking works
- [ ] Hover is subtle
- [ ] All rows align perfectly
- [ ] No broken icons remain
- [ ] No strange glyphs remain
- [ ] No layout overlap
- [ ] Add input still works
- [ ] Reset Defaults still works
- [ ] Done still works
- [ ] Dialog remains compact and lightweight

**Do not change anything outside the Exclusion Filters dialog.**
