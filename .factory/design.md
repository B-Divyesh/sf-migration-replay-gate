# Visual thesis — the replay observatory

## Direction and rationale

Migration Replay Gate uses a **luminous glass data landscape**: a dark, mineral-blue observatory where schema states appear as translucent database strata and migration paths travel between them as cold cyan light. The glass is explanatory, not ornamental—each layer represents clean, repeat, and partial-state replays, while the amber fracture plane marks the exact point an unsafe apply diverges. This makes an invisible CI process feel inspectable without borrowing a generic dashboard or gradient-hero language.

The treatment is intentionally single-mode. Deep navy gives terminal output and fine cyan traces reliable contrast, and it lets the original replay landscape carry the product’s identity. Every essential state also has an icon and text label, so color never bears meaning alone.

## Tokens

- Background `#061117` (abyss navy); raised background `#0A1B22`.
- Glass surface `rgba(20, 52, 62, .72)`; strong surface `#11303A`; hairline `#2C5963`.
- Primary text `#F2F8F5`; muted text `#AFC4C3`.
- Replay cyan `#77E8D2`; accent contrast `#06211E`.
- Success `#72D7A5`; warning `#F2C66D`; danger `#FF8E82`.
- Focus ring `#F7D27D` with a dark offset halo.

All body/muted combinations target WCAG AA at 16 px or larger. Translucent surfaces always sit over the painted navy background; copy never relies on the generated image for contrast.

## Typography

- Display: `Arial Narrow`, `Aptos Narrow`, system sans-serif—compressed, uppercase sparingly, like a database console’s instrument labels.
- Body: `Inter`, `ui-sans-serif`, system sans-serif. Inter is self-hosted as a single WOFF2 subset if available; the system fallback is first-class.
- Code: `ui-monospace`, `SFMono-Regular`, Consolas, monospace with tabular figures.
- Scale: 14 / 16 / 20 / 28 / clamp(44–72) px. Body is never below 16 px. Reading measure stays under 72 characters.

## Spacing and shape

The base unit is 4 px, with a working rhythm of 8 / 12 / 16 / 24 / 32 / 48 / 72 px. Large sections use 96 px on wide screens and 64 px on phones. Glass planes use asymmetric 20–28 px corners, while terminal and state rows use 8–12 px radii. The contrast keeps the landscape atmospheric and the diagnostic data exact.

## Interaction grammar

- Primary actions are filled cyan capsules; secondary actions are outlined glass controls.
- Scenario selection behaves like a physical three-position replay rail: Clean → Repeat → Partial. Arrow keys change the selected scenario; the result region is announced politely.
- Copy controls confirm in-place (“Copied”) and reset without shifting layout.
- Cards are reserved for independent replay outcomes. Explanatory content groups by proximity and vertical rules.
- Mobile drops ornamental coordinate labels, stacks the replay rail, and keeps the command/result first.

## Motion policy

One entrance sequence reveals the three schema planes from back to front over 480 ms. UI changes use 180–240 ms opacity and transform transitions. The illustration never loops. Under `prefers-reduced-motion: reduce`, transforms and smooth scrolling are removed and state changes are instantaneous; depth remains through overlap, border luminosity, and scale.

## Asset plan and provenance

- `site/public/replay-landscape.webp`: original AI-generated raster, used as the explanatory hero landscape. Generated on 2026-08-27 with `/opt/fleet/lib/gen-image.sh` using the factory `factory-image` deployment, then downsampled and encoded locally to WebP. Prompt: “Wide editorial 3D illustration for a developer tool landing page. A dark navy data observatory containing three staggered translucent glass database planes, each plane formed from precise rows and columns, connected by a single luminous mint-cyan replay path. The first two planes align perfectly; the third has one restrained amber fracture line showing schema divergence. Oblique isometric perspective, deep mineral blue void, subtle volumetric haze, crisp etched grid details, premium technical atmosphere, generous negative space around the central object, no people, no logos, no letters, no text, no UI screenshot, no generic gradient, no watermark.” The generated image is an original project asset; no stock imagery or third-party marks are used.
- Product mark and status glyphs are hand-authored CSS/SVG geometry derived from stacked schema planes. No external icon library.

