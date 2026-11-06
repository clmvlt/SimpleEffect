import type { Meta, StoryObj } from '@storybook/react-vite';
import { useEffect } from 'react';

import { Button } from '../button';
import { Toaster, toast } from './toast';

const meta = {
  title: 'Primitives/Toast',
  component: Toaster,
  args: { label: 'Notifications', closeLabel: 'Fermer la notification' },
} satisfies Meta<typeof Toaster>;

export default meta;
type Story = StoryObj<typeof meta>;

/** Each button sends one kind of toast. */
export const Default: Story = {
  render: (args) => (
    <div className="flex flex-wrap gap-2">
      <Button onClick={() => toast('Projet enregistré')}>Neutre</Button>
      <Button
        onClick={() =>
          toast.success('Export terminé', { description: 'interview.mp4 · 1 min 12 s' })
        }
      >
        Succès
      </Button>
      <Button onClick={() => toast.warning('Espace disque faible')}>Avertissement</Button>
      <Button
        onClick={() =>
          toast.error('Échec de l’import', { description: 'Codec non pris en charge : ProRes RAW' })
        }
      >
        Erreur
      </Button>
      <Button onClick={() => toast.info('Proxy en cours de création')}>Info</Button>
      <Button
        onClick={() =>
          toast('Clip supprimé', { action: { label: 'Annuler', onClick: () => undefined } })
        }
      >
        Avec action
      </Button>
      <Toaster {...args} />
    </div>
  ),
};

function ToastGallery() {
  useEffect(() => {
    const options = { duration: Number.POSITIVE_INFINITY };
    const ids = [
      toast.success('Export terminé', { ...options, description: 'interview.mp4 · 1 min 12 s' }),
      toast.error('Échec de l’import', { ...options, description: 'Codec non pris en charge' }),
      toast.warning('Espace disque faible', options),
      toast.info('Proxy en cours de création', options),
      toast.loading('Transcription…', options),
    ];
    return () => {
      ids.forEach((id) => toast.dismiss(id));
    };
  }, []);
  return null;
}

/** Every kind of toast at once, kept on screen to review them (expanded stack). */
export const AllKinds: Story = {
  render: (args) => (
    <>
      <ToastGallery />
      <Toaster {...args} />
    </>
  ),
};
