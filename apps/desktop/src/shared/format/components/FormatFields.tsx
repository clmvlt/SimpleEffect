import type { FrameRate } from '@se/protocol';
import { AspectRatioPicker } from '@se/ui';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

import { withDraftRatio, type FormatDraft } from '../format-draft';
import { resolutionsFor } from '../resolutions';
import { FrameRateSelect } from './FrameRateSelect';
import { ResolutionFields } from './ResolutionFields';

/** Props of {@link FormatFields}. */
export interface FormatFieldsProps<T extends FormatDraft> {
  readonly value: T;
  readonly onChange: (next: T) => void;
  /** The frame rates offered (`NewProjectDefaults.frameRates`). */
  readonly frameRates: readonly FrameRate[];
  /** Why the frame rate cannot change; the field is disabled when set. */
  readonly frameRateLocked?: string | undefined;
  /** Shown when the frame size is refused. */
  readonly sizeError?: string | undefined;
}

/**
 * The fields of a frame format: ratio (presets or custom), resolution (offered sizes or a custom
 * width and height) and frame rate. Choosing a ratio proposes its 1080p-class size.
 */
export function FormatFields<T extends FormatDraft>({
  value,
  onChange,
  frameRates,
  frameRateLocked,
  sizeError,
}: FormatFieldsProps<T>) {
  const { t } = useTranslation();
  const [customChosen, setCustomChosen] = useState(false);
  const resolutions = resolutionsFor(value.ratio);
  const listed = resolutions.some(
    (resolution) => resolution.width === value.width && resolution.height === value.height,
  );
  return (
    <div className="flex flex-col gap-4">
      <div className="flex flex-col gap-1.5">
        <span className="text-sm font-medium text-text">{t('format.ratio')}</span>
        <AspectRatioPicker
          value={value.ratio}
          onChange={(ratio) => {
            setCustomChosen(false);
            onChange(withDraftRatio(value, ratio));
          }}
          label={t('format.ratio')}
          customLabel={t('format.customRatio')}
          customWidthLabel={t('format.ratioWidth')}
          customHeightLabel={t('format.ratioHeight')}
        />
      </div>
      <ResolutionFields
        value={value}
        resolutions={resolutions}
        customSize={customChosen || !listed}
        onChoose={(resolution) => {
          setCustomChosen(resolution === 'custom');
          if (resolution !== 'custom') onChange({ ...value, ...resolution });
        }}
        onSize={(size) => {
          onChange({ ...value, ...size });
        }}
        error={sizeError}
      />
      <FrameRateSelect
        rates={frameRates}
        value={value.frameRate}
        disabled={frameRateLocked !== undefined}
        description={frameRateLocked}
        onChange={(frameRate) => {
          onChange({ ...value, frameRate });
        }}
      />
    </div>
  );
}
