import {
  Dock,
  DockHandle,
  DockPanel,
  Panel,
  PanelBody,
  PanelHeader,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
  type DockLayoutStorage,
} from '@se/ui';
import { useTranslation } from 'react-i18next';

import { AiView, useAiTaskSync } from '../../features/ai';
import { AnimationsPanel, useCreateGraphic } from '../../features/graphics';
import { InspectorView } from '../../features/inspector';
import { MediaBinPanel } from '../../features/media-bin';
import { TimelinePanel } from '../../features/timeline';
import { TranscriptView } from '../../features/transcript';
import { ViewerPanel } from '../../features/viewer';
import { isSidePanelTab, showSidePanelTab, useSidePanelTab } from '../../shared/side-panel';

/** Props of {@link EditorLayout}. */
export interface EditorLayoutProps {
  /** Where panel sizes are saved (the app's UI preferences). */
  readonly layoutStorage: DockLayoutStorage;
}

/**
 * The editor layout: media on the left, viewer in the centre, inspector and AI as tabs on the
 * right, timeline at the bottom; it keeps the project's AI tasks in sync (`useAiTaskSync`). Every boundary is resizable (pointer or keyboard) and the sizes
 * come back on the next start. Layout ids are stable: renaming one resets users' sizes.
 */
export function EditorLayout({ layoutStorage }: EditorLayoutProps) {
  const { t } = useTranslation();
  const creator = useCreateGraphic();
  useAiTaskSync();
  return (
    <div className="min-h-0 flex-1">
      <Dock layoutId="editor.rows" orientation="vertical" storage={layoutStorage}>
        <DockPanel id="workspace" defaultSize="62%" minSize={220}>
          <Dock layoutId="editor.columns" storage={layoutStorage}>
            <DockPanel id="media" defaultSize="20%" minSize={200}>
              <MediaBinPanel
                onAddImage={
                  creator === null
                    ? undefined
                    : (asset) => {
                        creator.createImage(asset.id);
                      }
                }
              />
            </DockPanel>
            <DockHandle label={t('editor.resize.mediaViewer')} />
            <DockPanel id="viewer" minSize={320}>
              <ViewerPanel />
            </DockPanel>
            <DockHandle label={t('editor.resize.viewerSide')} />
            <DockPanel id="side" defaultSize="26%" minSize={260}>
              <SidePanel />
            </DockPanel>
          </Dock>
        </DockPanel>
        <DockHandle label={t('editor.resize.workspaceTimeline')} />
        <DockPanel id="timeline" minSize={140}>
          <TimelinePanel />
        </DockPanel>
      </Dock>
    </div>
  );
}

/**
 * The right column: « Inspecteur », « Animations », « IA » and « Transcription » share one panel as tabs (features
 * may switch it, e.g. the timeline's « Propriétés »).
 */
function SidePanel() {
  const { t } = useTranslation();
  const tab = useSidePanelTab();
  return (
    <Panel aria-label={t('editor.sidePanel')}>
      <Tabs
        value={tab}
        onValueChange={(value) => {
          if (isSidePanelTab(value)) showSidePanelTab(value);
        }}
        className="min-h-0 flex-1 gap-0"
      >
        <PanelHeader>
          <TabsList variant="underline" size="xs" className="h-full border-b-0">
            <TabsTrigger value="inspector">{t('editor.inspector.title')}</TabsTrigger>
            <TabsTrigger value="animations">{t('editor.animations.title')}</TabsTrigger>
            <TabsTrigger value="ai">{t('editor.ai.title')}</TabsTrigger>
            <TabsTrigger value="transcript">{t('editor.transcript.title')}</TabsTrigger>
          </TabsList>
        </PanelHeader>
        <PanelBody scrollable={false}>
          <TabsContent value="inspector" className="size-full">
            <InspectorView />
          </TabsContent>
          <TabsContent value="animations" className="size-full">
            <AnimationsPanel />
          </TabsContent>
          <TabsContent value="ai" className="size-full">
            <AiView />
          </TabsContent>
          <TabsContent value="transcript" className="size-full">
            <TranscriptView />
          </TabsContent>
        </PanelBody>
      </Tabs>
    </Panel>
  );
}
