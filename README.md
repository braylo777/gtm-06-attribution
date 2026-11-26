# gtm-06-attribution

**GTM Engineering — Stage 6 of 12: Multi-Touch Attribution & RevOps**

Prove which spend makes revenue: five attribution models on the same journeys with ROAS per channel, funnel leakage priced in dollars against benchmarks, and cohort retention triangles.

**Live demo:** https://gtm-06-attribution.braylo777.workers.dev — the Rust core runs in your browser as WebAssembly. No server, no tracking.

## Why this matters

Every budget meeting asks the same question: what is working? Last-touch gives all credit to the final email; first-touch gives it all to the first ad. The honest answer is a range. Show the range across models, and the CFO sees where the model choice changes the decision. Then price the funnel leaks and the retention curve in dollars, because that is the language the decision is made in.

## What it does

- First-touch, last-touch, linear, time-decay (half-life) and U-shaped 40/20/40 models; each gives out exactly 100% of value
- Touches after the conversion day are ignored; open opportunities are skipped
- Per-channel revenue, share and ROAS per model, plus each channel's share range across models
- Funnel: stage-to-stage rate, benchmark gap, extra wins at benchmark (carried through later stages), revenue gap, biggest leak
- Cohorts: logo and revenue retention triangles by signup month, size-weighted average curve

## What it does not do (yet)

- No data-driven (Shapley / Markov) model yet: the five rule-based models are the baseline
- No SQL engine: inputs are pre-joined journeys and stage counts as JSON (the SQL that builds them lives in the warehouse)
- No significance testing on funnel differences

## Operations

Every operation takes JSON in and gives JSON out. The CLI, the tests and the demo page all use the same entry point.

| Operation | What it does |
|---|---|
| `attribute` | Credit 24 synthetic B2B opportunities to channels with 5 models. Compare each channel's share range. |
| `funnel_leakage` | Find the funnel step that costs the most revenue against benchmark conversion rates. |
| `cohort_retention` | Logo and revenue retention by signup month. Revenue retention can pass 100% when customers expand. |

## Run it

You need Rust (stable). No Node, no npm.

```bash
git clone https://github.com/braylo777/gtm-06-attribution
cd gtm-06-attribution
cargo test
cargo run -- ops
```

Try the first demo example from the command line:

```bash
python3 -c 'import json;print(json.dumps(json.load(open("web/examples.json"))["ops"][0]["input"]))' | cargo run -q -- attribute
```

## Build and deploy the demo

```bash
./build.sh    # tests, then compiles the core to web/app.wasm
./deploy.sh   # serves web/ as a Cloudflare Workers static-assets site (needs a Cloudflare login)
```

`web/` is a plain static site (`index.html`, `app.js`, `app.wasm`). It has no build step, so any static host works.

## Layout

```
src/lib.rs        operation list and router
src/api.rs        JSON entry point + the 3 WASM exports (alloc, dealloc, run)
src/main.rs       CLI
src/*.rs          one module per capability, each with unit tests
tests/            integration tests (every demo example must pass)
web/              demo page; examples.json drives both the page and this README
```

## Design choices

- **Rust, few dependencies.** Only `serde_json`, pinned to exact versions with a committed `Cargo.lock`.
- **Deterministic.** Time, randomness and keys are inputs, so every result can be replayed in a test or an audit.
- **Fail closed.** Unknown input or missing fields give an error, never a silent default.

## Part of a series

This is one of 12 repos that work through a public Go-To-Market (GTM) Engineering roadmap: [12 stages by Suraj Sharma (@suraj_sharma14)](https://x.com/suraj_sharma14/status/2125552583452270692). The roadmap is the inspiration; the code here is original. All 12: [github.com/braylo777?q=gtm-](https://github.com/braylo777?tab=repositories&q=gtm-).

## Data and privacy

Every example uses synthetic data. Nothing calls a paid API, scrapes a site, identifies a real visitor or sends an email.

## License

MIT — see [LICENSE](LICENSE). Built by [Braylo SI](https://braylo.si).
