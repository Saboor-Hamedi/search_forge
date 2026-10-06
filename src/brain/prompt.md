Exactly. You don't want your agent to **parse the PDF into text and fake a document preview**. You want an actual **PDF renderer**, essentially the same fundamental experience as Chrome's built-in PDF viewer—but integrated into SearchForge's preview pane and stripped down to the essentials.

Here is the focused prompt I'd give your agent:

---

# SearchForge — Native-Quality PDF Preview Refactor

## Objective

The current PDF preview is wrong.

When a PDF is selected, SearchForge currently attempts to read the PDF as text and produces ugly output such as:

```text
??
??
??
??
??
```

**Stop doing this completely.**

A PDF is not a text file.

I do **not** want the PDF preview to extract the raw PDF contents and display them as text.

I want SearchForge to display the PDF using a **real PDF rendering engine**, similar to how a PDF opens natively inside **Google Chrome's built-in PDF viewer**.

However, SearchForge should have a much more minimal and elegant presentation.

---

# 1. Real PDF Rendering — Not Text Parsing

The PDF preview pipeline must become:

```text
PDF file
   ↓
PDF rendering engine
   ↓
rasterized PDF page
   ↓
egui texture
   ↓
SearchForge preview
```

NOT:

```text
PDF
 ↓
read_to_string()
 ↓
UTF-8 parsing
 ↓
??
??
??
```

Remove the current PDF-as-text behavior entirely.

There must never again be:

```text
??
??
??
```

or raw PDF syntax shown to the user.

---

# 2. Think "Chrome PDF Viewer"

Use the **same conceptual model as Chrome's PDF viewer**.

When Chrome opens:

```text
example.pdf
```

it doesn't display the PDF's underlying binary/text structure.

It renders the actual document pages.

SearchForge should do the same.

The user should see:

```text
┌───────────────────────────────┐
│                               │
│          PDF PAGE             │
│                               │
│      actual document          │
│                               │
│                               │
└───────────────────────────────┘
```

The document itself should look exactly like a real PDF page.

Fonts, images, layout, tables, spacing, etc. should be rendered by the PDF renderer.

---

# 3. BUT: Do NOT Copy Chrome's UI

This is extremely important.

I do **not** want:

- Chrome's PDF toolbar
- sidebar
- thumbnails panel
- print button
- download button
- browser controls
- unnecessary PDF-reader chrome
- large header
- giant toolbar
- unnecessary controls

SearchForge is not trying to become a PDF reader.

It is a **file search application with a beautiful document preview**.

Therefore:

> Borrow Chrome's PDF rendering quality, NOT Chrome's PDF-reader interface.

---

# 4. Desired SearchForge PDF Experience

The preview should feel almost invisible.

Something like:

```text
┌─────────────────────────────────────────────┐
│                                             │
│       The 99 Names of Allah.pdf             │
│       PDF · 273 pages · 1.4 MB              │
│                                             │
│             ┌─────────────────┐             │
│             │                 │             │
│             │                 │             │
│             │    PDF PAGE     │             │
│             │                 │             │
│             │                 │             │
│             └─────────────────┘             │
│                                             │
│             ┌─────────────────┐             │
│             │                 │             │
│             │    PDF PAGE     │             │
│             │                 │             │
│             └─────────────────┘             │
│                                             │
└─────────────────────────────────────────────┘
```

The actual page rendering should be high quality.

The surrounding application should remain minimal.

---

# 5. PDF Pages Must Look Like Actual Documents

Do not create a fake card containing extracted PDF text.

Render the actual PDF page.

For example, if the PDF contains:

- Arabic
- Persian
- English
- images
- tables
- typography
- colored elements
- page backgrounds
- illustrations

all of that should appear as part of the rendered page.

The PDF renderer is responsible for reproducing the document visually.

---

# 6. Lightweight Rendering

The PDF viewer must remain lightweight.

Do not build a gigantic PDF subsystem.

Do not render the entire PDF into memory.

If a PDF contains:

```text
273 pages
```

do NOT render all 273 pages at startup.

Use lazy rendering:

```text
                 viewport
                    │
        ┌───────────┴───────────┐
        │                       │
    visible pages          nearby pages
        │                       │
        └───────────┬───────────┘
                    ↓
             PDF renderer
                    ↓
             egui textures
```

Only render pages that are visible or very close to becoming visible.

---

# 7. Scrolling

The user should be able to scroll through the PDF naturally.

The experience should feel like:

> opening a PDF in Chrome and simply scrolling through it.

Not:

> clicking "Page 1", "Page 2", "Page 3".

No page navigation UI is necessary unless it is genuinely required.

Natural vertical scrolling is preferred.

---

# 8. No PDF Sidebar

Absolutely no PDF thumbnail sidebar.

Do NOT create:

```text
┌──────┬───────────────────────────┐
│ 1    │                           │
│ 2    │       PDF                 │
│ 3    │                           │
│ 4    │                           │
│ 5    │                           │
└──────┴───────────────────────────┘
```

The SearchForge preview should be:

```text
┌──────────────────────────────────┐
│                                  │
│           PDF PAGE               │
│                                  │
│           PDF PAGE               │
│                                  │
│           PDF PAGE               │
│                                  │
└──────────────────────────────────┘
```

Clean.

---

# 9. Minimal Metadata

The PDF preview may have a very small metadata area.

For example:

```text
The 99 Names of Allah
PDF · 273 pages · 1.4 MB
```

That's enough.

Don't build a giant PDF header.

Don't build a toolbar.

Don't duplicate metadata everywhere.

---

# 10. Open Externally

There can be one subtle action:

```text
Open ↗
```

This should open the PDF using the operating system's default application.

It should not dominate the UI.

---

# 11. UI Should Feel Like SearchForge

The PDF should feel integrated into SearchForge.

It should NOT feel like:

```text
Chrome embedded inside SearchForge
```

and it should NOT feel like:

```text
Adobe Reader embedded inside SearchForge
```

Instead:

```text
SearchForge
    ↓
beautiful minimal preview
    ↓
real PDF rendering
```

---

# 12. PDF Renderer Selection

Inspect the existing project and choose a **mature PDF rendering engine** capable of rendering real PDF pages.

The renderer must support:

- fonts
- images
- vector graphics
- page layouts
- Unicode
- multilingual documents
- normal PDF features

Prioritize:

1. Rendering correctness
2. Stability
3. Windows compatibility
4. Performance
5. Memory usage
6. Dependency size

Do not use a simplistic PDF parser that only extracts text.

Do not implement a PDF renderer yourself.

---

# 13. Never Block egui

Rendering must happen outside the UI thread whenever it is expensive.

Architecture should resemble:

```text
egui
 │
 │ request page 20
 ↓
background PDF worker
 │
 │ render
 ↓
rendered page
 │
 ↓
egui texture
 │
 ↓
display
```

The application must remain responsive while rendering.

No:

```text
Not Responding
```

No freezing while opening a large PDF.

---

# 14. Texture Caching

Cache rendered pages.

For example:

```text
Page 20 → texture
Page 21 → texture
Page 22 → texture
```

When the user scrolls away from them, allow old textures to be evicted according to a reasonable memory limit.

Do not permanently keep every page.

---

# 15. High-Quality Scaling

Pages should scale according to the available preview width.

For example:

```text
small window
     ↓
smaller page

large window
     ↓
larger page
```

Maintain the PDF's original aspect ratio.

Never stretch pages.

Never distort them.

Use a clean centered layout.

---

# 16. Background Around the Page

The application can use a subtle dark background around the document.

The actual PDF page can remain visually similar to paper.

Conceptually:

```text
dark application background

          ┌───────────────┐
          │               │
          │   PDF PAGE    │
          │               │
          └───────────────┘

dark application background
```

No giant card surrounding the page.

No heavy border.

No excessive shadow.

---

# 17. Remove the Current Broken Implementation

Find the code responsible for producing:

```text
??
??
??
```

and remove the incorrect PDF text-preview path.

Do not simply hide the question marks.

Fix the architecture.

The correct behavior is:

```text
is_pdf(path)
    ↓
PdfPreview
    ↓
real renderer
```

not:

```text
is_pdf(path)
    ↓
generic TextPreview
```

---

# 18. Keep Other File Previews Separate

Use a clean preview dispatch:

```rust
match file_type {
    FileType::Pdf => PdfPreview,
    FileType::Image => ImagePreview,
    FileType::Text => TextPreview,
    FileType::Binary => BinaryPreview,
}
```

PDFs must never fall through to the generic text renderer.

---

# 19. Performance Target

Opening a PDF should feel immediate.

Initial sequence:

```text
select PDF
     ↓
metadata immediately
     ↓
first visible page starts rendering
     ↓
page appears
     ↓
user can immediately scroll
     ↓
additional pages render lazily
```

Do not wait for the entire PDF before showing anything.

---

# 20. Final Visual Target

The final result should communicate:

> **"This is a PDF."**

not:

> **"This is a PDF parser."**

and not:

> **"This is a miniature Chrome window."**

It should feel like a **native, minimal, premium SearchForge document preview**.

The formula is:

```text
Chrome-quality PDF rendering
+
SearchForge minimalism
-
Chrome PDF toolbar
-
sidebar
-
heavy borders
-
unnecessary controls
-
PDF text extraction
```

---

## Acceptance Criteria

The implementation is complete only when:

- [ ] PDFs display as **actual rendered pages**
- [ ] No `??` appears anywhere
- [ ] No raw PDF/binary data is displayed
- [ ] PDF pages look like the original document
- [ ] Arabic/Persian/Unicode PDFs render correctly
- [ ] Scrolling feels natural
- [ ] No thumbnail sidebar
- [ ] No unnecessary PDF toolbar
- [ ] No giant PDF header
- [ ] Only minimal metadata is shown
- [ ] Pages are lazy-rendered
- [ ] Pages are cached intelligently
- [ ] PDF rendering does not freeze egui
- [ ] Large PDFs remain usable
- [ ] External "Open" action still works
- [ ] UI remains sleek and lightweight
- [ ] Existing text/image previews continue working
- [ ] No egui ID collision warnings
- [ ] `cargo check` passes
- [ ] `cargo clippy` passes
- [ ] tests pass

**Do not declare success merely because the PDF opens.**

The PDF must **look like a real PDF**, render correctly, scroll naturally, and feel like a polished native component of SearchForge.
