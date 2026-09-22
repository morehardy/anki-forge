import type { BuildOptions, InspectLimits } from './types';
import { native } from './internal/native';
import { options, string } from './internal/validation';

export const inspectKeys = [
  'maxArchiveBytes',
  'maxEntries',
  'maxCentralDirectoryBytes',
  'maxZipEntryBytes',
  'maxZipTotalBytes',
  'maxMetaBytes',
  'maxMediaMapBytes',
  'maxCollectionBytes',
  'maxMediaBytes',
  'maxDecodedTotalBytes',
  'maxZstdWindowBytes',
];
/** @internal Preserve exact integers across JSON without a global BigInt serializer. */
export function encodeInspectLimits(limits: InspectLimits): Record<string, number | string> {
  options(limits, inspectKeys, 'inspectLimits');
  const encoded: Record<string, number | string> = {};
  for (const [key, value] of Object.entries(limits)) {
    if (value === undefined) continue;
    if (typeof value === 'bigint') {
      if (value < 0n || value > 18446744073709551615n)
        throw new TypeError('Inspect limits must fit an unsigned 64-bit integer');
      encoded[key] = value.toString();
    } else {
      if (typeof value !== 'number' || !Number.isSafeInteger(value) || value < 0)
        throw new TypeError('Inspect limits must be non-negative safe integers or u64 bigints');
      encoded[key] = value;
    }
  }
  return encoded;
}
export function defaultInspectLimits(): Readonly<Record<keyof InspectLimits, number>> {
  const limits: Record<keyof InspectLimits, number> = JSON.parse(
    native().defaultInspectLimits(),
  );
  if (
    inspectKeys.some(
      (key) =>
        !Number.isSafeInteger(limits[key as keyof InspectLimits]) ||
        limits[key as keyof InspectLimits] < 0,
    )
  )
    throw new Error('Native default inspection budgets exceed the safe number range');
  return Object.freeze(limits);
}
export function firstUpdateSafeBuild(
  identityLockfile: string,
): Pick<BuildOptions, 'identityLockfile' | 'writeIdentityLockfile' | 'updateSafety'> {
  string(identityLockfile, 'identityLockfile');
  return { identityLockfile, writeIdentityLockfile: true, updateSafety: 'strict' };
}
export function updateSafe(
  identityLockfile: string,
): Pick<BuildOptions, 'identityLockfile' | 'writeIdentityLockfile' | 'updateSafety'> {
  string(identityLockfile, 'identityLockfile');
  return { identityLockfile, updateSafety: 'strict' };
}
