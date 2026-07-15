# nelo v2 — Design Notes

Working design doc for the Rust rewrite. Captures the API decisions, the reasoning
behind them, and — where a decision isn't final — the current leaning and the open
question. v1 refers to the C++ implementation on the `main` branch.

## North Star

Two concrete use cases define "done." Every abstraction below should be justified by
whether it serves at least one of them:

1. **3Blue1Brown-style math video** (à la Manim): precise 2D coordinate systems,
   plots/graphs, text and LaTeX-quality math, and *transforming one equation/shape into
   another* (morphing) as the signature move. Timing and easing must feel hand-crafted.
2. **Sebastian Lague-style coding adventure** (e.g. boids): iterative simulations,
   particle systems / fields, visualized in 2D or 3D, with parameters the author tweaks
   and re-runs. Determinism for clean re-renders.

The through-line: **everything reduces to a `Timeline` (a function of time)**, and the two
"hard" features — morphing and simulation — are made to *compose* into that model rather
than bolt on beside it.

## Core Philosophy

> Moments in a scene are independent of one another. Everything is a function of time.

The engine is **stateless and random-access**: `sample(t)` for any `t`, any order, no
hidden frame-to-frame state. This is what makes scrubbing/exploration and reproducible
export possible. The one principled exception is simulation, which is quarantined behind a
bake step (see [Simulation](#simulation)).

---

## Cross-Cutting Decisions

These two gate almost everything else and should be settled first.

| Decision | Leaning | Gates |
| --- | --- | --- |
| `Signal` carries `length() -> Option<f32>` (`None` = unbounded) | **Yes** | sequencing, animation defaults, export range |
| Renderer boundary is a backend-agnostic **draw-command list** | **Yes** | library split, all 4 renderer types, ffmpeg export |
| Timelines are built by **immutable composition**, never mutation | **Yes** | eliminates dependency cycles, matches Rust ownership |
| Everything renderable reduces to a **`Path`** (SDF as optimization) | **Lean yes** | universal morphing, unified stroke/fill |

---

## 1. Core Abstractions: Signal, Timeline, Path

**v1:** `timeline<T>` is a `shared_ptr`-backed, *mutable* object with `add_keyframe` /
`add_timeline` methods. `path = timeline<vec3>` (a type alias), `curve = timeline<path>`.

**v2:**

- `Signal` is the trait: `sample(&self, t: f32) -> Self::Output`, plus
  `length(&self) -> Option<f32>`. Closures implement it (already true in the Rust code).
- `Timeline<T>` is `Constant(T) | Dynamic(Arc<dyn Signal>)` (already implemented).
- `Path` is a **newtype**, not an alias:
  ```rust
  struct Path(Timeline<Vec2>);   // parameter alpha ∈ [0,1]
  ```
  - Implements `Signal<Output = Vec2>` and `From<Path> for Timeline<Vec2>` so a path can
    be dropped in anywhere a position timeline is expected (e.g. drive a circle along a
    path) — **without** losing the newtype's misuse protection.
  - Drop to `Vec2` for 2D; v1 used `vec3` with `z=0`, pure overhead. (3D gets its own type later.)

**Decisions**
- ✅ `Path` is a newtype implementing `Signal` + `From<Path> for Timeline<Vec2>`.
- ✅ Immutable composition (see [Combinators](#2-composition--combinators)).
- 🔶 Open: introduce `Alpha(f32)` newtype to distinguish "path parameter" from "wall-clock
  time" at the type level? Both are `f32` today; the confusion is the root of v1's
  `path_property` awkwardness. Leaning: at least document the convention on `Path`; adopt
  `Alpha` if the two-dimensional (`Timeline<Path<T>>`) cases prove error-prone.

## 2. Composition / Combinators

**v1:** only additive/multiplicative *layering* exists, and it lives *inside* the timeline
as a mutable `animations` list with a per-timeline offset. Compose/Sequence/Sample from
the README were never really built. Cycle-safety needed runtime ID tracking precisely
because timelines were mutable shared state.

**v2:** each combinator is its own zero-cost struct implementing `Signal`; `Timeline<T>`
gets fluent methods that construct them. Immutable composition means cycles are
**structurally impossible** — `c = a.add(b)` builds a new value from things that already
exist and are finalized; you can't retroactively make `b` depend on `c` without `unsafe`.
No ID tracking needed.

The five README operations, made precise:

- **Add** — `T: Add`. Binary and pure. Time-offset is *not* a parameter here; it's pushed
  out into a separate `shift(t0)` / `delay` combinator to keep `add` orthogonal.
- **Multiply** — `T: Mul`. Order matters (matrices/quats are non-commutative); document the
  convention (`a.mul(b)` = apply `b` then `a`, matrix-style).
- **Compose** — `self.compose(g)` where `g: Signal<Output=f32>` ⇒ `t → self.sample(g(t))`.
  The time-remapping primitive; loop, ping-pong, ease-over-time all fall out of it.
- **Sequence** — `a.then(b)`: play `a` for `a.length()`, then `b`. **Requires `length()`.**
- **Sample** — higher-order: `self.sample_within(|inner| …)`. The `Timeline<Path>` "timeline
  of timelines" pattern. May need `Box<dyn>` at the boundary in Rust.
- **Splice** *(new; not in the README)* — `base.splice(at, seg)`: `base` for `t < at`, `seg`
  (shifted to `at`) after. Cut-and-continue; one branch over `shift`. This is the primitive the
  Sequencing layer (§6) rides on — pulled out here because it belongs with the combinators.

**Decisions**
- ✅ Combinators are `Signal`-implementing wrapper structs, constructed via fluent methods.
- ✅ Time-offset lives in its own `shift`/`delay` combinator, not baked into add/multiply.
- ✅ `Signal::length() -> Option<f32>`; `None` = unbounded; `sequence` requires a bound.
- 🔶 Open: constant-folding (`Constant + Constant → Constant`)? Premature; the enum keeps it
  possible later. Skip until profiling asks for it.

## 3. Keyframes & Easing

**v1:** `keyframe<T> { at, value, easing_func }`, added by mutation. Sampling loops through
keyframes; eases if `T: lerpable`, otherwise snaps (good — handles text/discrete types).
`easing_func = std::function<double(double)>`.

**v2:**

- Easing attaches to the *destination* keyframe (the ease describes the segment arriving at
  it) — standard convention (CSS/AE). Name it to make that explicit.
- Easing is an **enum with a `Custom` escape hatch**, not just a closure — serializable
  (matters for scene files / reproducible export), matchable, cheap:
  ```rust
  enum Easing {
      Step, Linear,
      Quad(Direction), Cubic(Direction),
      CubicBezier(f32, f32, f32, f32),
      Custom(Arc<dyn Fn(f32) -> f32>),
  }
  ```
- Ease-or-snap: `T: Lerp` eases; non-`Lerp` snaps. Leaning **explicit** — a distinct
  `step`/discrete constructor rather than v1's implicit runtime branch — so intent shows.
- Builder for construction:
  ```rust
  Timeline::keyframes()
      .at(0.0, 0.0)
      .at(1.0, 100.0, Easing::Cubic(InOut))   // ease optional → Linear
      .build()
  ```

**Decisions**
- ✅ `Easing` enum + `Custom(Arc<dyn Fn>)`.
- ✅ Easing on the destination keyframe.
- 🔶 Open: implicit snap (one type, runtime branch like v1) vs explicit step constructor.
  Leaning explicit.

## 4. Scenes (the store)

**v1:** hand-rolled ECS — `scene` maps `type_index → collection<T>`, entities are `u32`,
every entity implicitly has `transform` + `visibility`, `get_view<…>` does set
intersection. Components are `timeline<T>`. Has bugs (`has_compenent` typo, `remove`
doesn't return, monotonic entity counter can dangle).

**v2 — genuine fork, decide explicitly:**

- **Roll-your-own vs `hecs`/`bevy_ecs`.** nelo is an *animation* engine, not a game: tens to
  thousands of entities, and every component is a `Timeline`. A full ECS is likely overkill
  and fights the functional model. **Leaning: purpose-built store.** Decision hinges on
  whether you want ECS-style queries/systems (simulation wants *something* like systems —
  see [Simulation](#simulation)) or just "a bag of animated objects."
- **Open vs closed component set.** v1 is open (arbitrary user component types via
  `std::any`). If the set can be **closed** (transform, visibility, shape/path, text,
  curve, …), the whole `Any`/`TypeId` machinery disappears and everything becomes
  serializable. **Leaning: closed**, at least to start.
- **Implicit components** (transform + visibility on every entity) become **struct fields**,
  not map entries — non-removable structurally, no runtime `throw`.
- **Entity handles:** `Entity(u32)` newtype. Generational index *only if* entities are
  destroyed mid-scene. **Leaning: append-only scene** → sidesteps generational indices.

**Decisions**
- 🔶 Open (high-impact): roll-your-own store vs ECS crate. Leaning roll-your-own.
- 🔶 Open: open vs closed component set. Leaning closed.
- ✅ transform + visibility are structural fields, not removable components.
- 🔶 Open: append-only vs destroyable entities. Leaning append-only.

## 5. Renderers (2D / Text / Math / 3D)

**v1:** concrete `curve_renderer` + `circle_renderer`, each owning GL buffers, immediate-mode
`begin(t)/submit/end`. A `scene_renderer` orchestrates. A comment explicitly wishes for a
graphics-API abstraction.

**v2 — the abstraction v1 wanted:**

- **Draw-command list is the boundary.** The scene emits a backend-agnostic command list;
  the GPU layer consumes it. This single decision gives us: the library split (`nelo`
  produces commands, `native` rasterizes), all four renderer types behind one seam, *and*
  the headless ffmpeg path (same commands feed window and encoder). Prefer this over a
  per-backend `Renderer` trait with an associated `Primitive`.
- **2D primitives: SDF vs tessellation.** SDF for filled/rounded primitives
  (resolution-independent — ideal for an animation engine and for crisp static shapes);
  CPU-tessellation (v1's curve trapezoid approach, which is sound) for arbitrary strokes and
  anything mid-morph. Mixing is fine and expected.
- **Camera/viewport** is a `Timeline`, not a renderer constructor constant. v1 hardcoded
  `scene_height = 5.0`; zoom/pan are just animatable timelines. This directly serves the
  3B1B use case (coordinate systems, camera moves).
- **Text & Math** are deferred internally but *not* architecturally: define the command
  enum so text/math are future command variants (glyph atlas / `cosmic-text` for text; math
  likely a TeX layout à la MathJax, or shell to a typesetter). Don't design their internals
  yet; don't paint into a circle-only command enum. **These are load-bearing for 3B1B** — the
  command-list must anticipate typeset runs and per-glyph transforms (needed for equation
  morphs).

**Decisions**
- ✅ Backend-agnostic draw-command list is the renderer boundary.
- ✅ Camera/viewport is a timeline.
- ✅ SDF for static/filled primitives, tessellation for strokes/morphs.
- 🔶 Open: exact command enum shape (must accommodate future text/math variants).

## 6. Animations & Sequencing

**v1:** none as such — "animations" were just the add/multiply layers; sequencing was
`scene.play(start, end)` driving wall-clock time by mutation (conflating authoring and playback).

**v2:** three thin layers over the existing combinators, **no new runtime concept**. The
load-bearing idea: **sequencing happens at *authoring* time, not render time.** There are two
clocks — wall-clock `t` (the argument to `Signal::sample`) and an *authoring cursor* ("where in
the storyboard I'm writing"). The cursor lives only in the builder and is gone by the time
anything is sampled. That is what lets an imperative, Manim-like `play`/`wait` authoring feel
produce a purely declarative result — a scene of independent timelines — with the stateless
model intact. More speculative than §1–§3; the value is real but the couplings below are.

**Layer 1 — Sugar (`Anim`): *what* changes.** `fade_in`, `move_to`, `recolor`, `pop_in`, `spin`
are constructors of an `Anim` — a duration plus a segment that begins at the value it is handed.
`from` is what makes `move_to` mean "from wherever I am now."

```rust
trait Anim {
    type Value;
    fn duration(&self) -> f32;
    /// Segment on local time [0, duration], beginning at `from`.
    fn segment(&self, from: Self::Value) -> Timeline<Self::Value>;
}
```

Every constructor desugars to the keyframe builder (§3); e.g. `move_to(p, d)` is
`keyframes(from).ease_at(d, p, …)`. Because `Keyframes` holds its last value past the end (§3),
a finished anim leaves the component parked with no extra machinery.

- **Relative vs absolute** (unchanged): `.to(x)` keyframes to an absolute value; `.by(dx)` is an
  additive layer (the Add/Multiply combinators, §2). v1's `add_timeline` is the relative case.

**Layer 2 — `splice`: *where in time*.** `.play` needs "old timeline until the cursor, then this
segment." That is the `splice` combinator (§2), adding one branch over `shift`. The segment
starts from `base.sample(at)`, so an interrupted `move_to` splices in mid-motion —
**interrupt-and-redirect is a free consequence**, not special-cased. Immutable composition holds:
`splice` builds a new timeline and consumes the old; it never mutates a shared `Signal`.

**Layer 3 — `Storyboard`: the authoring clock.** Owns the cursor. `play` reads the current value
at the cursor, splices the anim in, and advances the cursor; `wait` advances only; `par!`/`each`
place several commits and advance by the longest.

```rust
struct Storyboard<'s> { scene: &'s mut Scene, cursor: f32 }
impl Storyboard<'_> {
    fn wait(&mut self, d: f32) { self.cursor += d; }
    fn play(&mut self, action: Action) { self.cursor = action.commit(self.scene, self.cursor); }
}
```

`entity.animate(field, anim)` yields an `Action` — a bundle of type-erased commits
(`Box<dyn FnOnce(&mut Scene, f32)>`, one per component) so one `par!` can carry a move, a fade,
and a recolor together. Each commit, run at absolute time `at`: reads `scene.get(entity, field)`,
samples it at `at`, writes back `splice(base, at, anim.segment(from))`. `.delay(d)` shifts a
commit's offset — the stagger in `row.each(|i, e| … .delay(i * dt))`.

**Realism / cost.** This layers strictly on top of the existing combinators, so the *core* risk
is low — but three couplings are load-bearing and should be decided deliberately:

- **Depends on the closed component set (§4).** `scene.get(entity, field) -> &Timeline<T>` only
  typechecks against a fixed, typed field set (Opacity→`f32`, Position→`Vec2`, Color→`Vec4`).
  The open/`Any` store cannot give this signature without runtime downcasts. Sequencing
  ergonomics therefore *pull §4 toward closed* — decide the two together.
- **Commit order is authoring order, and `par` reads the pre-`par` value.** `play` commits
  synchronously, so a later `play` sees earlier splices; but two anims on the *same* field inside
  one `par!` both start from the value before the block. A real semantic to **document, not
  discover**.
- **`pop_in` overshoot needs an easing the enum lacks (§3).** Requires a `Back`/elastic variant
  or the `Custom`/`CubicBezier` escape hatch.

**Not a master timeline.** The `Storyboard` does *not* build one `Timeline<Scene>`; the scene is
the *product* of its per-component timelines, sampled componentwise at `t`. `.build()` finalizes
the scene and returns the total duration (the final cursor) as the default `export` range.

**Decisions**
- ✅ Resolves the old "return vs mutate" question: sugar builds a pure `Anim`/`Timeline`; `.play`
  is the *authoring-time* mutation that splices it in. Both halves hold, at different layers.
- ✅ `splice` is the one new combinator (catalogued in §2); interrupt-redirect falls out of it.
- ✅ Sequencing is authoring-time; the artifact is pure independent timelines (two-clocks).
- 🔶 Open: keep the imperative `Storyboard` clock, or expose only declarative composition
  (`a.then(b).par(c)`) with the clock as optional sugar? Leaning: storyboard as the default
  authoring surface, over a declarative core.
- 🔶 Open: `par` same-field start-value semantics (pre-block read) — confirm this is the rule.
- 🔶 Open: source of overshoot/elastic easing (`Back` variants vs `Custom`) — ties to §3.

## 7. Grouping & Positioning

**v1:** none; transforms are flat/per-entity.

**v2:** a group is a transform hierarchy — child world transform =
`parent.transform.mul(child.transform)`. **This is just the Multiply combinator on
transforms**, so grouping needs no new core infrastructure once transform composition
exists.

- 🔶 Open (genuine fork): is the hierarchy **stored** (parent/children on the entity —
  conventional, editable) or **functional** (child position literally *is*
  `compose(parent, local)` — very on-brand, harder to edit interactively)? Leaning: stored
  hierarchy for editability, computed via the multiply combinator.
- **Positioning helpers** (align, distribute, grid, stack) are sugar that outputs
  transforms — same category as Animations — so they compose with grouping for free. (3B1B
  leans heavily on arranging/aligning objects.)

## 8. Object Morphing (circle → square)

**Already ~80% solved by v1's design** and it fits the philosophy perfectly. Because shapes
are **parametric functions** (`Path`: `alpha → Vec2`), not vertex lists:

```
morph(t)(alpha) = lerp(a(alpha), b(alpha), t)
```

- No vertex-count reconciliation (both are continuous functions of `alpha`; sample as
  finely as you like at render time).
- Topology differences vanish for same-genus shapes (one closed loop → one closed loop).
- Fill follows the boundary for free.

**The real open problem is correspondence** — pointwise lerp only looks good if
parameterizations are aligned. v1 relies on invisible *convention* (all built-in paths
start at angle 0, CCW). Options for v2:

- Convention (cheap, breaks silently on custom paths).
- **Normalization**: `Path::morph(a, b, t)` resamples both to arc-length over N points before
  blending (robust, handles arbitrary user paths, costs a resample).
- Explicit `.align_to(other)` (rotate parameter offset / reverse winding to minimize travel;
  best visuals, most machinery).

**Decisions**
- ✅ Morphing = pointwise `lerp` of two `Path`s (`Timeline<Path>` lerp, as in v1).
- 🔶 Open: correspondence strategy. Leaning: arc-length **normalization by default**, with an
  opt-out for paths already known to correspond (built-ins).
- 🔶 Open: does morphing force tessellated rendering? Yes mid-morph — resolved by "every
  shape has a canonical `Path`; SDF is a static-only optimization." Confirms the
  everything-reduces-to-`Path` cross-cutting decision.

> For 3B1B specifically: equation morphing = per-glyph `Path` morphs with correspondence
> between source/target glyphs. The morph machinery is the same; the hard part is the
> glyph-matching policy, which belongs with the Math renderer.

## 9. Simulation

The principled exception. **Core tension: random-access (scrub anywhere) vs. recurrence
(state at `t` depends on history).** These are fundamentally in conflict; you reconcile them
by **converting recurrence into a lookup once (bake), then restoring random access.** The
only real decision is *where the seam goes*.

- **Simulation is a new timeline source** (alongside Constants, Closures, Keyframes, Input).
  A solver runs the recurrence and **bakes** each agent's trajectory into sampled/keyframed
  timelines. Afterward the rest of the engine sees ordinary functions of time — scrub,
  morph, transform, group, export all compose for free. Statefulness never leaks into
  `Signal`.
- **Simulation lives at the system layer, not the Signal layer** — a `Signal` only sees `t`,
  but boids need to see *the other agents*. So a simulator owns collective state and steps
  it as a whole (this is an argument for keeping *some* system concept in the scene):
  ```rust
  trait Simulator {
      type State;
      fn init(&self) -> Self::State;
      fn step(&mut self, state: &mut Self::State, t: f32, dt: f32);
      fn emit(&self, state: &Self::State) -> Vec<AgentSnapshot>;
  }
  ```
  A driver runs it at fixed `dt` from `start`, records snapshots, emits one baked
  `Timeline<Vec2>` per agent/property.
- **The seam — bake strategy:**
  - *Explicit bake* (`sim.bake(range, dt) -> timelines`): predictable memory, simple,
    reproducible; re-bake on param change. **Leaning: this is the primitive.**
  - *Lazy bake with checkpoints*: `sample(t)` integrates from nearest checkpoint, memoizes;
    backward scrub jumps to a checkpoint. Magical UX, unpredictable memory, more complex.
    Optional sugar layered on top later.
- **Determinism (non-negotiable, mostly free):** fixed `dt` (`frame n → t = n/fps`),
  **seeded RNG passed into the simulator**, no wall-clock. A baked sim is bit-reproducible —
  the payoff of the stateless model for export.
- **The one thing given up — name it explicitly:** *interactive* simulation (boids chasing
  the live mouse) can't be pre-baked, because input isn't known ahead. That's the
  live/preview path — forward-only, no scrub-backward. **Invariant: you get random-access OR
  live-recurrence, not both at once. Baking is the airlock** (record a live run → baked
  timeline → now scrubbable).
- **Composition payoff:** a baked sim is just timelines, so a boid's position can drive a
  morphing shape, get easing layered on, be offset/grouped, etc. Simulation output is a
  first-class citizen.

**Decisions**
- ✅ Simulation is a timeline *source*; state is quarantined in a solver and **baked** into
  timelines.
- ✅ Solver operates on collective state at the system layer (not per-`Signal`).
- ✅ Determinism via fixed `dt` + seeded RNG.
- 🔶 Open: explicit bake (leaning: primitive) vs lazy-with-checkpoints (leaning: later sugar).
- ✅ Invariant: random-access XOR live-recurrence; baking bridges them.

## 10. Exporting (ffmpeg)

**v1:** `core/encoder.h` + `scene.play(start, end)` (scene owns its own time and drives
rendering; conflates preview and export).

**v2:**

- **Headless render to an offscreen target**, read back, pipe to ffmpeg — the payoff of the
  command-list boundary (same commands as the window path).
- **Subprocess pipe vs libav FFI:** spawn `ffmpeg` and pipe raw frames to stdin (simple,
  zero FFI, matches the README's "via `ffmpeg`") vs. link `libav` (faster, heavier dep).
  **Leaning: subprocess pipe** to start.
- **Split the drivers** v1 conflated: `scene.preview()` (interactive, live time) vs
  `scene.export(range, fps, path)` (headless, fixed timestep). Same scene, two drivers.
- **Determinism invariant:** export uses a fixed timestep and must not sample live input.
  Input timelines used in an export must be recorded/replayed (ties to the simulation
  airlock). The stateless timeline model gives this for free — state it now so input
  timelines don't break it later.

**Decisions**
- ✅ Headless offscreen render → frame readback → encoder, over the command-list boundary.
- ✅ Split `preview()` (live) from `export()` (fixed-step, deterministic).
- 🔶 Open: ffmpeg subprocess (leaning) vs libav FFI.

---

## Library Boundary (recurring)

`GPU` currently lives in `src/context.rs` (the `nelo` lib) but needs a window handle, so any
consumer pulls in `wgpu` and can't run headless. Target split:

- `nelo` (lib): timelines, paths, scene/store, sugar, simulation solvers, **command-list
  production**. Pure logic, no window, testable, headless.
- `native` (bin): windowing (winit), `wgpu` rasterization of command lists, input capture,
  ffmpeg export driver.

This makes the ffmpeg path and testing clean, and is implied by the command-list boundary
decision.

## Suggested Build Order

1. **Combinators + `length()`** on `Signal` (the core; everything composes on it).
2. **Keyframes + `Easing`** (first real timeline source beyond closures/constants).
3. **`Path`** newtype + morphing (`lerp`) — unlocks the 3B1B shape story early.
4. **Command-list boundary** + move GPU into `native`; port the 2D renderer to consume
   commands.
5. **Scene/store** + transforms + grouping (multiply combinator).
6. **Animation sugar** + **sequencing** (`splice`, `Anim`, `Storyboard`) + positioning helpers.
7. **Simulation** solver + explicit bake — unlocks the Lague/boids story.
8. **Export** driver (offscreen + ffmpeg pipe).
9. **Text**, then **Math** renderers (the long pole for 3B1B).
10. **3D** renderer.

## Open Decisions Index

- Alpha newtype for path-parameter vs wall-clock time (§1)
- Constant-folding combinators (§2)
- Implicit snap vs explicit step keyframes (§3)
- Roll-your-own store vs ECS crate (§4) — *high impact*
- Open vs closed component set (§4)
- Append-only vs destroyable entities (§4)
- Draw-command enum shape / text-math accommodation (§5)
- Storyboard imperative clock vs pure declarative composition (§6)
- `par` same-field start-value semantics (§6)
- Overshoot/elastic easing source: `Back` variants vs `Custom` (§3, §6)
- Stored vs functional group hierarchy (§7)
- Morph correspondence strategy (§8)
- Explicit vs lazy simulation bake (§9)
- ffmpeg subprocess vs libav FFI (§10)
