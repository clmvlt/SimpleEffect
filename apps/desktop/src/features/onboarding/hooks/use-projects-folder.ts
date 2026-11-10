import { toast } from '@se/ui';
import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';

import { useCoreClient } from '../../../shared/core';
import { useDesktopBridge } from '../../../shared/desktop';
import { errorMessageKey } from '../../../shared/errors';
import { getProjectsFolder, setProjectsFolder } from '../api';

/** The folder new projects are proposed in, and how to choose another. */
export interface ProjectsFolder {
  /** `null` while loading. */
  readonly folder: string | null;
  readonly choose: () => void;
}

/** Reads the projects folder when `active`; `choose` shows the folder picker and saves the pick. */
export function useProjectsFolder(active: boolean): ProjectsFolder {
  const { t } = useTranslation();
  const client = useCoreClient();
  const bridge = useDesktopBridge();
  const [folder, setFolder] = useState<string | null>(null);
  const fail = (error: unknown) => {
    console.warn('[onboarding] projects folder', error);
    toast.error(t(errorMessageKey(error)));
  };
  useEffect(() => {
    if (!active) return;
    let current = true;
    getProjectsFolder(client)
      .then((loaded) => {
        if (current) setFolder(loaded);
      })
      .catch((error: unknown) => {
        console.warn('[onboarding] projects folder unavailable', error);
      });
    return () => {
      current = false;
    };
  }, [active, client]);
  const choose = () => {
    const pick = async () => {
      const path = await bridge.chooseFolder({
        title: t('onboarding.projects.dialogTitle'),
        buttonLabel: t('onboarding.projects.dialogButton'),
        ...(folder === null ? {} : { defaultPath: folder }),
      });
      if (path === null) return;
      setFolder((await setProjectsFolder(client, path)).parentDir);
    };
    pick().catch(fail);
  };
  return { folder, choose };
}
