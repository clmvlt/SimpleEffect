// A minimal Chrome DevTools Protocol client for the performance scripts (prompt 29): connects to
// the built app started with `--remote-debugging-port`, evaluates code in the renderer, drives the
// mouse and talks to se-core through the renderer's own connection details.

/** Connects to the first page of the app listening on `port`. */
export async function connectPage(port) {
  const targets = await (await fetch(`http://127.0.0.1:${port}/json`)).json();
  const page = targets.find((target) => target.type === 'page');
  if (page === undefined) throw new Error(`no page on the DevTools port ${port}`);
  const socket = new WebSocket(page.webSocketDebuggerUrl);
  await new Promise((resolve, reject) => {
    socket.onopen = resolve;
    socket.onerror = () => reject(new Error('cannot reach the DevTools socket'));
  });
  let nextId = 0;
  const pending = new Map();
  socket.onmessage = (message) => {
    const data = JSON.parse(String(message.data));
    const request = pending.get(data.id);
    if (request === undefined) return;
    pending.delete(data.id);
    if (data.error) request.reject(new Error(`${request.method}: ${data.error.message}`));
    else request.resolve(data.result);
  };
  const send = (method, params = {}) =>
    new Promise((resolve, reject) => {
      nextId += 1;
      pending.set(nextId, { resolve, reject, method });
      socket.send(JSON.stringify({ id: nextId, method, params }));
    });
  return { send, close: () => socket.close() };
}

/** Evaluates `expression` (awaited) in the page and returns its JSON value. */
export async function evaluate(page, expression) {
  const result = await page.send('Runtime.evaluate', {
    expression,
    awaitPromise: true,
    returnByValue: true,
  });
  if (result.exceptionDetails) {
    const description = result.exceptionDetails.exception?.description;
    throw new Error(description ?? result.exceptionDetails.text);
  }
  return result.result.value;
}

/** Installs `window.__perfCore(command)`: a second socket to se-core, like the renderer's. */
export async function installCoreHelper(page) {
  await evaluate(
    page,
    `window.__perfCore ??= (async () => {
      const connection = await window.simpleEffect.getCoreConnection();
      const url = new URL('/rpc', connection.url);
      url.protocol = 'ws:';
      url.searchParams.set('t', connection.token);
      const socket = new WebSocket(url);
      await new Promise((resolve, reject) => { socket.onopen = resolve; socket.onerror = reject; });
      let id = 0;
      const pending = new Map();
      socket.onmessage = (event) => {
        const message = JSON.parse(event.data);
        const request = pending.get(message.id);
        if (request === undefined) return;
        pending.delete(message.id);
        if ('error' in message) request.reject(new Error(JSON.stringify(message.error)));
        else request.resolve(message.result);
      };
      return (command) => new Promise((resolve, reject) => {
        id += 1;
        pending.set(id, { resolve, reject });
        socket.send(JSON.stringify({ type: 'request', id, command }));
      });
    })(); true`,
  );
}

/** Sends `command` to se-core and returns its result. */
export function core(page, command) {
  return evaluate(page, `window.__perfCore.then((send) => send(${JSON.stringify(command)}))`);
}

/** Waits until `expression` is truthy in the page (polled every 250 ms). */
export async function waitFor(page, expression, timeoutMs = 60_000) {
  const end = Date.now() + timeoutMs;
  for (;;) {
    const value = await evaluate(page, expression);
    if (value) return value;
    if (Date.now() > end) throw new Error(`timed out waiting for ${expression}`);
    await new Promise((resolve) => setTimeout(resolve, 250));
  }
}

/** Clicks the centre of the first element matching `selector` whose text contains `text`. */
export async function clickText(page, selector, text) {
  const point = await evaluate(
    page,
    `(() => {
      const element = [...document.querySelectorAll(${JSON.stringify(selector)})]
        .find((candidate) => candidate.textContent.includes(${JSON.stringify(text)}));
      if (!element) return null;
      const box = element.getBoundingClientRect();
      return { x: box.x + box.width / 2, y: box.y + box.height / 2 };
    })()`,
  );
  if (point === null) throw new Error(`nothing to click: ${selector} « ${text} »`);
  for (const type of ['mousePressed', 'mouseReleased']) {
    await page.send('Input.dispatchMouseEvent', { type, ...point, button: 'left', clickCount: 1 });
  }
}
