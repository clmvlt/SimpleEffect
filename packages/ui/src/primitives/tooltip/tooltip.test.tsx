import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';

import { Tooltip, TooltipProvider } from './tooltip';

function renderTooltip() {
  return render(
    <TooltipProvider>
      <Tooltip content="Couper le clip" shortcut={['Ctrl', 'K']}>
        <button type="button">Couper</button>
      </Tooltip>
    </TooltipProvider>,
  );
}

describe('Tooltip', () => {
  it('is hidden until its trigger is focused', () => {
    renderTooltip();
    expect(screen.queryByRole('tooltip')).not.toBeInTheDocument();
  });

  it('appears on keyboard focus with its label and shortcut, and describes the trigger', async () => {
    const user = userEvent.setup();
    renderTooltip();

    await user.tab();
    const tooltip = await screen.findByRole('tooltip');
    expect(tooltip).toHaveTextContent('Couper le clip');
    expect(tooltip).toHaveTextContent('CtrlK');
    expect(screen.getByRole('button', { name: 'Couper' })).toHaveAccessibleDescription(
      'Couper le clipCtrlK',
    );
  });

  it('closes on Escape', async () => {
    const user = userEvent.setup();
    renderTooltip();

    await user.tab();
    await screen.findByRole('tooltip');
    await user.keyboard('{Escape}');
    expect(screen.queryByRole('tooltip')).not.toBeInTheDocument();
  });
});
