import type { Meta, StoryObj } from '@storybook/react-vite';

import { KeyboardShortcut } from './keyboard-shortcut';

const FRENCH_KEYS = {
  Shift: 'Maj',
  Enter: 'Entrée',
  Delete: 'Suppr',
  Escape: 'Échap',
  Space: 'Espace',
};

const meta = {
  title: 'Editor/KeyboardShortcut',
  component: KeyboardShortcut,
  args: { keys: ['Mod', 'Shift', 'Z'] },
  argTypes: { platform: { control: 'inline-radio', options: ['apple', 'other'] } },
} satisfies Meta<typeof KeyboardShortcut>;

export default meta;
type Story = StoryObj<typeof meta>;

/** Detected from the browser running Storybook. */
export const Detected: Story = {};

export const Apple: Story = { args: { platform: 'apple' } };

/** Windows and Linux, with the French key names the app passes. */
export const WindowsAndLinux: Story = { args: { platform: 'other', keyLabels: FRENCH_KEYS } };

/** A cheat sheet of common editor shortcuts on both platforms. */
export const CheatSheet: Story = {
  render: () => {
    const shortcuts = [
      { action: 'Annuler', keys: ['Mod', 'Z'] },
      { action: 'Couper au curseur', keys: ['Mod', 'K'] },
      { action: 'Supprimer et refermer', keys: ['Shift', 'Delete'] },
      { action: 'Envoyer au modèle', keys: ['Mod', 'Enter'] },
      { action: 'Image précédente', keys: ['ArrowLeft'] },
    ];
    return (
      <table className="text-xs text-text-muted">
        <thead>
          <tr className="text-left text-text">
            <th className="pr-6 pb-2 font-medium">Action</th>
            <th className="pr-6 pb-2 font-medium">macOS</th>
            <th className="pb-2 font-medium">Windows / Linux</th>
          </tr>
        </thead>
        <tbody>
          {shortcuts.map(({ action, keys }) => (
            <tr key={action}>
              <td className="py-1 pr-6">{action}</td>
              <td className="py-1 pr-6">
                <KeyboardShortcut keys={keys} platform="apple" />
              </td>
              <td className="py-1">
                <KeyboardShortcut keys={keys} platform="other" keyLabels={FRENCH_KEYS} />
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    );
  },
};
