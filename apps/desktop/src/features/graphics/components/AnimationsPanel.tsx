import type { AnimationPhase } from '@se/protocol';
import {
  EmptyState,
  ScrollArea,
  ScrubInput,
  Switch,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from '@se/ui';
import { Sparkles } from '@se/ui/icons';
import { useTranslation } from 'react-i18next';

import { useApplyPreset } from '../hooks/use-apply-preset';
import { usePersistedGraphicsPreferences } from '../hooks/use-graphics-preferences';
import { useAnimationPresets } from '../store/animation-presets-store';
import {
  ANIMATION_SECONDS_RANGE,
  setGraphicsPreferences,
} from '../store/graphics-preferences-store';
import { PresetCard } from './PresetCard';

const PHASES: readonly AnimationPhase[] = ['in', 'out', 'emphasis'];

function formatSeconds(value: number): string {
  return `${value.toFixed(2).replace('.', ',')} s`;
}

/**
 * The « Animations » tab: presets by category (entrance, exit, emphasis) previewed on hover and
 * applied to the selected items in one click. Applying becomes ordinary keyframes the core
 * computes (`applyAnimationPreset`), editable afterwards.
 */
export function AnimationsPanel() {
  const { t } = useTranslation();
  const presets = useAnimationPresets();
  const applier = useApplyPreset();
  const preferences = usePersistedGraphicsPreferences();
  const custom = preferences.animationSeconds !== null;
  return (
    <div className="flex h-full flex-col">
      <div className="flex items-center gap-2 border-b border-border px-2 py-1.5 text-xs text-text-muted">
        <Switch
          checked={custom}
          aria-label={t('editor.animations.customDuration')}
          onCheckedChange={(checked) => {
            setGraphicsPreferences({ animationSeconds: checked ? 0.5 : null });
          }}
        />
        <span>{t('editor.animations.customDuration')}</span>
        {custom && (
          <ScrubInput
            size="xs"
            className="ml-auto w-20"
            label={t('editor.animations.duration')}
            value={preferences.animationSeconds}
            min={ANIMATION_SECONDS_RANGE.min}
            max={ANIMATION_SECONDS_RANGE.max}
            step={0.05}
            format={formatSeconds}
            onChange={(animationSeconds) => {
              setGraphicsPreferences({ animationSeconds });
            }}
          />
        )}
      </div>
      {applier.items.length === 0 && (
        <p className="px-2 py-1.5 text-xs text-text-subtle">{t('editor.animations.selectHint')}</p>
      )}
      {presets.length === 0 ? (
        <EmptyState icon={Sparkles} title={t('editor.animations.loading')} />
      ) : (
        <Tabs defaultValue="in" className="flex min-h-0 flex-1 flex-col">
          <TabsList className="mx-2 mt-1.5">
            {PHASES.map((phase) => (
              <TabsTrigger key={phase} value={phase}>
                {t(`editor.animations.phases.${phase}`)}
              </TabsTrigger>
            ))}
          </TabsList>
          {PHASES.map((phase) => (
            <TabsContent key={phase} value={phase} className="min-h-0 flex-1">
              <ScrollArea className="h-full">
                <div className="grid grid-cols-[repeat(auto-fill,minmax(7rem,1fr))] gap-2 p-2">
                  {presets
                    .filter((preset) => preset.phases.includes(phase))
                    .map((preset) => (
                      <PresetCard
                        key={preset.id}
                        preset={preset}
                        phase={phase}
                        disabled={!applier.applicable(preset)}
                        onApply={() => {
                          applier.apply(preset, phase);
                        }}
                      />
                    ))}
                </div>
              </ScrollArea>
            </TabsContent>
          ))}
        </Tabs>
      )}
    </div>
  );
}
