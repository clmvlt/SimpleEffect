import {
  Button,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
  Icon,
  StatusDot,
} from '@se/ui';
import { Palette } from '@se/ui/icons';
import { useTranslation } from 'react-i18next';

import { useProjectSelector } from '../../../shared/project-store';
import { useBrandKitActions } from '../hooks/use-brand-kit-actions';
import { projectKitStatus } from '../services/kit-status';
import { useBrandKits } from '../store/brand-kits-store';

/** Radio value of « Aucun ». */
const NO_KIT = 'none';

/**
 * The editor header's « Brand kit » menu (prompt 25): the project's kit (name on the button, a
 * dot when the library holds a newer version), every kit of the library to choose from or none,
 * the update when one is available (proposed, never applied by itself), « Ajouter à mes brand
 * kits » for a kit made on another machine, and « Gérer les brand kits… » (the settings).
 */
export function BrandKitButton({
  projectId,
  onManage,
}: {
  readonly projectId: string;
  /** Opens the settings' « Brand kits » section. */
  readonly onManage: () => void;
}) {
  const { t } = useTranslation();
  const copy = useProjectSelector((state) => state.mirror?.project.brandKit ?? null);
  const kits = useBrandKits();
  const actions = useBrandKitActions();
  const status = projectKitStatus(copy, kits);
  const selected = status.kind === 'linked' ? status.kit.id : status.kind === 'none' ? NO_KIT : '';
  const newer = status.kind === 'linked' ? status.newer : null;
  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button variant="ghost" size="sm" aria-label={t('brandKit.menu.label')}>
          <Icon icon={Palette} size="sm" />
          <span className="max-w-40 truncate">{copy?.name ?? t('brandKit.menu.none')}</span>
          {newer !== null && <StatusDot tone="warning" />}
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end" className="w-64">
        <DropdownMenuLabel>{t('brandKit.menu.kits')}</DropdownMenuLabel>
        <DropdownMenuRadioGroup
          value={selected}
          onValueChange={(value) => {
            const kit = kits?.find((candidate) => candidate.id === value) ?? null;
            if (value === NO_KIT || kit !== null) void actions.setProjectKit(projectId, kit);
          }}
        >
          <DropdownMenuRadioItem value={NO_KIT}>{t('brandKit.menu.noKit')}</DropdownMenuRadioItem>
          {(kits ?? []).map((kit) => (
            <DropdownMenuRadioItem key={kit.id} value={kit.id}>
              {kit.name}
            </DropdownMenuRadioItem>
          ))}
        </DropdownMenuRadioGroup>
        {newer !== null && (
          <>
            <DropdownMenuSeparator />
            <DropdownMenuItem
              onSelect={() => {
                void actions.setProjectKit(projectId, newer);
              }}
            >
              {t('brandKit.menu.update', { version: newer.version })}
            </DropdownMenuItem>
          </>
        )}
        {status.kind === 'orphan' && (
          <>
            <DropdownMenuSeparator />
            <p className="px-2 py-1 text-2xs text-text-muted">{t('brandKit.menu.notInLibrary')}</p>
            <DropdownMenuItem
              onSelect={() => {
                void actions.addProjectKit(projectId);
              }}
            >
              {t('brandKit.menu.addToLibrary')}
            </DropdownMenuItem>
          </>
        )}
        <DropdownMenuSeparator />
        <DropdownMenuItem onSelect={onManage}>{t('brandKit.menu.manage')}</DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
