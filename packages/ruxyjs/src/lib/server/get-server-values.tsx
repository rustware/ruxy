import { useState } from 'react';

export function useServerValues() {
  const [serverValues] = useState(__RUXY_DATA__.serverValues);

  // Post-routing update logic

  return serverValues;
}
