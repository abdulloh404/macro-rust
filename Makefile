.PHONY: run run-log record

run:
	SKILLBOT_TRACE_INPUT=0 SKILLBOT_TRACE_MANUAL=0 cargo run --locked

run-log:
	@log=$$(mktemp --suffix=.log ./logs/input-timing.XXXXXX) && \
	printf 'Macro timing log: %s\n' "$$log" && \
	SKILLBOT_TRACE_INPUT=1 SKILLBOT_TRACE_MANUAL=0 cargo run --locked 2>"$$log"

record:
	@log=$$(mktemp --suffix=.log ./logs/manual-input-timing.XXXXXX) && \
	printf 'Manual timing log: %s\n' "$$log" && \
	SKILLBOT_TRACE_INPUT=0 SKILLBOT_TRACE_MANUAL=1 cargo run --locked 2>"$$log"
