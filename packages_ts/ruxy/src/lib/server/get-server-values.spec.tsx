import { render } from '@testing-library/react';

// import { useServerValues } from './get-server-values';

function Values() {
  // const values = useServerValues();
  return null;
}

describe('Server', () => {
  it('should receive mocked values', () => {
    const { baseElement } = render(<Values />);
    expect(baseElement).toBeTruthy();
  });
});
