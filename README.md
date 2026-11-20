# No Air Mining Penalty — Horizon 版

Minecraft 26.3 的 Horizon 客户端 mod。去掉「脚不着地时挖掘速度 ÷5」的原版惩罚，水下惩罚保持原版。

这是 [no_air_mining_penalty](https://github.com/czw63/no_air_mining_penalty)（Fabric / NeoForge 版）在
[Horizon](https://github.com/horizon-rust/horizon)（Rust 重写的 MC 客户端）上的移植。

## 装

用 Horizon launcher 的 Mods 页面 → **Add from GitHub** → 填这个仓库地址：

```
https://github.com/czw63/horizon-no-air-mining-penalty
```

launcher 会取最新 release 里的 `.hmod`，直接装进实例，不需要你本机编译。

或者手动：把 release 里的 `.hmod` 放进游戏目录的 `horizon-mods/`，在 `horizon-mods.txt` 加一行
`mod.no_air_mining_penalty:true`。

## 行为

- 玩家不在方块上（`on_ground == false`）且眼睛不在水里 → 挖掘速度还原成 ×5（即去掉惩罚）。
- 眼睛在水里 → 不动，保持原版的水下惩罚（`SUBMERGED_MINING_SPEED`）。
- 拿不到玩家状态、眼睛坐标、方块流体或水标签时一律不改速度（保守）。
- 水下判定用的是真实流体状态（`LevelRef::fluid_state` + `minecraft:water` 标签 +
  `fluid_height_for_camera`），不是整格近似；眼睛正好在整数边界时会额外检查下面一格，
  和 Horizon 自己的 `mining_eyes_in_water` 对齐。

语义对齐 NeoForge 版（走 `BreakSpeed` 事件那一版），不是 Fabric 那个直接改常量的版本。

## 目标 build

pin 死 `5ef7734fdd854add5efd6817178be3c988d8d050`（nightly 2026-11-19）。

Horizon 每天出 nightly，`[wasm]` 里的 toolchain 和 sha512 是编进包里的。游戏 build 不匹配时会被拒载，
需要按新 commit 重新编译本仓库再发一版（本仓库的 Cargo.toml 里有 sdk pin）。

## 实现

wasm mod，只用 mod API 层的 `player_break_speed` hook（`side = both`，NeoForge `PlayerEvent.BreakSpeed`
的对应物），没有 mixin、没有 native 代码、不需要 `--trust-native-code`。

- `src/lib.rs` — hook 入口，从 `event.player` 取实体，组装 `PlayerView`
- `src/mining.rs` — 判定逻辑（trait 抽象，方便单测）

`cargo test` 有 11 个单测（离地/着地/水下/边界/未知值/负坐标/零速）。

## 构建

```
cargo install --locked --git https://github.com/horizon-rust/horizon --rev <sdk commit> cargo-horizon-mod
proxychains4 -q cargo horizon-mod build --wasm
```

`cargo horizon-mod` 用 ureq，不读 `http_proxy`，需要代理时只能靠 proxychains 这类强制转发。

## License

MIT，同上游 mod。
