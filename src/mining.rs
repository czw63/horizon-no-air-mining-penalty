pub(crate) trait MiningView {
    fn on_ground(&self) -> Option<bool>;
    fn eye_position(&self) -> Option<[f64; 3]>;
    // None means unavailable; Some(None) means the cell contains no water.
    fn water_surface(&self, pos: [i32; 3]) -> Option<Option<f32>>;
}

pub(crate) fn adjust_speed(view: &impl MiningView, original_speed: f32, new_speed: &mut f32) {
    if view.on_ground() == Some(false) && eyes_in_water(view) == Some(false) {
        *new_speed = original_speed * 5.0;
    }
}

fn eyes_in_water(view: &impl MiningView) -> Option<bool> {
    let eye = view.eye_position()?;
    if eye.iter().any(|v| !v.is_finite()) {
        return None;
    }
    let [x, y, z] = eye.map(|v| v.floor() as i32);
    let inside = |y| {
        Some(
            view.water_surface([x, y, z])?
                .is_some_and(|height| eye[1] <= f64::from(y) + f64::from(height)),
        )
    };
    if inside(y)? {
        return Some(true);
    }
    // Match mining_eyes_in_water: a capped source below can touch eyes on a cell boundary.
    if eye[1] == f64::from(y) {
        inside(y.checked_sub(1)?)
    } else {
        Some(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    struct View {
        ground: Option<bool>,
        eye: Option<[f64; 3]>,
        surface: Option<Option<f32>>,
        below: Option<Option<f32>>,
        reads: RefCell<Vec<[i32; 3]>>,
    }

    impl Default for View {
        fn default() -> Self {
            Self {
                ground: Some(false),
                eye: Some([1.5, 64.5, 2.5]),
                surface: Some(None),
                below: Some(None),
                reads: RefCell::new(Vec::new()),
            }
        }
    }

    impl MiningView for View {
        fn on_ground(&self) -> Option<bool> {
            self.ground
        }

        fn eye_position(&self) -> Option<[f64; 3]> {
            self.eye
        }

        fn water_surface(&self, pos: [i32; 3]) -> Option<Option<f32>> {
            self.reads.borrow_mut().push(pos);
            if pos[1] == self.eye.unwrap()[1].floor() as i32 {
                self.surface
            } else {
                self.below
            }
        }
    }

    fn speed(view: &View) -> f32 {
        let mut speed = 7.0;
        adjust_speed(view, 2.0, &mut speed);
        speed
    }

    #[test]
    fn airborne_dry_uses_original_not_preceding_mod_speed() {
        assert_eq!(speed(&View::default()), 10.0);
    }

    #[test]
    fn grounded_and_unknown_ground_skip_all_fluid_queries() {
        for ground in [Some(true), None] {
            let view = View {
                ground,
                ..View::default()
            };
            assert_eq!(speed(&view), 7.0);
            assert!(view.reads.borrow().is_empty());
        }
    }

    #[test]
    fn underwater_airborne_preserves_speed() {
        let view = View {
            surface: Some(Some(8.0 / 9.0)),
            ..View::default()
        };
        assert_eq!(speed(&view), 7.0);
    }

    #[test]
    fn eyes_above_flowing_water_are_dry() {
        let view = View {
            surface: Some(Some(4.0 / 9.0)),
            ..View::default()
        };
        assert_eq!(speed(&view), 10.0);
    }

    #[test]
    fn exact_surface_is_submerged() {
        let view = View {
            surface: Some(Some(0.5)),
            ..View::default()
        };
        assert_eq!(speed(&view), 7.0);
    }

    #[test]
    fn boundary_checks_capped_source_below() {
        let view = View {
            eye: Some([1.5, 65.0, 2.5]),
            below: Some(Some(1.0)),
            ..View::default()
        };
        assert_eq!(speed(&view), 7.0);
        assert_eq!(*view.reads.borrow(), [[1, 65, 2], [1, 64, 2]]);
    }

    #[test]
    fn boundary_above_uncapped_source_is_dry() {
        let view = View {
            eye: Some([1.5, 65.0, 2.5]),
            below: Some(Some(8.0 / 9.0)),
            ..View::default()
        };
        assert_eq!(speed(&view), 10.0);
    }

    #[test]
    fn negative_coordinates_use_floor() {
        let view = View {
            eye: Some([-0.2, -0.2, -1.2]),
            ..View::default()
        };
        assert_eq!(speed(&view), 10.0);
        assert_eq!(*view.reads.borrow(), [[-1, -1, -2]]);
    }

    #[test]
    fn missing_eye_fluid_or_tags_preserves_speed() {
        let missing_eye = View {
            eye: None,
            ..View::default()
        };
        let missing_surface = View {
            surface: None,
            ..View::default()
        };
        let missing_below = View {
            eye: Some([1.5, 65.0, 2.5]),
            below: None,
            ..View::default()
        };
        for view in [missing_eye, missing_surface, missing_below] {
            assert_eq!(speed(&view), 7.0);
        }
    }

    #[test]
    fn invalid_eye_position_preserves_speed() {
        let view = View {
            eye: Some([f64::NAN, 64.5, 2.5]),
            ..View::default()
        };
        assert_eq!(speed(&view), 7.0);
        assert!(view.reads.borrow().is_empty());
    }

    #[test]
    fn zero_and_negative_original_speeds_are_not_clamped() {
        for original in [0.0, -1.0] {
            let mut speed = 7.0;
            adjust_speed(&View::default(), original, &mut speed);
            assert_eq!(speed, original * 5.0);
        }
    }
}
