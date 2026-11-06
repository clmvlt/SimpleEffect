/**
 * A frame-count `HH:MM:SS:FF` codec at a fixed rate, for stories and tests only: the real codec
 * converts flicks with the sequence frame rate and lives with the domain helpers.
 */
export function createSampleFrameCodec(framesPerSecond: number) {
  const pad = (part: number) => String(part).padStart(2, '0');
  return {
    format: (frames: number): string => {
      const totalSeconds = Math.floor(frames / framesPerSecond);
      return [
        Math.floor(totalSeconds / 3600),
        Math.floor(totalSeconds / 60) % 60,
        totalSeconds % 60,
        frames % framesPerSecond,
      ]
        .map(pad)
        .join(':');
    },
    parse: (text: string): number | null => {
      const match = /^(\d{1,2}):(\d{2}):(\d{2}):(\d{2})$/.exec(text.trim());
      if (match === null) {
        return null;
      }
      const [hours, minutes, seconds, frames] = match.slice(1).map(Number) as [
        number,
        number,
        number,
        number,
      ];
      if (minutes > 59 || seconds > 59 || frames >= framesPerSecond) {
        return null;
      }
      return ((hours * 60 + minutes) * 60 + seconds) * framesPerSecond + frames;
    },
  };
}
