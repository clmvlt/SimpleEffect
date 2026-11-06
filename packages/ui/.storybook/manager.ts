import { addons } from 'storybook/manager-api';
import { themes } from 'storybook/theming';

// Dark Storybook chrome, matching the app.
addons.setConfig({ theme: themes.dark });
