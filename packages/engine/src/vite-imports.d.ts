// Vite import suffixes used by the engine (bundled by Vite in the app, Vitest in tests).

/** A file's text (`import text from './data.json?raw'`). */
declare module '*?raw' {
  const content: string;
  export default content;
}

/** A served URL of a file (`import url from './clip.mp4?url'`), for golden-frame fixtures. */
declare module '*?url' {
  const url: string;
  export default url;
}

/** The URL of a module bundled as its own file (`?worker&url`): the meter's AudioWorklet. */
declare module '*?worker&url' {
  const url: string;
  export default url;
}
