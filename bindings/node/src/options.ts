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
export function checkInspectLimits(limits: InspectLimits): void {
  options(limits, inspectKeys, 'inspectLimits');
  for (const value of Object.values(limits))
    if (value !== undefined && (!Number.isSafeInteger(value) || Number(value) < 0))
      throw new TypeError('Inspect limits must be non-negative safe integers');
}
export function defaultInspectLimits(): Readonly<Required<InspectLimits>> {
  return Object.freeze(JSON.parse(native().defaultInspectLimits()));
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
