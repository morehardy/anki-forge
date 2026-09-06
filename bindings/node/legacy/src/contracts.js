function fail(parsePhase, message) {
  const error = new Error(message);
  error.parsePhase = parsePhase;
  throw error;
}

const OBSERVATION_MODEL_VERSIONS = ['phase3-inspect-v1', 'phase3-inspect-v2'];

const CONTRACT_RULES = {
  normalize: {
    kind: 'normalization-result',
    required: ['kind', 'result_status', 'tool_contract_version', 'diagnostics'],
    versionFields: [['tool_contract_version', 'phase2-v1']],
  },
  build: {
    kind: 'package-build-result',
    required: [
      'kind',
      'result_status',
      'tool_contract_version',
      'writer_policy_ref',
      'build_context_ref',
      'diagnostics',
    ],
    versionFields: [['tool_contract_version', 'phase3-v1']],
  },
  'product-build': {
    kind: 'anki-forge-build-report',
    required: [
      'kind',
      'schema_version',
      'status',
      'comparison',
      'counts',
      'diagnostics',
      'policy',
    ],
    versionFields: [['schema_version', 'phase4-build-report-v2']],
  },
  inspect: {
    kind: 'inspect-report',
    required: [
      'kind',
      'observation_model_version',
      'source_kind',
      'source_ref',
      'artifact_fingerprint',
      'observation_status',
      'missing_domains',
      'degradation_reasons',
      'observations',
    ],
    versionFields: [['observation_model_version', OBSERVATION_MODEL_VERSIONS]],
  },
  diff: {
    kind: 'diff-report',
    required: [
      'kind',
      'comparison_status',
      'left_fingerprint',
      'right_fingerprint',
      'left_observation_model_version',
      'right_observation_model_version',
      'summary',
      'uncompared_domains',
      'comparison_limitations',
      'changes',
    ],
    versionFields: [
      ['left_observation_model_version', OBSERVATION_MODEL_VERSIONS],
      ['right_observation_model_version', OBSERVATION_MODEL_VERSIONS],
    ],
  },
};

export function validateContractPayload(command, payload) {
  const rules = CONTRACT_RULES[command];
  if (!payload || typeof payload !== 'object' || Array.isArray(payload)) {
    fail('contract-shape', `${command} contract payload must be an object`);
  }
  if (payload.kind !== rules.kind) {
    fail('contract-shape', `${command} contract kind must be ${rules.kind}`);
  }
  for (const field of rules.required) {
    if (!(field in payload)) {
      fail('contract-shape', `${command} contract payload missing required field ${field}`);
    }
  }
  for (const [field, expected] of rules.versionFields) {
    const supported = Array.isArray(expected) ? expected : [expected];
    if (!supported.includes(payload[field])) {
      fail('contract-version', `${command} contract field ${field} must be ${supported.join(' or ')}`);
    }
  }
  const object = value => value !== null && typeof value === 'object' && !Array.isArray(value);
  const strings = value => Array.isArray(value) && value.every(item => typeof item === 'string');
  const requireShape = (condition, field) => { if (!condition) fail('contract-shape', `${command} contract field ${field} has an invalid shape`); };
  const enumField = (field, values) => requireShape(values.includes(payload[field]), field);
  for (const field of ['source_kind', 'source_ref', 'artifact_fingerprint', 'writer_policy_ref', 'build_context_ref', 'left_fingerprint', 'right_fingerprint', 'summary']) {
    if (field in payload) requireShape(typeof payload[field] === 'string', field);
  }
  if ('result_status' in payload) enumField('result_status', ['success', 'invalid', 'error']);
  if ('diagnostics' in payload) {
    const diagnostics = command === 'product-build' ? payload.diagnostics : payload.diagnostics?.items;
    requireShape(Array.isArray(diagnostics), 'diagnostics');
    for (const item of diagnostics) requireShape(object(item) && typeof item.code === 'string' && typeof item.message === 'string'
      && ['info', 'warning', 'error'].includes(item.severity), 'diagnostics item');
  }
  if (command === 'product-build') {
    enumField('status', ['success', 'blocked', 'invalid', 'error']);
    enumField('comparison', ['not_requested', 'complete', 'partial', 'unavailable']);
    requireShape(object(payload.counts) && ['notes', 'cards', 'media'].every(key => Number.isSafeInteger(payload.counts[key]) && payload.counts[key] >= 0), 'counts');
    requireShape(object(payload.policy), 'policy');
    if ('artifact' in payload) requireShape(payload.artifact === null || (object(payload.artifact) && typeof payload.artifact.path === 'string'), 'artifact');
  }
  if (command === 'inspect') {
    enumField('observation_status', ['complete', 'degraded', 'unavailable']);
    requireShape(strings(payload.missing_domains) && strings(payload.degradation_reasons), 'observation limitations');
    requireShape(object(payload.observations) && Object.values(payload.observations).every(Array.isArray), 'observations');
  }
  if (command === 'diff') {
    enumField('comparison_status', ['complete', 'partial', 'unavailable']);
    requireShape(strings(payload.uncompared_domains) && strings(payload.comparison_limitations), 'comparison limitations');
    requireShape(Array.isArray(payload.changes) && payload.changes.every(object), 'changes');
  }
}
