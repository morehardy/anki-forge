import path from 'node:path';

export function warningCount(result) {
  const diagnostics = result.diagnostics?.items ?? [];
  return diagnostics.filter((item) => item.level === 'warning').length;
}

function artifactPathFromRef(artifactsDir, ref) {
  if (!artifactsDir || !ref) {
    return null;
  }
  const normalizedRef = ref.replace(/^artifacts\//, '');
  return path.join(artifactsDir, ...normalizedRef.split('/'));
}

export function helperView(command, result, request) {
  return {
    isInvalid: result.result_status === 'invalid',
    isDegraded: result.observation_status === 'degraded',
    isPartial: result.comparison_status === 'partial',
    warningCount: warningCount(result),
    artifactPaths:
      command === 'build'
        ? {
            stagingManifest: artifactPathFromRef(
              request.artifactsDir,
              result.staging_ref ?? null,
            ),
            apkg: artifactPathFromRef(request.artifactsDir, result.apkg_ref ?? null),
          }
        : null,
  };
}
