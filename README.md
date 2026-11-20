# No Air Mining Penalty — Horizon 版

Minecraft 26.3 的 Horizon 客户端 mod。去掉「脚不着地时挖掘速度 ÷5」的原版惩罚，水下惩罚保持原版。

这是 [no_air_mining_penalty](https://github.com/czw63/no_air_mining_penalty)（Fabric / NeoForge 版）在
[Horizon](https://github.com/horizon-rust/horizon)（Rust 重写的 MC 客户端）上的移植。

## 装

Horizon launcher → Mods 页面 → **Add from GitHub** → 填：

```
https://github.com/czw63/horizon-no-air-mining-penalty
```

launcher 会取本仓库的源码，用它自带的 Rust 工具链编一遍（这个 mod 逻辑只有二十几行，几秒钟），
不需要你预装 Rust，也不会重编游戏本体。

## 行为

- 玩家不在方块上（`on_ground == false`）且眼睛不在水里 → 挖掘速度还原成 ×5（即去掉惩罚）。
- 眼睛在水里 → 不动，保持原版的水下惩罚（`SUBMERGED_MINING_SPEED`）。
- 拿不到玩家状态、眼睛坐标、方块流体或水标签时一律不改速度（保守）。
- 水下判定用真实流体状态（`LevelRef::fluid_state` + `minecraft:water` 标签 +
  `fluid_height_for_camera`），不是整格近似；眼睛正好在整数边界时会额外检查下面一格，
  和 Horizon 自己的 `mining_eyes_in_water` 对齐。

语义对齐 NeoForge 版（走 `BreakSpeed` 事件那一版），不是 Fabric 那个直接改常量的版本。

## 目标 build

`[native]` 里 pin 死 `5ef7734fdd854add5efd6817178be3c988d8d050`（nightly 2026-11-19）。

Horizon 每天出 nightly。游戏 build 不匹配时 launcher 会报错，需要把 `horizon-mod.toml` 的 `horizon`
换成新 commit 重新发一版。

## 实现

只用 mod API 层的 `player_break_speed` hook（`side = both`，NeoForge `PlayerEvent.BreakSpeed` 的对应物），
不用 mixin。

- `src/lib.rs` — hook 入口，从 `event.player` 取实体，组装 `PlayerView`
- `src/mining.rs` — 判定逻辑（trait 抽象，方便单测）

## License

MIT，同上游 mod。
