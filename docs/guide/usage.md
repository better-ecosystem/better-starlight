# Usage

1. To run use the command:

``` sh
starlight
```

## CLI Options

``` sh
// shows help message
starlight -h

// shows current version
starlight -V

// shows Debug logs
starlight -d, --debug

// start with command runner mode
starlight -r, --run

// start with web search mode
starlight -w, --web

```

## Modes

### Default

`starlight` will work as a appliaction launcher by default.

### Command runner

use `r:` or `run:` in the search entry to switch to command runner.

### Web search

use `w:` or `web:` in the search entry to switch to web search mode.

### Unit converter

Example:

``` sh
10 gb to mb

1020 mb
```

#### Available units

- Distance
- Storage Sizes
- Temperature

### Available search engines

- DuckDuckgo (default)
- Google
- Youtube
- StackOverflow
