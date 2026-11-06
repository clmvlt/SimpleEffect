import type { Meta, StoryObj } from '@storybook/react-vite';
import { AudioLines, Captions, Film } from 'lucide-react';

import { Icon } from '../icon';
import { Tabs, TabsContent, TabsList, TabsTrigger, type TabsListProps } from './tabs';

/** Story controls are the list's look; the composition is fixed. */
function InspectorTabs({ variant, size }: Pick<TabsListProps, 'variant' | 'size'>) {
  return (
    <Tabs defaultValue="video" className="w-80">
      <TabsList variant={variant} size={size} aria-label="Propriétés du clip">
        <TabsTrigger value="video">Vidéo</TabsTrigger>
        <TabsTrigger value="audio">Audio</TabsTrigger>
        <TabsTrigger value="effects" disabled>
          Effets
        </TabsTrigger>
        <TabsTrigger value="captions">Sous-titres</TabsTrigger>
      </TabsList>
      <TabsContent value="video" className="text-sm text-text-muted">
        Position, échelle, rotation et opacité.
      </TabsContent>
      <TabsContent value="audio" className="text-sm text-text-muted">
        Volume, panoramique et fondus.
      </TabsContent>
      <TabsContent value="effects" className="text-sm text-text-muted">
        Effets appliqués au clip.
      </TabsContent>
      <TabsContent value="captions" className="text-sm text-text-muted">
        Style et position des sous-titres.
      </TabsContent>
    </Tabs>
  );
}

const meta = {
  title: 'Primitives/Tabs',
  component: InspectorTabs,
  args: { variant: 'segmented', size: 'sm' },
  argTypes: {
    variant: { control: 'inline-radio', options: ['segmented', 'underline'] },
    size: { control: 'inline-radio', options: ['xs', 'sm'] },
  },
} satisfies Meta<typeof InspectorTabs>;

export default meta;
type Story = StoryObj<typeof meta>;

/** Compact pill group, for toolbars and dense inspectors. */
export const Segmented: Story = {};

/** Text tabs over a border, for panel headers. */
export const Underline: Story = { args: { variant: 'underline' } };

export const ExtraSmall: Story = { args: { size: 'xs' } };

/** Icons before the labels (decorative: the text names the tab). */
export const WithIcons: Story = {
  render: () => (
    <Tabs defaultValue="video">
      <TabsList aria-label="Pistes">
        <TabsTrigger value="video">
          <Icon icon={Film} size="sm" />
          Vidéo
        </TabsTrigger>
        <TabsTrigger value="audio">
          <Icon icon={AudioLines} size="sm" />
          Audio
        </TabsTrigger>
        <TabsTrigger value="captions">
          <Icon icon={Captions} size="sm" />
          Sous-titres
        </TabsTrigger>
      </TabsList>
      <TabsContent value="video" className="text-sm text-text-muted">
        Pistes vidéo
      </TabsContent>
      <TabsContent value="audio" className="text-sm text-text-muted">
        Pistes audio
      </TabsContent>
      <TabsContent value="captions" className="text-sm text-text-muted">
        Pistes de sous-titres
      </TabsContent>
    </Tabs>
  ),
};
