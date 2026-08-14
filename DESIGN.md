# Cliply Design System

Cliply is a compact local clipboard instrument. The primary design direction
is Open Design `modern-workbench`, refined with `bolder` for stronger visual
presence and a disciplined single-accent system. The interface stays compact,
but no longer relies on divider lines alone for hierarchy.

## Open Design source

- Open Design project: `cliply-modern-20260811` (`Cliply Modern UI`)
- Artifact: `cliply-modern.html`, Version 1
- Repository mirror: `designs/cliply-modern-open-design.html`
- Standalone installer implementation: `apps/cliply-installer/src`
- UI skill context: Open Design `ui-skills` entry, routed to upstream
  `ui-skills-root`, `impeccable`, and `colorize`

The Open Design palette uses achromatic graphite workspace surfaces:

- Background: `oklch(97.5% 0.003 255)`
- Surface: `oklch(99.7% 0.001 255)`
- Foreground: `oklch(19% 0.008 255)`
- Muted: `oklch(54% 0.007 255)`
- Border: `oklch(89.5% 0.005 255)`
- Default accent: coral `#FF6257`
- Coral hover / active: `#FF7066` / `#F75A50`
- Semantic mint: `#168F73` in light mode, `#3DD0AC` in dark mode
- Installer artwork uses the same coral/mint paper mark in the app icon,
  NSIS header, NSIS sidebar, setup logo, and standalone installer window.

The Windows shell uses native Acrylic behind the WebView. CSS materials then
layer translucent chrome, panes, cards, and controls over that system blur.

The Cliply mark is a flat two-sheet symbol: coral for the active clip, mint
for its history, a warm paper fold, and graphite content rules at larger
sizes. Small Windows sizes are optically simplified instead of downscaling the
full artwork. Avoid blue document stacks, clipboard outlines, lettermarks,
gradients, sparkles, glow, and mascot-like AI imagery.
At 16-48px Windows sizes, the mark uses an optically enlarged variant whose
visible bounds occupy roughly 88-92% of the canvas so it matches neighboring
taskbar icons without touching the raster edge.

User theme choices change the only accent hue. The accent is derived into
hover, active, selection, focus, link, pinned, and filled-primary roles.
Content types remain neutral and are distinguished by icon shape and text.
The default coral is shared with the active sheet in the Cliply mark. Mint is
reserved for local, successful, and history-complete states; it does not
compete with coral for primary actions. Glass surfaces remain neutral.

## Product posture

- Optimize for repeated scanning, keyboard selection, and immediate paste.
- Look like a finished desktop utility, not a dashboard or marketing page.
- Keep the current accent on focus, selection, links, pinned state, and paste.
- Fill only the primary paste action; secondary controls stay neutral.
- Use shadows only for menus and dialogs.
- Use tabular numerics for counts, timestamps, dimensions, and sizes.
- Interface copy is neutral, factual, and concise. Use standard desktop terms
  such as install, update, sync, save, delete, and retry; avoid slogans,
  rhetorical questions, personification, conversational reassurance, and
  marketing claims.
- Status text states the current result or required next action. Security and
  data-retention text describes verified behavior without repeating promises
  across the same screen.

## Layout contract

- No sidebar.
- No page heading, eyebrow label, hero, or outer workbench card.
- Compact native title bar.
- One compact 52px search command row with the result count at its trailing
  edge; the 40px search control retains the full keyboard-friendly hit area.
- Edge-to-edge two-pane workbench with a fluid 280-344px history pane and
  flexible detail pane. History uses roughly one third of available width and
  contracts smoothly as the window narrows. Both panes remain visible at the
  860px minimum window width and under high-DPI scaling.
- When the detail container falls below 600px, its content inset contracts from
  20-24px to 16px so preview and metadata retain usable width.
- Filter labels remain single-line at narrow widths and scroll horizontally
  inside their toolbar instead of changing the toolbar rhythm.
- Below a 680px detail-container width, secondary actions collapse to familiar
  icon buttons with tooltips. The primary Paste command remains fully labeled
  and visible, including at the combined minimum-width/minimum-height state.
- The workspace grid, detail pane, overlay viewport, and action bar all carry
  explicit zero-minimum and full-width constraints. Long code lines scroll only
  inside the code preview and cannot enlarge the detail column.
- History filters and the detail title occupy aligned 48px pane toolbars.
- A thin status bar spans the bottom edge.

## Type and spacing

- UI: system font stack led by Segoe UI Variable on Windows.
- Code: Cascadia Code, JetBrains Mono, Consolas, then monospace.
- Body: 13-14px. Supporting text: 10.5-12px.
- Use regular and semibold weights; avoid ornamental display typography.
- Do not alter letter spacing.
- Use an 8px spacing rhythm with 4px optical adjustments.

## Components

- Search is a 40px translucent control with an 8px radius, restrained focus
  ring, and a compact result-count control at the trailing edge.
- Filters are a compact segmented control. Only the active segment is raised.
- History rows are 68px inset scanning surfaces with neutral framed type icons.
- Selection combines a quiet accent tint, fine border, and short 3px accent rail.
- Detail content uses two functional panels: content preview and record info.
  These panels are not decorative cards; they provide labels, format context,
  scan boundaries, and consistent handling for code, text, links, and images.
- Metadata uses an icon-assisted two-column definition grid. Source window spans
  the full width; tags remain compact neutral chips.
- Secondary actions stay left; the primary paste action stays right.
- Secondary actions use quiet bordered controls; the primary paste action stays
  filled and slightly taller. Pane hierarchy comes from adjacent neutral surface
  weights rather than dark or accent-colored horizontal bands.
- Glass is the primary material language. The window uses native Windows
  Acrylic and a transparent WebView root. Internal material weights are:
  window 34%, chrome 58-68%, panes 36-48%, cards 60%, and controls 76% in light
  mode; dark mode uses heavier 62-84% graphite materials for legibility.
- Use stronger blur for structural layers (20-30px), 16px for content panels,
  and 14px for compact controls. Do not animate these large blur surfaces.
- Every translucent surface requires a faint highlight edge and a solid
  fallback for `prefers-reduced-transparency` and `prefers-contrast: more`.
- Empty, loading, error, hover, focus, disabled, and dark states are required.

## Motion

- Do not animate the application shell or layout.
- Do not add interaction animation unless explicitly requested.
- Existing dialog/toast feedback must remain under 200ms and respect
  `prefers-reduced-motion`.

## Standalone installer surface

- The standalone installer is a fixed `720 × 480` desktop task surface with a
  compact 44px draggable title bar. It uses the same Windows Acrylic material
  as the main app and retains solid fallbacks for reduced transparency and
  increased contrast.
- Install and update screens use three layers only: product introduction,
  functional setup panel, and bottom action rail. The setup panel contains the
  path, update/data-preservation status, install choices, and a compact factual
  summary so the window does not depend on decorative empty space.
- Coral is reserved for the install/update button, focus, selected options, and
  active progress. Mint communicates detected installations, preserved local
  data, and successful completion. Destructive uninstall actions use a separate
  muted red role and never borrow the primary coral action treatment.
- Working and completion screens reuse the same panel and action-rail geometry
  instead of becoming unrelated centered splash screens. Progress uses a solid
  fill, tabular percentage, concrete current step, and an explicit local-data
  assurance.
- Installer icons come from Lucide with a consistent line weight. Sparkles,
  glossy gradients, decorative icon tiles, and purple setup conventions are
  excluded so the surface reads as Cliply rather than a generic installer.
- Every install, update, uninstall, working, success, error, hover, focus,
  reduced-motion, reduced-transparency, and high-contrast state must fit the
  fixed logical viewport without clipping its primary action.

## Avoid

- Sidebars, gradients, glow, decorative imagery, nested cards, colored icon
  tiles, pill-heavy controls, and blue-tinted chrome.
- Marketing copy or visible usage instructions in the primary workspace.
- Adding a second accent, tinting chrome with the accent, coloring content
  types independently, or using status colors as decoration.
- Decorative glass with no hierarchy, same-opacity glass across every layer,
  or removing the solid fallback for reduced-transparency and increased-
  contrast users.
