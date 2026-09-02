# Dev tools: sitting in a run and interrogating it

**Status (2026-09-02): accepted, DT1 queued.** Both §4 decisions ruled. DT1
dispatches once PE3b lands, because both touch the genet host's input and
lanes and one working tree holds one agent at a time.

## 0. Objective

Mark, 2026-09-01: "We need dev tools to evaluate the game." The base is to sit
in a running game and interrogate it: pause, step, speed, follow any critter,
read its body and flows, force an event, end the epoch now. Watching the whole
ecology on one panel and comparing runs are data, and where those panes live
is not decided, so they are out of scope here.

## 1. What exists (verified 2026-09-01)

- **mesocosm-genet.** Play controls only: WASD, E or Space, Q, C, arrows,
  Escape (`input.rs:144-154`, `app.rs:444`). Headless flags in `main.rs`:
  `--frames`, `--capture`, `--trace`, `--receipt`, `--replay`,
  `--record-demo`, `--auto-eat`, `--slab`, `--seed`. No pause, step, speed,
  inspect, or force.
- **mesocosm-runtime.** `Runtime::advance` runs what the clock authorises and
  `Runtime::step(n)` ignores the clock (`runtime.rs:103`, `:132`), so time
  under the hand is already separable from wall time. A checkpoint holds the
  world without banking time. Readings a lane can draw from: `readings()`,
  `trend()`, `windows()`, `history()`, `last_outcomes()`, `end_epoch()`.
- **Chrome.** Three lanes: the painted HUD (textless by lane discipline), the
  vitals panel, the succession panel. Words go through cambium, per the views
  founding plan §6 as amended 2026-08-29.
- **genet.** `genet-probe` has `Automatable` and `Driveable` and a text
  `Scenario` with the verbs act, click, settle, wait, assert, log, capture
  (`components/genet-probe/scenario.rs:328-336`). `workbench` has `TileTree`
  and `TileEvent` for split-and-tab layouts (`components/workbench/lib.rs`).
  cambium has `CommandItem` (`command_surface.rs:16`), `TreeItem`
  (`disclosure.rs:330`) and sprigging's `GridSpec` (`grid.rs:30`).
- **isometry.** A side panel with edit modes and undo/redo, and an
  `overlay_panel` widget. No inspector, no console.
- **Nowhere in the stack:** pause/step/speed, free follow, click-to-inspect,
  spawn or kill, force-an-event, value editing, or a console.

## 2. Principles

1. **Dev tools are ordinary chrome in the cambium lane.** One more lane, the
   dev lane, built from cambium widgets. Nothing textual enters the paint lane.
2. **Two kinds of dev action, kept apart.** A host-only action (pause, step,
   speed, follow, inspect) never touches the world and lives outside the
   snapshot, so a run paused a hundred times hashes like one never paused. A
   world-changing action (end the epoch, force a birth, kill, place matter)
   is an `Intent` in the trace, so replay reproduces it and the hash stays
   honest. There is no third kind.
3. **The lane invents no readings.** Every value shown is read from `Runtime`
   or `World`. If a fact is not readable, the gap is filled in core with a
   test, not papered over in the panel.
4. **Consolidate into the stack, and grow it when needed.** The bespoke replay
   and demo harness folds toward genet-probe rather than growing. Layout,
   when more than one pane is wanted, uses workbench. No hand-rolled panel
   code where cambium has the widget. Where the stack lacks a piece (a
   time-control strip, an inspector tree over readings), build it as a stack
   component in cambium or genet-probe's own idiom, not as mesocosm-local
   code, so the next game finds one implementation. Ruled by Mark
   2026-09-02: broadening the stack is fine; duplicated or irreconcilable
   runs at the same problem are what the rule forbids.
5. **A played receipt tells the truth.** A run that used a world-changing dev
   action is labelled as such in its receipt, so a playtest cannot be quietly
   assisted.

## 3. Phases

### DT1. Time under the hand

Pause, single step, step N, and a speed multiplier on the clock, host-only,
with the current state shown in the dev lane.

**Done when:** a paused run advances exactly the steps asked; a trace played
with pauses and speed changes produces the same hash as the same trace played
straight; the dev lane shows paused, stepping, or the multiplier; the flag
that enables the lane is recorded in the receipt.

### DT2. Follow and inspect

Follow any critter, by roster cycling or by selecting it in the section, and
read it: parts and roles, sites and cells, its accounts from the flow record,
its discoveries, and its species' current program revision.

**Done when:** following a critter that is not under control does not move
control; every field shown comes from a core query; a test renders the panel
for a fixture critter and compares it against the readings directly; a
critter that dies while followed is reported, not silently dropped.

### DT3. Force and end

World-changing dev intents: end the epoch now (this is PE3's player-triggered
epoch rule, ruled a dev tool on 2026-09-01), force a birth from the followed
critter, kill the followed critter, place matter at a cell.

**Done when:** each is an `Intent` in the trace; replay reproduces it and a
falsified hash still exits 1; matter is conserved through every one (a kill
leaves a corpse, placed matter enters through a named dev source so the flow
record still reconciles); the receipt counts dev intents and labels the run.
The epoch half waits on PE3's rule enum; the rest does not.

### DT4. Harness consolidation

Express `--replay`, `--record-demo` and `--auto-eat` through genet-probe's
`Driveable` and `Scenario`, and drive the existing fixture as a scenario.

**Done when:** the `ps1_played` fixture replays through the scenario driver at
the recorded hash; the bespoke code it replaces is deleted and the deletion is
listed; the headed-verify home under `Code/testing/mesocosm` still receives
receipts and captures.

**Order.** DT1, then DT2, then DT3. DT4 is independent and can go beside any
of them. Each phase is one agent round on a non-Fable model.

## 4. Decisions, ruled by Mark 2026-09-02

1. **How the lane is enabled.** A `--dev` runtime flag. One binary; the
   receipt records the flag.
2. **Where the lane sits.** Look to cambium, not to isometry. Isometry's side
   panel is app code that happens to sit on cambium, a precedent for "docked"
   and not a component to reuse. The dev lane is a workbench tile holding
   cambium widgets, and its placement is whatever workbench's split-and-tab
   tree gives; no hand-rolled panel frame, no isometry code copied across.

## 5. Stop rules

- No world mutation outside the trace. No second authority over the world.
- No text in the paint lane.
- No reading computed in the lane that core cannot answer.
- No dev-only physics or dev-only rules: a forced birth is the ordinary birth.
- No new harness where genet-probe already has the verb.

## Findings

- **2026-09-01:** the stack has automation (genet-probe), layout (workbench)
  and widgets (cambium) for a dev lane, and no run-time dev tools at all in
  any game. Mesocosm's replay harness is the only headless evaluation tooling
  and it is app-local.

## Progress

- **2026-09-01:** assessment written from a read-only inventory of mesocosm,
  isometry, genet, cambium and mere, verified against file paths above. No
  code dispatched.
