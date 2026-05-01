.PHONY: verify-ci verify-fast

verify-ci:
	./scripts/verify-ci.sh --ci

verify-fast:
	./scripts/verify-ci.sh --fast
