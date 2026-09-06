import {
  BindingProtocolError,
  BuildError,
  ProjectAddError,
  MediaError,
  ProductNoteError,
  TemplateBundleError,
  ProjectDiffError,
  DeckError,
} from '../errors';
import { BuildReport, ProjectDiffReport, diagnostics } from '../report';

export function outcome(input: string): Record<string, unknown> {
  let envelope;
  try {
    envelope = JSON.parse(input);
  } catch {
    throw new BindingProtocolError('Invalid native result JSON');
  }
  if (!envelope || typeof envelope.ok !== 'boolean')
    throw new BindingProtocolError('Invalid native result envelope');
  if (envelope.ok) return envelope.value;
  const error = envelope.error;
  if (typeof error?.code !== 'string' || typeof error.message !== 'string')
    throw new BindingProtocolError('Invalid native error');
  if (error.kind === 'add') throw new ProjectAddError(diagnostics([error.details?.diagnostic])[0]);
  if (error.kind === 'build')
    throw new BuildError(
      error.message,
      error.code,
      new BuildReport(error.details?.report, error.details?.pretty),
      error.details?.cause,
    );
  if (error.kind === 'media') throw new MediaError(error.message, error.code);
  if (error.kind === 'diff')
    throw new ProjectDiffError(
      error.message,
      error.code,
      new ProjectDiffReport(error.details?.report),
      error.details?.cause,
    );
  if (error.kind === 'note') throw new ProductNoteError(error.message, error.code);
  if (error.kind === 'deck') throw new DeckError(error.message, error.code);
  if (error.kind === 'template_bundle')
    throw new TemplateBundleError(
      error.message,
      error.code,
      error.details?.path,
      error.details?.byteOffset,
    );
  throw new BindingProtocolError('Unknown native error');
}
