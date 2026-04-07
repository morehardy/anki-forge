class ContractValidationError(Exception):
    def __init__(self, parse_phase: str, message: str):
        super().__init__(message)
        self.parse_phase = parse_phase


CONTRACT_RULES = {
    "normalize": {
        "kind": "normalization-result",
        "required": ["kind", "result_status", "tool_contract_version", "diagnostics"],
        "version_fields": [("tool_contract_version", "phase2-v1")],
    },
    "build": {
        "kind": "package-build-result",
        "required": [
            "kind",
            "result_status",
            "tool_contract_version",
            "writer_policy_ref",
            "build_context_ref",
            "diagnostics",
        ],
        "version_fields": [("tool_contract_version", "phase3-v1")],
    },
    "inspect": {
        "kind": "inspect-report",
        "required": [
            "kind",
            "observation_model_version",
            "source_kind",
            "source_ref",
            "artifact_fingerprint",
            "observation_status",
            "missing_domains",
            "degradation_reasons",
            "observations",
        ],
        "version_fields": [("observation_model_version", "phase3-inspect-v1")],
    },
    "diff": {
        "kind": "diff-report",
        "required": [
            "kind",
            "comparison_status",
            "left_fingerprint",
            "right_fingerprint",
            "left_observation_model_version",
            "right_observation_model_version",
            "summary",
            "uncompared_domains",
            "comparison_limitations",
            "changes",
        ],
        "version_fields": [
            ("left_observation_model_version", "phase3-inspect-v1"),
            ("right_observation_model_version", "phase3-inspect-v1"),
        ],
    },
}


def validate_contract_payload(command: str, payload: object) -> None:
    rules = CONTRACT_RULES[command]
    if not isinstance(payload, dict):
        raise ContractValidationError(
            "contract-shape", f"{command} contract payload must be an object"
        )
    if payload.get("kind") != rules["kind"]:
        raise ContractValidationError(
            "contract-shape", f"{command} contract kind must be {rules['kind']}"
        )
    for field in rules["required"]:
        if field not in payload:
            raise ContractValidationError(
                "contract-shape",
                f"{command} contract payload missing required field {field}",
            )
    for field, expected in rules["version_fields"]:
        if payload.get(field) != expected:
            raise ContractValidationError(
                "contract-version",
                f"{command} contract field {field} must be {expected}",
            )
