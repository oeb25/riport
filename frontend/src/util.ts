export const intersparse = <T, S>(xs: T[], y: S): (T | S)[] => {
  const out = [];
  for (const x of xs) {
    out.push(x);
    out.push(y);
  }
  if (out.length > 0) {
    return out.slice(0, -1);
  }
  return out;
};

export const replaceAt = <T>(xs: T[], i: number, x: T): T[] =>
  xs
    .slice(0, i)
    .concat(x)
    .concat(xs.slice(i + 1));

export type Mapper<T> = { [K in keyof T]: { type: K } & T[K] }[keyof T];
