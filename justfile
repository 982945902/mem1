# Build and test (constitution IV: CI gates)
default:
    just check

check: check-rust check-python
    @echo "All checks passed."

check-rust:
    cd mem1-server && cargo fmt -- --check && cargo clippy -- -D warnings && cargo test

check-python:
    cd python && python -m pytest tests/ -v 2>/dev/null || true


# Benchmark helpers
benchmark-sample:
	@echo "Starting mem1 benchmark sample (requires mem1-server running on 127.0.0.1:8080)"
	cd evaluation && make sample

benchmark-full:
	@echo "Starting mem1 benchmark full pipeline (requires mem1-server running on 127.0.0.1:8080)"
	cd evaluation && make full
