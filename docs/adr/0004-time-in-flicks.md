# 0004 — Time represented as integer flicks

- Status: Accepted
- Date: 2026-11-06

## Context
A video editor mixes frame rates (23.976, 24, 25, 29.97, 30, 50, 59.94, 60, 120) and sample rates (44.1, 48, 96 kHz). Floating-point seconds accumulate rounding errors, cause off-by-one-frame bugs and make equality checks unreliable.

## Decision
- All instants and durations in the domain are `Flicks(i64)`: **705,600,000 flicks per second**. This base divides every common frame rate (including NTSC x/1001 rates) and sample rate exactly.
- Frame rates are rationals (`FrameRate { numerator, denominator }`).
- Conversions to seconds, frames or timecode strings happen only at boundaries: UI display, MCP tool I/O, export writers.
- In TypeScript, flicks are a branded `number`, exact up to ±2^53 (~147 days). The domain rejects values outside that range.

## Consequences
- Exact arithmetic, no drift, simple integer comparisons.
- Every boundary needs explicit conversion helpers (`Flicks::from_seconds_f64`, `Timecode::parse`, `formatTimecode`), which must be the only conversion paths.
- Exports to FCP7 XML / OTIO convert to frame counts or rational times at the sequence rate.

## Alternatives considered
- **f64 seconds**: simple, but rounding errors. Rejected.
- **Rational time everywhere (value, rate)** like OTIO: exact but heavier to manipulate and compare. Rejected for internal use; used only in OTIO export.
- **Premiere ticks (254,016,000,000/s)**: exact too but overflows faster and is less common. Rejected.
