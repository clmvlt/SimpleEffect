import type { Meta, StoryObj } from '@storybook/react-vite';

import { ColourSwatches } from './catalog/colour-swatches';
import { LayersAndMotion, RadiusAndElevation, SpacingScale } from './catalog/layout-scales';
import { TypeScale } from './catalog/type-scale';

/**
 * The design tokens, read from the token stylesheets (`src/tokens/*.css`) at build time: adding a
 * token there adds it here. Use the Tailwind utility shown next to each token, never its value.
 */
const meta = {
  title: 'Foundations/Tokens',
  parameters: { layout: 'padded' },
} satisfies Meta;

export default meta;
type Story = StoryObj<typeof meta>;

export const Colours: Story = { render: () => <ColourSwatches /> };

export const Typography: Story = { render: () => <TypeScale /> };

export const Spacing: Story = { render: () => <SpacingScale /> };

export const RadiiAndElevation: Story = { render: () => <RadiusAndElevation /> };

export const LayersAndMotionTokens: Story = {
  name: 'Layers and motion',
  render: () => <LayersAndMotion />,
};
