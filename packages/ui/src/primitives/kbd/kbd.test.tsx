import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { Kbd } from './kbd';

describe('Kbd', () => {
  it('renders a single key from its children', () => {
    const { container } = render(<Kbd>Espace</Kbd>);
    expect(screen.getByText('Espace').tagName).toBe('KBD');
    expect(container.querySelectorAll('kbd')).toHaveLength(1);
  });

  it('renders one key cap per key of a shortcut, inside a wrapping kbd', () => {
    const { container } = render(<Kbd keys={['Ctrl', 'Maj', 'K']} />);
    const wrapper = container.firstElementChild;
    expect(wrapper?.tagName).toBe('KBD');
    expect(wrapper).toHaveTextContent('CtrlMajK');
    expect(wrapper?.querySelectorAll('kbd')).toHaveLength(3);
  });
});
