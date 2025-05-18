import { render } from '@testing-library/react';

import Values from './get-server-values';

describe('Server', () => {
  it('should render successfully', () => {
    const { baseElement } = render(<Values />);
    expect(baseElement).toBeTruthy();
  });
});
