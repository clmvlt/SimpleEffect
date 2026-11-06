/**
 * Keyboard focus indicator shared by interactive primitives: a 2 px `focus-ring` outline drawn just
 * outside the element, only for keyboard focus (`:focus-visible`). An outline (not a box-shadow
 * ring) leaves the element's own shadow and border free and survives forced-colour modes.
 */
export const focusRing =
  'outline-none focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-focus-ring';
