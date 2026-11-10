import { useTranslation } from 'react-i18next';

import { useVisionStatus } from '../../reframing';
import { useMachineCheck } from '../hooks/use-machine-check';
import {
  codecsCheck,
  gpuCheck,
  toolsCheck,
  visionCheck,
  type MediaCheck,
} from '../services/media-step';
import { CheckRow } from './CheckRow';

/** Props of {@link MediaStep}. */
export interface MediaStepProps {
  /** Whether the step shows (checks start the first time). */
  readonly active: boolean;
}

const ROW_LABELS = {
  tools: 'onboarding.media.tools.label',
  gpu: 'onboarding.media.gpu.label',
  codecs: 'onboarding.media.codecs.label',
  vision: 'onboarding.media.vision.label',
} as const;

/**
 * « Binaires & GPU »: ffmpeg/ffprobe and the hardware encoders, the GPU, what WebCodecs decodes
 * and encodes here, and face detection.
 */
export function MediaStep({ active }: MediaStepProps) {
  const { t } = useTranslation();
  const check = useMachineCheck(active);
  const vision = useVisionStatus();
  const rows: ReadonlyArray<readonly [keyof typeof ROW_LABELS, MediaCheck]> = [
    ['tools', toolsCheck(check.tools)],
    ['gpu', gpuCheck(check.gpu)],
    ['codecs', codecsCheck(check.codecs)],
    ['vision', visionCheck(vision)],
  ];
  return (
    <div className="flex flex-col gap-3">
      <p className="text-sm text-text-muted">{t('onboarding.media.intro')}</p>
      <ul className="flex flex-col">
        {rows.map(([id, row]) => (
          <CheckRow key={id} tone={row.tone} label={t(ROW_LABELS[id])}>
            <span className="break-words">
              {t(`onboarding.media.${row.key}`, { ...row.values })}
            </span>
          </CheckRow>
        ))}
      </ul>
    </div>
  );
}
