/**
 * Extensions offered by the import dialog's « Médias » filter. Anything else can still be picked
 * through « Tous les fichiers »: the core probes every file and reports what it cannot read.
 */
export const MEDIA_EXTENSIONS: readonly string[] = [
  // Video
  'mp4',
  'm4v',
  'mov',
  'mkv',
  'webm',
  'avi',
  'mxf',
  'mts',
  'm2ts',
  'mpg',
  'mpeg',
  'wmv',
  '3gp',
  // Sound
  'wav',
  'mp3',
  'm4a',
  'aac',
  'flac',
  'ogg',
  'opus',
  'aif',
  'aiff',
  'wma',
  // Images
  'png',
  'jpg',
  'jpeg',
  'webp',
  'gif',
  'bmp',
  'tif',
  'tiff',
];
