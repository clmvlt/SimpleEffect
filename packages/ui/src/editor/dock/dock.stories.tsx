import type { Meta, StoryObj } from '@storybook/react-vite';

import { Panel, PanelBody, PanelHeader, PanelTitle } from '../panel';
import { Dock, DockHandle, DockPanel } from './dock';
import type { DockLayoutStorage } from './dock-layout-storage';

/** Session storage keeps the sizes while browsing Storybook, without polluting localStorage. */
const storyStorage: DockLayoutStorage = {
  getItem: (key) => sessionStorage.getItem(`storybook.${key}`),
  setItem: (key, value) => {
    sessionStorage.setItem(`storybook.${key}`, value);
  },
};

function DemoPanel({ title }: { title: string }) {
  return (
    <Panel>
      <PanelHeader>
        <PanelTitle>{title}</PanelTitle>
      </PanelHeader>
      <PanelBody className="p-2 text-sm text-text-muted">Glissez les séparateurs.</PanelBody>
    </Panel>
  );
}

const meta = {
  title: 'Editor/Dock',
  component: Dock,
  parameters: { layout: 'fullscreen' },
  decorators: [
    (Story) => (
      <div className="h-[32rem] w-full">
        <Story />
      </div>
    ),
  ],
  args: { layoutId: 'story.dock' },
} satisfies Meta<typeof Dock>;

export default meta;
type Story = StoryObj<typeof meta>;

/** Two panels side by side; focus a handle and use the arrow keys to resize from the keyboard. */
export const Horizontal: Story = {
  render: (args) => (
    <Dock {...args}>
      <DockPanel id="left" defaultSize="30%" minSize={160}>
        <DemoPanel title="Médias" />
      </DockPanel>
      <DockHandle label="Redimensionner les panneaux" />
      <DockPanel id="right" minSize={200}>
        <DemoPanel title="Viewer" />
      </DockPanel>
    </Dock>
  ),
};

/** The editor arrangement: nested docks, sizes kept in session storage across reloads. */
export const EditorLayout: Story = {
  args: { layoutId: 'story.editor.rows', orientation: 'vertical', storage: storyStorage },
  render: (args) => (
    <Dock {...args}>
      <DockPanel id="top" defaultSize="62%" minSize={160}>
        <Dock layoutId="story.editor.columns" storage={storyStorage}>
          <DockPanel id="media" defaultSize="22%" minSize={180}>
            <DemoPanel title="Médias" />
          </DockPanel>
          <DockHandle label="Redimensionner Médias et Viewer" />
          <DockPanel id="viewer" minSize={240}>
            <DemoPanel title="Viewer" />
          </DockPanel>
          <DockHandle label="Redimensionner Viewer et Inspecteur" />
          <DockPanel id="inspector" defaultSize="26%" minSize={220}>
            <DemoPanel title="Inspecteur" />
          </DockPanel>
        </Dock>
      </DockPanel>
      <DockHandle label="Redimensionner la timeline" />
      <DockPanel id="timeline" minSize={120}>
        <DemoPanel title="Timeline" />
      </DockPanel>
    </Dock>
  ),
};
