/**
 * How a time falling between two frames is rounded (mirror of `se_domain::RoundingMode`):
 * `floor` toward −∞, `ceil` toward +∞, `nearest` with halves away from zero.
 */
export type RoundingMode = 'floor' | 'ceil' | 'nearest';

/** `dividend / divisor` rounded as `mode` says, with BigInt (no precision loss). */
function divideBig(dividend: bigint, divisor: bigint, mode: RoundingMode): bigint {
  let floor = dividend / divisor;
  let remainder = dividend % divisor;
  if (remainder < 0n) {
    floor -= 1n;
    remainder += divisor;
  }
  if (mode === 'floor' || remainder === 0n) return floor;
  if (mode === 'ceil') return floor + 1n;
  const doubled = remainder * 2n;
  return doubled > divisor || (doubled === divisor && floor >= 0n) ? floor + 1n : floor;
}

/**
 * The quotient of two integers (|values| ≤ 2^53, positive `divisor`) rounded as `mode` says,
 * exactly like the Rust `RoundingMode::divide`. Float division is used when its result can be
 * corrected exactly; BigInt otherwise.
 */
export function divideRounded(dividend: number, divisor: number, mode: RoundingMode): number {
  let floor = Math.floor(dividend / divisor);
  const product = floor * divisor;
  if (!Number.isSafeInteger(product)) {
    return Number(divideBig(BigInt(dividend), BigInt(divisor), mode));
  }
  let remainder = dividend - product;
  // The float quotient may land one off near an integer: fix it with the exact remainder.
  if (remainder < 0) {
    floor -= 1;
    remainder += divisor;
  } else if (remainder >= divisor) {
    floor += 1;
    remainder -= divisor;
  }
  if (mode === 'floor' || remainder === 0) return floor;
  if (mode === 'ceil') return floor + 1;
  const doubled = remainder * 2;
  return doubled > divisor || (doubled === divisor && floor >= 0) ? floor + 1 : floor;
}
