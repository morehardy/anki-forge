class RuntimeInvocationError(Exception):
    def __init__(
        self,
        message,
        *,
        command,
        exit_status=None,
        stdout="",
        stderr="",
        resolved_runtime=None,
        failure_phase=None,
        parse_phase=None,
    ):
        super().__init__(message)
        self.command = command
        self.exit_status = exit_status
        self.stdout = stdout
        self.stderr = stderr
        self.resolved_runtime = resolved_runtime
        self.failure_phase = failure_phase
        self.parse_phase = parse_phase


class ProtocolParseError(Exception):
    def __init__(
        self,
        message,
        *,
        command,
        exit_status=None,
        stdout="",
        stderr="",
        resolved_runtime=None,
        parse_phase="json",
    ):
        super().__init__(message)
        self.command = command
        self.exit_status = exit_status
        self.stdout = stdout
        self.stderr = stderr
        self.resolved_runtime = resolved_runtime
        self.parse_phase = parse_phase
