// Vite import suffixes used by the engine's tests (Vitest runs them through Vite).

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
