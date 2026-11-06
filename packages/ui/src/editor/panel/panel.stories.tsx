import type { Meta, StoryObj } from '@storybook/react-vite';
import { Ellipsis, Plus } from 'lucide-react';

import { IconButton } from '../../primitives/icon-button';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '../../primitives/tabs';
import { Panel, PanelActions, PanelBody, PanelHeader, PanelTitle } from './panel';

const meta = {
  title: 'Editor/Panel',
  component: Panel,
  parameters: { layout: 'fullscreen' },
  decorators: [
    (Story) => (
      <div className="h-80 w-96 border border-border">
        <Story />
      </div>
    ),
  ],
} satisfies Meta<typeof Panel>;

export default meta;
type Story = StoryObj<typeof meta>;

/** Title, actions and a scrolling body. */
export const Default: Story = {
  render: () => (
    <Panel>
      <PanelHeader>
        <PanelTitle>Médias</PanelTitle>
        <PanelActions>
          <IconButton icon={Plus} label="Importer" size="xs" />
          <IconButton icon={Ellipsis} label="Plus d'options" size="xs" />
        </PanelActions>
      </PanelHeader>
      <PanelBody>
        <ul className="flex flex-col gap-1 p-2 text-sm text-text-muted">
          {Array.from({ length: 30 }, (_, index) => (
            <li key={index}>Plan {index + 1}.mp4</li>
          ))}
        </ul>
      </PanelBody>
    </Panel>
  ),
};

/** Tabs replace the title: the panel is then named with `aria-label`. */
export const WithTabs: Story = {
  render: () => (
    <Panel aria-label="Inspecteur et IA">
      <Tabs defaultValue="inspector" className="min-h-0 flex-1 gap-0">
        <PanelHeader>
          <TabsList variant="underline" size="xs" className="h-full border-b-0">
            <TabsTrigger value="inspector">Inspecteur</TabsTrigger>
            <TabsTrigger value="ai">IA</TabsTrigger>
          </TabsList>
        </PanelHeader>
        <PanelBody>
          <TabsContent value="inspector" className="p-3 text-sm text-text-muted">
            Propriétés de la sélection.
          </TabsContent>
          <TabsContent value="ai" className="p-3 text-sm text-text-muted">
            Tâches de l'assistant.
          </TabsContent>
        </PanelBody>
      </Tabs>
    </Panel>
  ),
};

/** Content that handles its own size (viewer, timeline) opts out of scrolling. */
export const FixedBody: Story = {
  render: () => (
    <Panel>
      <PanelHeader>
        <PanelTitle>Viewer</PanelTitle>
      </PanelHeader>
      <PanelBody scrollable={false} className="flex items-center justify-center bg-bg">
        <div className="aspect-video w-3/4 rounded-sm border border-border bg-panel-raised" />
      </PanelBody>
    </Panel>
  ),
};
