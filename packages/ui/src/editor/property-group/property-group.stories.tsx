import type { Meta, StoryObj } from '@storybook/react-vite';
import { RotateCcw } from 'lucide-react';
import { useState } from 'react';

import { IconButton } from '../../primitives/icon-button';
import { Switch } from '../../primitives/switch';
import { KeyframeButton } from '../keyframe-button';
import { ScrubInput } from '../scrub-input';
import { PropertyGroup, PropertyRow } from './property-group';

function TransformGroup() {
  const [opacity, setOpacity] = useState(100);
  const [x, setX] = useState(0);
  const [y, setY] = useState(0);
  const [visible, setVisible] = useState(true);
  return (
    <div className="w-80 bg-panel">
      <PropertyGroup
        title="Transformation"
        actions={<IconButton icon={RotateCcw} label="Réinitialiser la transformation" size="xs" />}
      >
        <PropertyRow label="Position">
          <ScrubInput label="X" size="xs" value={x} onChange={setX} unit="px" />
          <ScrubInput label="Y" size="xs" value={y} onChange={setY} unit="px" />
        </PropertyRow>
        <PropertyRow
          label="Opacité"
          htmlFor="story-opacity"
          modified={opacity !== 100}
          modifiedLabel="modifié"
          keyframe={
            <KeyframeButton state="onKeyframe" label="Clé d'opacité" onToggle={() => undefined} />
          }
        >
          <ScrubInput
            id="story-opacity"
            size="xs"
            value={opacity}
            onChange={setOpacity}
            min={0}
            max={100}
            unit="%"
          />
        </PropertyRow>
        <PropertyRow label="Visible" htmlFor="story-visible">
          <Switch id="story-visible" checked={visible} onCheckedChange={setVisible} />
        </PropertyRow>
      </PropertyGroup>
      <PropertyGroup title="Audio" defaultOpen={false}>
        <PropertyRow label="Volume" htmlFor="story-volume">
          <ScrubInput id="story-volume" size="xs" value={0} onChange={() => undefined} unit="dB" />
        </PropertyRow>
      </PropertyGroup>
    </div>
  );
}

const meta = {
  title: 'Editor/PropertyGroup',
  component: TransformGroup,
} satisfies Meta<typeof TransformGroup>;

export default meta;
type Story = StoryObj<typeof meta>;

/** Inspector groups: rows align their labels, controls and keyframe buttons. */
export const Inspector: Story = {};
