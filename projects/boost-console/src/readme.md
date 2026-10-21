# `boost-console`

## `fmt` module owns

- color, paint, and style values
- stream-aware foreground/background wrappers
- ANSI formatting helpers

`fmt` depends only on the standard library. Enable with `boost-console` feature `fmt` (no `console` feature required).

## `console` feature owns

- styled and plain `LogEvent` rendering
- stderr sinks
- installing the default human-visible global subscriber

## Does not own

- `LogEvent`, spans, filters, or generic sinks (`boost-log`)
- TTY detection, ANSI capability, or device handles (`boost-terminal`)
