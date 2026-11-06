import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { createHashRouter, RouterProvider } from 'react-router';

import { AppProviders } from './app/AppProviders';
import { createAppRoutes } from './app/routes';
import { createI18n } from './i18n';
import { createCoreClient } from './shared/core';
import { uiPreferencesStorage } from './shared/storage';
import './app/styles.css';

const container = document.getElementById('root');
if (container === null) {
  throw new Error('index.html must contain a <div id="root"> element for the React app.');
}

// Composition root: one core client (it reconnects by itself when se-core restarts), one
// i18next instance, and a hash router (the renderer is a file in production, so paths live in
// the URL fragment).
const coreClient = createCoreClient(window.simpleEffect);
const i18n = createI18n();
const router = createHashRouter(createAppRoutes({ layoutStorage: uiPreferencesStorage() }));

createRoot(container).render(
  <StrictMode>
    <AppProviders coreClient={coreClient} i18n={i18n}>
      <RouterProvider router={router} />
    </AppProviders>
  </StrictMode>,
);
