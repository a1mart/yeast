import { useEffect, useState } from 'react';

export function useAccumDist() {
  const [fn, setFn] = useState<((candles: any[]) => number[]) | null>(null);

  useEffect(() => {
    async function init() {
      const wasm = await import('@/wasm/stox_wasm.js'); // served from /public/wasm
      await wasm.default();
      if (wasm.accum_dist_line) {
        setFn(() => (candles) => {
          const json = JSON.stringify(candles);
          return wasm.accum_dist_line(json);
        });
      }
    }
    init();
  }, []);

  return fn;
}
