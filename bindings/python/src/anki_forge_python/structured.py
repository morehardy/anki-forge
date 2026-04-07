from __future__ import annotations

import json

from .contracts import ContractValidationError, validate_contract_payload
from .errors import ProtocolParseError, RuntimeInvocationError
from .helpers import helper_view
from .raw import run_raw


def run_structured(command: str, request: dict, **runtime_kwargs) -> dict:
    raw = run_raw(command, request, **runtime_kwargs)

    if raw.exit_status != 0:
        raise RuntimeInvocationError(
            f"{command} exited with status {raw.exit_status}",
            command=command,
            exit_status=raw.exit_status,
            stdout=raw.stdout,
            stderr=raw.stderr,
            resolved_runtime=raw.resolved_runtime,
            failure_phase="process-exit",
        )

    try:
        parsed = json.loads(raw.stdout)
    except json.JSONDecodeError as error:
        raise ProtocolParseError(
            str(error),
            command=command,
            exit_status=raw.exit_status,
            stdout=raw.stdout,
            stderr=raw.stderr,
            resolved_runtime=raw.resolved_runtime,
            parse_phase="json",
        ) from error

    try:
        validate_contract_payload(command, parsed)
    except ContractValidationError as error:
        raise ProtocolParseError(
            str(error),
            command=command,
            exit_status=raw.exit_status,
            stdout=raw.stdout,
            stderr=raw.stderr,
            resolved_runtime=raw.resolved_runtime,
            parse_phase=error.parse_phase,
        ) from error

    result = dict(parsed)
    result["resolved_runtime"] = raw.resolved_runtime
    result["raw_command"] = {
        "command": raw.command,
        "argv": raw.argv,
        "exit_status": raw.exit_status,
    }
    result["helper"] = helper_view(command, parsed, request)
    return result


def normalize(request: dict, **runtime_kwargs) -> dict:
    return run_structured("normalize", request, **runtime_kwargs)


def build(request: dict, **runtime_kwargs) -> dict:
    return run_structured("build", request, **runtime_kwargs)


def inspect(request: dict, **runtime_kwargs) -> dict:
    return run_structured("inspect", request, **runtime_kwargs)


def diff(request: dict, **runtime_kwargs) -> dict:
    return run_structured("diff", request, **runtime_kwargs)
