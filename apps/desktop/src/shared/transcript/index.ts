/**
 * Transcripts seen from the renderer (prompt 21): the words of a sequence in sequence time,
 * mirroring `se_domain::sequence_transcript` (same vectors), shared by the transcript panel and
 * the text-based editing and captions of later prompts.
 */
export {
  EMPTY_SEQUENCE_TRANSCRIPT,
  sequenceTranscript,
  wordAt,
  type SequenceSegment,
  type SequenceTranscript,
  type SequenceWord,
  type SoundClip,
} from './sequence-words';
export { assetsOf, soundClipsOf } from './sound-clips';
