import {
  DropdownMenu,
  DropdownMenuCheckboxItem,
  DropdownMenuContent,
  DropdownMenuLabel,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
  ToolbarButton,
  Tooltip,
} from '@se/ui';
import { Scan } from '@se/ui/icons';
import { SAFE_AREA_PLATFORMS } from '@se/engine/safe-areas';
import { useTranslation } from 'react-i18next';

import { hasGuides } from '../services/frame-guides';
import { setSequenceGuides, type ViewerGuides } from '../store/viewer-store';

const NONE = 'none';

/** Props of {@link GuidesMenu}. */
export interface GuidesMenuProps {
  readonly sequenceId: string;
  readonly guides: ViewerGuides;
}

/**
 * The guides of the active sequence (remembered per sequence): a platform's safe areas (TikTok,
 * Instagram Reels, YouTube Shorts, YouTube 16:9), the rule-of-thirds grid and the centre cross.
 */
export function GuidesMenu({ sequenceId, guides }: GuidesMenuProps) {
  const { t } = useTranslation();
  return (
    <DropdownMenu>
      <Tooltip content={t('editor.viewer.guides.label')}>
        <DropdownMenuTrigger asChild>
          <ToolbarButton
            icon={Scan}
            label={t('editor.viewer.guides.label')}
            pressed={hasGuides(guides)}
          />
        </DropdownMenuTrigger>
      </Tooltip>
      <DropdownMenuContent align="end">
        <DropdownMenuLabel>{t('editor.viewer.guides.safeAreas')}</DropdownMenuLabel>
        <DropdownMenuRadioGroup
          value={guides.safeArea ?? NONE}
          onValueChange={(value) => {
            const platform = SAFE_AREA_PLATFORMS.find((candidate) => candidate.id === value);
            setSequenceGuides(sequenceId, { safeArea: platform?.id ?? null });
          }}
        >
          <DropdownMenuRadioItem value={NONE}>
            {t('editor.viewer.guides.none')}
          </DropdownMenuRadioItem>
          {SAFE_AREA_PLATFORMS.map((platform) => (
            <DropdownMenuRadioItem key={platform.id} value={platform.id}>
              {platform.name}
            </DropdownMenuRadioItem>
          ))}
        </DropdownMenuRadioGroup>
        <DropdownMenuSeparator />
        <DropdownMenuCheckboxItem
          checked={guides.thirds}
          onCheckedChange={(checked) => {
            setSequenceGuides(sequenceId, { thirds: checked });
          }}
        >
          {t('editor.viewer.guides.thirds')}
        </DropdownMenuCheckboxItem>
        <DropdownMenuCheckboxItem
          checked={guides.centre}
          onCheckedChange={(checked) => {
            setSequenceGuides(sequenceId, { centre: checked });
          }}
        >
          {t('editor.viewer.guides.centre')}
        </DropdownMenuCheckboxItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
