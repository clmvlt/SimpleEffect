import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';

import { App } from './app/App';
import { CoreClientProvider, createCoreClient } from './shared/core';
import './app/styles.css';

const container = document.getElementById('root');
if (container === null) {
  throw new Error('index.html must contain a <div id="root"> element for the React app.');
}

// One client for the app's lifetime: it reconnects by itself when se-core restarts.
const coreClient = createCoreClient(window.simpleEffect);

createRoot(container).render(
  <StrictMode>
    <CoreClientProvider client={coreClient}>
      <App />
    </CoreClientProvider>
  </StrictMode>,
);
