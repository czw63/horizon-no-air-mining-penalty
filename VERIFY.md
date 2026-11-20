# No Air Mining Penalty (Horizon wasm)

## Target and behavior

- Mod ID: `no_air_mining_penalty`, version `0.1.0`, API `0.2`, side `both`.
- Horizon: `5ef7734fdd854add5efd6817178be3c988d8d050`,
  nightly 2026-11-19, Minecraft 26.3.
- Implements the NeoForge version, not the Fabric constant replacement.
- Subscribes only to `player_break_speed`. Resolves `event.player` through
  `host.entity`, then reads the player's on-ground state and eye position.
- Grounded: leaves `new_speed` unchanged. Eyes in water: also leaves it
  unchanged, including the airborne and `SUBMERGED_MINING_SPEED` penalties.
- Otherwise sets `new_speed = original_speed * 5.0`, not `new_speed * 5.0`.
  This intentionally replaces an earlier listener's speed in this branch,
  just as the Java implementation does.
- Always returns `Flow::Continue`; never modifies cancellation.

## Water detection and limitations

The mod queries the player's own level, `LevelRef::fluid_state`, membership
in `minecraft:water` (including flowing water), and
`LevelRef::fluid_height_for_camera`. The latter uses the engine's camera
surface height: ordinary fluid amount / 9, full height under the same fluid,
and full height for a source capped by a block with a sturdy underside.
Waterlogged blocks are handled through their fluid state, not block names.

The eye/surface comparison is inclusive (`<=`). Like Horizon's
`game/items/hand.rs::mining_eyes_in_water`, when the eye Y is exactly an integer,
the cell immediately below is also checked, so a capped source touching the
eyes is not missed. This uses actual fluid information, not a whole-block or
player-feet approximation. The water tag follows NeoForge's `FluidTags.WATER`;
custom tag membership may differ from Horizon's vanilla-only water enum check.

Unavailable entity handles, ground state, eye position, level, fluid reads,
or required water tags conservatively leave speed unchanged. In particular,
before client tags arrive, a nonempty fluid with unknown tag membership does
not receive a speed boost. An unloaded block above follows the engine API's
own height-query behavior. No native mixins or platform-specific code are used.

No game runtime or Apple Silicon installation has been tested here. Unit
tests check the decision and surface-boundary logic; they do not substitute
for exercising the host ABI in the target game.

## Build

Run from this project directory after the matching `cargo-horizon-mod`
installation has finished:

```sh
cargo horizon-mod build --wasm
cp target/horizon-mod/no_air_mining_penalty.hmod \
   target/horizon-mod/no_air_mining_penalty-0.1.0.hmod
```

The dependency uses the required `.horizon/sdk/api/...` path. The tool fetches
the pinned SDK into `.horizon/sdk/` before compiling, installs its pinned Rust
toolchain and `rust-src` if needed, writes root `mod.wasm`, generates `[wasm]`
in `horizon-mod.toml`, and packages the archive. Do not hand-write `[wasm]`.
No direct crates.io dependency has been added; transitive dependencies belong
to the SDK. Keep `Cargo.lock` if Cargo generates one.

The pinned tool's `pack_into` writes `<id>.hmod` (without the version).
The copy above preserves that original and produces the requested versioned
delivery artifact, with identical bytes:

```text
target/horizon-mod/no_air_mining_penalty-0.1.0.hmod
```

If the command is not found, wait for the already-running installation and
retry. Do not run another game-source clone or compile the game. For a new
build machine, install the mod tool built from the target commit using the
Horizon installation instructions; an older tool can produce incompatible
wasm std symbols even with the same Rust toolchain.

For a local SDK mirror, `HORIZON_SDK_BASE` may point to a directory containing
`sdk-5ef7734fdd854add5efd6817178be3c988d8d050/` with the original signed SDK
release files. `/root/horizon-mods/vendor/sdk/` is read-only reference data,
not a directory to modify. A normal build can fetch the official SDK directly.

Formatting and full crate tests (after SDK staging):

```sh
cargo fmt --package no-air-mining-penalty -- --check
cargo test --locked
```

The dependency-free decision tests can also run while the tool is installing:

```sh
mkdir -p target
rustc --edition=2024 --test src/mining.rs -o target/mining-tests
./target/mining-tests
```

## Install on Apple Silicon

1. Use the target Horizon build above, then fully stop the game.
2. In the launcher's instance menu choose **Open folder**. Put the built
   `.hmod` directly in that instance's `horizon-mods/` directory.
3. In `horizon-mods.txt` beside `options.txt`, add or update this line:

   ```text
   mod.no_air_mining_penalty:true
   ```

4. Restart the game and inspect the Mods screen for load/refusal diagnostics.
   The package contains `mod.wasm`; no Rust compiler, Linux library or Java
   mod loader is required on the Mac.
5. For multiplayer, install and enable the same package on the dedicated
   server as well as each client, so server validation and client prediction
   use the same mining speed. An integrated server shares the installation.

## In-game acceptance checks

Use survival mode, the same block/tool/effects, and compare enabled/disabled
runs. Test both singleplayer and a server with the mod on both installations.

| Situation | Expected result |
| --- | --- |
| Grounded, dry | Unchanged |
| Airborne, eyes dry | Speed equals original speed times 5 |
| Feet in water, eyes dry, airborne | Airborne penalty removed |
| Eyes underwater, grounded | Unchanged underwater penalty |
| Eyes underwater, airborne | Unchanged underwater AND airborne penalties |
| Flowing water surface below eyes | Treated as dry |
| Eyes exactly touching water surface | Treated as underwater |
| Capped source below eyes on an integer Y boundary | Treated as underwater |
| Lava / other non-water-tag fluid | Does not trigger the water exemption |

Repeat underwater checks with altered `SUBMERGED_MINING_SPEED`; the mod must
not override that attribute. Other mods editing break speed remain subject to
listener order. There is no extra synchronization packet.

## Verification result

Verified on 2026-11-20:

- Copied the provided API source into the ignored local `.horizon/sdk/api/`
  for host-side compilation. No files under `vendor/` were changed. This is
  temporary source staging, not a fabricated verified SDK cache: no SDK
  `build` marker was written, and the mod tool will still fetch/verify its SDK.
- Full crate compilation and `cargo test` succeeded, including the actual
  `#[mod_api::horizon_mod]` implementation and the host API calls. All 11 unit
  tests passed; doc-tests passed (0 tests).
- Retained generated `Cargo.lock`, locking the SDK's transitive dependencies.
- `cargo test --locked --offline` passed again with all 11 tests. To avoid the
  ongoing tool install's package-cache lock, these Cargo tests used
  `CARGO_HOME=/root/horizon-mods/no-air-mining-penalty/target/cargo-home`
  with the existing registry linked there.
- `cargo fmt --package no-air-mining-penalty -- --check` passed.
- Standalone `rustc --edition=2024 --test src/mining.rs` also passed all 11 tests.
- `cargo horizon-mod build --wasm` was attempted twice, after the code and
  instructions were complete. Both attempts exited 101:
  `error: no such command: horizon-mod`. The existing tool installation was
  still fetching its source, and `/root/.cargo/bin/cargo-horizon-mod` did not
  yet exist. No competing install, game clone or game build was started.
- Consequently **no `mod.wasm` or `.hmod` has been produced**, wasm compilation
  and archive contents remain unverified, and `[wasm]` is intentionally absent
  until the build tool generates it. The wasm build acceptance criterion is
  still pending; host-side compilation is not being reported as wasm success.

After the tool installation finishes, run the two commands in **Build**.
Then check the archive contains root `mod.wasm` and a generated `[wasm]` table
with its matching SHA-512, and run the in-game acceptance checks above.
