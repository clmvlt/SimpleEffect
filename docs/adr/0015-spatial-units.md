# 0015 — Spatial units: normalised positions, design pixels on the shorter side

- Status: Accepted
- Date: 2026-11-06

## Context
A project may hold several sequences of different formats: a 16:9 edit (1920 × 1080), its 9:16 version for Shorts/Reels/TikTok (1080 × 1920), a 1:1 or 4:5 post, sometimes a 4K master. Items (texts, shapes, images, media clips, templates) must keep their placement and look when a sequence changes format or is duplicated into another one, and AI agents must reason about positions and sizes without converting units on every call.

Pixel coordinates tie every value to one resolution. Fully normalised sizes (fractions of the width or height) make a font change size whenever the aspect ratio changes, and are hard to read ("font size 0.044"). A reference resolution is needed for sizes, and the choice of that reference decides how sizes behave across aspect ratios.

## Decision
- **Positions** are normalised in the sequence frame: `(0, 0)` top-left, `(1, 1)` bottom-right, whatever the format. Values outside `[0, 1]` are off-screen and allowed.
- **Anchors** are normalised in the item's own box: `(0.5, 0.5)` is its centre.
- **Scales** are relative factors: `(1, 1)` is the natural size; negative values mirror.
- **Rotations** are degrees, clockwise (the y axis points down).
- **Opacity** is a number in `[0, 1]`.
- **Sizes** (font size, stroke width, corner radius, shape extents, paddings…) are **design pixels** (`DesignPixels`, an `f64`): pixels of a frame whose **shorter side is 1080** (`DESIGN_REFERENCE_SIDE`). Real pixels = design pixels × `min(width, height) / 1080` (`Format::design_scale`).
  - 1 design pixel is 1 real pixel in every 1080-class format: 16:9 1920 × 1080, 9:16 1080 × 1920, 1:1 1080 × 1080, 4:5 1080 × 1350. It is 2 real pixels in 4K (3840 × 2160) and 0.67 in 720p.
  - The UI shows and edits pixels of the current format, which equal design pixels for every 1080-class format.

The prompt that introduced this decision proposed a reference **height** of 1080. The user chose the shorter side instead: with a height reference, every size in a 9:16 sequence is 1.78× larger relative to the frame width than in 16:9 (and 1.25× in 4:5), so stored and displayed values diverge and agents would have to convert constantly.

## Consequences
- The same value means the same visual size relative to the frame's shorter side in every format; positions keep their relative place. Converting an edit from 16:9 to 9:16 keeps fonts readable but may need re-layout of wide elements (a 1200 px title does not fit a 1080 px wide frame): that is the job of format conversion and reframing (prompts 16 and 26), not of the units.
- The engine (prompt 12) multiplies design pixels by the format's design scale when drawing; export renders at the sequence size through the same code path.
- MCP tools (prompts 18–19) document positions as normalised and sizes as pixels at 1080p-class resolution; agents need no unit conversion for the common formats.
- Shape and text-box extents are design pixels too, so a full-frame shape is expressed by its fit mode rather than by a size (decided with shapes in prompt 15).

## Alternatives considered
- **Pixels of the sequence format**: simple for one format, but every format change or duplication rewrites every value. Rejected.
- **Normalised sizes** (fraction of width or height): resolution-independent, but unreadable for users and agents, and a font changes size with the aspect ratio. Rejected.
- **Design pixels with a reference height of 1080** (the original proposal): identical to the shorter side in landscape, but enlarges everything in portrait formats. Rejected by the user.
- **Design pixels with a reference width of 1920**: shrinks everything in portrait formats (a 9:16 frame is only 1080 wide, so text would be 56 % of its 16:9 size). Rejected.
