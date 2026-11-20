use mod_api::{EntityRef, Flow, Hooks, Host, HostSide, PlayerBreakSpeedEvent};

mod mining;

#[derive(Default)]
pub struct Mod;

#[mod_api::horizon_mod]
impl Hooks for Mod {
    fn player_break_speed(&self, host: HostSide<'_>, event: &mut PlayerBreakSpeedEvent) -> Flow {
        let host: &dyn Host = match host {
            HostSide::Server(host) => host,
            HostSide::Client(host) => host,
        };
        if let Some(player) = host.entity(event.player) {
            mining::adjust_speed(
                &PlayerView { host, player },
                event.original_speed,
                &mut event.new_speed,
            );
        }
        Flow::Continue
    }
}

struct PlayerView<'a> {
    host: &'a dyn Host,
    player: EntityRef,
}

impl mining::MiningView for PlayerView<'_> {
    fn on_ground(&self) -> Option<bool> {
        self.player.on_ground(self.host)
    }

    fn eye_position(&self) -> Option<[f64; 3]> {
        self.player.eye_position(self.host)
    }

    fn water_surface(&self, pos: [i32; 3]) -> Option<Option<f32>> {
        let level = self.player.level(self.host)?;
        let fluid = level.fluid_state(self.host, pos)?;
        if fluid.is_empty() {
            return Some(None);
        }
        let name = fluid.fluid.id(self.host);
        if !self.host.fluid_in_tag(&name, "minecraft:water")? {
            return Some(None);
        }
        level.fluid_height_for_camera(self.host, pos).map(Some)
    }
}
