# Dev tools: sitting in a run and interrogating it

**Status (2026-09-02): DT1 and DT2 landed.** Both §4 decisions ruled and built.
DT3 (force and end) is next; DT4 is independent and can go beside it.

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
- **2026-09-02 (DT1):** `workbench` (genet, `components/workbench`) is a
  host-owned tree and reducer only — there is no genet-side surface that
  renders a `TileTree` to pixels, draws a tab strip, or turns a drag into a
  `TileEvent`. DT1 never needed one (its tree never holds more than the dev
  lane's one tile), but DT2's second dev pane is what would. See
  `mesocosm-genet/src/dev.rs`'s module docs.
- **2026-09-02 (DT2), three gaps found and each handled differently.**
  1. **Two core readings were genuinely missing** and were added there with
     tests, per principle 3. `flow::Accounts`
     (`mesocosm-core/src/flow/accounts.rs`) reduces one *body's* income, rent
     and outflow off the flow record, making exactly the split `Score` already
     makes for a *line* — written once so the two cannot come to disagree.
     `History::ending` answers when and which way a creature stopped living,
     off the record's own `Died`/`Returned` event and its envelope's tick. The
     driver holds the per-body window (`Runtime::watch`/`::accounts`) because
     it already drains the stream the ecology's own windows reduce; a world
     buffers one tick and the host never sees it.
  2. **A world's discoveries are world-scoped, not per line.** `World::observe`
     records against whoever is under the hand, so a world holds *one*
     discovery list and it is the played line's. The tile shows that list and
     says so; a per-line discovery ledger is reported here rather than
     invented in the panel. It is what DT3 or a later phase would need before
     the row can honestly be labelled "this critter's line".
  3. **Click-to-select in the section needs picking machinery the host does
     not have.** The pieces exist elsewhere — `mesocosm-runtime`'s
     `TactileWorld` raycasts ground and critter capsules, and
     `mesocosm-lens`'s `t1_picking` example judges a sweep on the CPU — but
     `mesocosm-genet`'s `Section` holds no hit data and builds no
     `GroundVoxelProfile`, so a click would need a tactile world synchronized
     with every carve, every roster body presented under its organism id each
     frame, and an unprojection through the orthographic slab. Left out, as
     the slice allows; roster cycling is the path.

## Progress

- **2026-09-01:** assessment written from a read-only inventory of mesocosm,
  isometry, genet, cambium and mere, verified against file paths above. No
  code dispatched.
- **2026-09-02 (DT1 landed):** `--dev` (off by default; recorded in the
  receipt as `dev: bool`) adds a fifth chrome lane — a cambium detail panel,
  top left, built the way `mesocosm-views::vitals` is — and five keys, live
  only while the flag is set: `P` pauses or unpauses the clock, `.` steps
  once and `,` steps `DEV_STEP_N` (10), both off the clock, and `[`/`]` move
  a five-rung speed ladder (1/4, 1/2, 1, 2, 4) that scales the elapsed
  microseconds the host passes to `Runtime::advance`. All three are host-only
  pacing: pause drops the elapsed time the same way a checkpoint hold already
  does, speed scales it before the clock ever sees it, and step calls
  `Runtime::step` directly — none of the three reaches `Runtime::queue`, so
  none can enter the trace or move a hash. `mesocosm-runtime`'s
  `pauses_speed_changes_and_manual_steps_do_not_change_the_hash` drives the
  same intents through pauses, speed changes and manual steps interleaved
  with `advance` and checks the hash against a straight run; `step`'s
  existing "N unless a checkpoint holds it, then fewer" contract is reused
  unmodified and stated explicitly now in `tests/checkpoint.rs`. Ruled §4.2:
  the dev lane's placement is a `workbench::TileTree`'s own geometry (one
  tile today; see the finding above), not a hand-rolled corner constant like
  the other three lanes'. `--dev --frames 200 --capture` was read: the panel
  reads "state / speed / tick / stepped", sits top left clear of the minimap
  (top right) and vitals (bottom left), and the whole frame is sane.
  `--replay` of the untouched `ps1_played.trace.json` fixture still exits 0
  at `081b4ba4bdc46190` with `--dev` off; a falsified hash still exits 1. The
  receipt's new `dev` field does not move the state hash. `cargo test
  --workspace`, clippy `-D warnings` (both profiles) and fmt are clean.
- **2026-09-02 (DT2 landed):** three more keys, live only under `--dev` and
  clear of everything play owns: `N` follows the next living critter in id
  order, `B` the previous, both wrapping, and `M` snaps the camera back to the
  critter under the hand. **Following moves the section's follow centre and
  nothing else** — `Host::follow` is host state beside the pan, control does
  not move, no intent is queued, and `mesocosm-genet`'s
  `following_does_not_move_control_and_queues_nothing` asserts the controlled
  id, the queue length, the trace and the state hash are all unchanged after a
  follow. The played body is still posed at full fidelity where *it* stands;
  only the slab's centre moves, so `frame` now reads a follow centre and a
  separate played position.

  The one dev tile (DT1's `TileTree` still holds exactly one) shows, for the
  followed critter and entirely from core queries: **id** and whether it is the
  controlled one (`World::controlled_id`), **species** with the registry's name
  (`Lineages::get`), **position**, the two accounts `flow::Account` names —
  **reserve** (`Organism::energy_mg`) and **substance** (`biomass_mg()`) —
  **flows** (income, rent, outflow, from the new `flow::Accounts`) with its
  **window** on a line of its own, the line's **revision**
  (`Species::program().current()`, or `founding`), the world's **discovered**
  conditions by name (`World::discoveries` through `discovery::name_of`), and
  the **parts** count plus one row per part giving its role
  (`plan::classify`), half-extent, and sites with process name and cell count
  (`BodyPhenotype::explain`). Three part rows, then `more +N parts` — the
  roster's own truncate-with-a-count, because the tile is one tile. A
  `mesocosm-views` test puts every one of those lines beside the query it came
  from.

  A followed critter that dies is **reported, then dropped**: `update_follow`
  takes `History::ending`, keeps it as a notice the tile prints
  (`critter 640 died at tick 812`), and snaps follow back to the controlled
  critter; following anybody else clears the notice. The succession lane is
  untouched — a death of the *controlled* critter is still the driver's
  checkpoint and behaves exactly as before.

  **The dock moved, and the tile still takes it off the tree.** DT1 put the
  lane in the top-left corner, which is 196 pixels tall before it reaches the
  vitals panel; an inspector with a dozen rows does not fit there. The dock is
  now the right column under the minimap, the other region no lane claims, and
  `rect_of` still walks the `TileTree` for it. §4.2 rules that the placement is
  the tree's; the corner was never a ruling.

  Two things were left out and are recorded as findings above: click-to-select
  in the section (it needs picking machinery `mesocosm-genet` does not have),
  and a per-line discovery ledger (a world's discovery list is the played
  line's, and the tile shows it as that).

  Receipts. `--dev --frames 300 --follow 640` was read at
  `Code/testing/mesocosm/dt2_inspect.png`: the tile reads `id 640`,
  `species 5`, `at -10, 22, -57`, `reserve 464 mg`, `substance 590 mg`,
  `flows in 552, rent 101, out 393`, `window 20 ticks`, `revision founding`,
  `discovered none`, `parts 33`, three part rows and `more +30 parts`; it sits
  in the right column clear of the minimap above and the vitals panel opposite,
  and the whole frame is the terrarium DT1's capture shows. A second run with
  `--follow 5` (a critter that died at tick 20) was read too and showed
  `critter 5 died at tick 20` under a tile snapped back to `id 0 (controlled)`.
  `--follow ID` is a new presentation-only flag that only says where the camera
  starts, added because a scripted trace has no dev keys in it. `--replay` of
  the untouched `ps1_played.trace.json` fixture exits 0 at `081b4ba4bdc46190`
  with explicit non-default receipt and capture paths; a falsified recorded
  hash exits 1. `cargo test --workspace`, clippy `-D warnings` (both profiles)
  and fmt are clean. `flow.rs` was split at the six-hundred-line ceiling to
  make room for `flow/accounts.rs`, and the host's follow state lives in
  `app/follow.rs` beside DT1's `app/devtime.rs` for the same reason.
