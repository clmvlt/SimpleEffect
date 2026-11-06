import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';

import { App } from './app/App';
import './app/styles.css';

const container = document.getElementById('root');
if (container === null) {
  throw new Error('index.html must contain a <div id="root"> element for the React app.');
}

createRoot(container).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
