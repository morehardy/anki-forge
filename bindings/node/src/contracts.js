function fail(parsePhase, message) {
  const error = new Error(message);
  error.parsePhase = parsePhase;
  throw error;
}

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
    versionFields: [['observation_model_version', 'phase3-inspect-v1']],
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
      ['left_observation_model_version', 'phase3-inspect-v1'],
      ['right_observation_model_version', 'phase3-inspect-v1'],
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
    if (payload[field] !== expected) {
      fail('contract-version', `${command} contract field ${field} must be ${expected}`);
    }
  }
}
