/**
 * Frame formats in forms (new project, sequence format, versions): ratios, resolutions, frame
 * rates and their fields, shared by the features that edit a format.
 */
export { FormatFields, type FormatFieldsProps } from './components/FormatFields';
export { FrameRateSelect, type FrameRateSelectProps } from './components/FrameRateSelect';
export { frameRateFromValue, frameRateValue } from './format-choices';
export {
  draftFromFormat,
  draftToFormat,
  isDraftSizeValid,
  withDraftRatio,
  type FormatDraft,
} from './format-draft';
export { aspectRatioOf, formatBadgeLabel, framesPerSecond, sameFrameRate } from './format-label';
export {
  MAX_FRAME_SIDE,
  MIN_FRAME_SIDE,
  defaultResolutionFor,
  isValidResolution,
  resolutionFor,
  resolutionsFor,
  type Resolution,
} from './resolutions';
