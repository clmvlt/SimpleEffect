import type { Meta, StoryObj } from '@storybook/react-vite';
import { Hand, Magnet, MousePointer2, Redo2, Scissors, Undo2 } from 'lucide-react';
import { useState } from 'react';

import { Tooltip } from '../../primitives/tooltip';
import { Toolbar, ToolbarButton, ToolbarGroup, ToolbarSeparator } from './toolbar';

function TimelineTools({ orientation }: { orientation?: 'horizontal' | 'vertical' }) {
  const [snapping, setSnapping] = useState(true);
  return (
    <Toolbar label="Outils de la timeline" orientation={orientation}>
      <ToolbarGroup label="Historique">
        <Tooltip content="Annuler" shortcut={['Ctrl', 'Z']}>
          <ToolbarButton icon={Undo2} label="Annuler" />
        </Tooltip>
        <Tooltip content="Rétablir" shortcut={['Ctrl', 'Maj', 'Z']}>
          <ToolbarButton icon={Redo2} label="Rétablir" />
        </Tooltip>
      </ToolbarGroup>
      <ToolbarSeparator />
      <ToolbarGroup label="Outils">
        <ToolbarButton icon={MousePointer2} label="Sélection" pressed />
        <ToolbarButton icon={Scissors} label="Lame" />
        <ToolbarButton icon={Hand} label="Main" />
      </ToolbarGroup>
      <ToolbarSeparator />
      <ToolbarButton
        icon={Magnet}
        label="Magnétisme"
        pressed={snapping}
        onClick={() => {
          setSnapping(!snapping);
        }}
      />
    </Toolbar>
  );
}

const meta = {
  title: 'Editor/Toolbar',
  component: TimelineTools,
} satisfies Meta<typeof TimelineTools>;

export default meta;
type Story = StoryObj<typeof meta>;

/** One tab stop; arrow keys move between tools. Hover for tooltips with shortcuts. */
export const Horizontal: Story = {};

export const Vertical: Story = { args: { orientation: 'vertical' } };
