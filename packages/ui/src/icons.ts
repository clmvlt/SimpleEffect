/**
 * `@se/ui/icons`: the icon set (lucide), re-exported so apps never import `lucide-react` directly.
 * Render icons through the `Icon` component for normalised sizes and accessibility:
 * `import { Icon } from '@se/ui'; import { Scissors } from '@se/ui/icons'; <Icon icon={Scissors} />`.
 *
 * A separate entry point keeps the ~1,600 icon names out of the main `@se/ui` namespace (lucide
 * also exports names such as `Icon` that would clash); bundlers keep only the icons used.
 */
export * from 'lucide-react';
