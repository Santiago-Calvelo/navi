//! Bounded, pure placement at admission time. No Windows calls or frame work.
use navi_state::{
    reactions::{Bounds, Reaction},
    visual::{variation_unit, Placement, ResolvedVariation},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MonitorArea {
    pub id: u64,
    pub work: Bounds,
    pub dpi: u32,
    pub blocked: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct ResolvedManifestation {
    pub reaction: Reaction,
    pub viewport: Bounds,
    pub dpi: u32,
    pub variation: ResolvedVariation,
    pub draw_scale: f32,
    pub monitor: u64,
    pub zone: u8,
    pub motion: bool,
}

#[derive(Clone, Copy)]
struct Recent {
    monitor: u64,
    bounds: Bounds,
    zone: u8,
    at: u64,
}

#[derive(Default)]
pub struct OrganicPlanner {
    recent: [Option<Recent>; 8],
    next: usize,
    sequence: u64,
    root: Option<u64>,
}

pub fn intersects(a: Bounds, b: Bounds) -> bool {
    a.left < b.right && a.right > b.left && a.top < b.bottom && a.bottom > b.top
}

fn dimensions(b: Bounds) -> Option<(f64, f64)> {
    let w = i64::from(b.right) - i64::from(b.left);
    let h = i64::from(b.bottom) - i64::from(b.top);
    (w > 0 && h > 0).then_some((w as f64, h as f64))
}

impl OrganicPlanner {
    pub fn resolve(
        &mut self,
        mut reaction: Reaction,
        monitors: &[MonitorArea],
        exclusions: &[Option<Bounds>; 3],
        root: u64,
        motion: bool,
        placement: Placement,
    ) -> Option<ResolvedManifestation> {
        if self.root != Some(root) {
            *self = Self {
                root: Some(root),
                ..Self::default()
            };
        }
        // Each successful manifestation consumes exactly one sequence number.
        let seed = root.wrapping_add(self.sequence.wrapping_mul(0x9e3779b97f4a7c15));
        let variation =
            ResolvedVariation::sample(seed, reaction.intensity, reaction.duration_ms, motion);
        let mut best: Option<(f32, ResolvedManifestation)> = None;
        for monitor in monitors {
            let mut local_best: Option<(f64, ResolvedManifestation)> = None;
            let Some((mw, mh)) = dimensions(monitor.work) else {
                continue;
            };
            if monitor.blocked || monitor.dpi == 0 || monitor.dpi > 960 {
                continue;
            }
            let dpi = f64::from(monitor.dpi) / 96.0;
            let margin = 16.0 * dpi;
            let aw = mw - margin * 2.0;
            let ah = mh - margin * 2.0;
            if aw <= 0.0 || ah <= 0.0 {
                continue;
            }
            let (sin, cos) = f64::from(variation.rotation_radians).sin_cos();
            let envelope_w = 540.0 * cos.abs() + 300.0 * sin.abs();
            let envelope_h = 540.0 * sin.abs() + 300.0 * cos.abs();
            let desired_width = (mw * 0.25)
                .min(mh * 0.44 * 1.8)
                .clamp(240.0 * dpi, 560.0 * dpi);
            let requested = desired_width / (540.0 * dpi) * f64::from(variation.scale);
            // Padding includes stroke and animated fragments in canonical space.
            let padding = 8.0 * dpi;
            let fit = requested
                .min((aw - 2.0 * padding) / (envelope_w * dpi))
                .min((ah - 2.0 * padding) / (envelope_h * dpi));
            if fit <= 0.0 || fit * 540.0 < 100.0 {
                continue;
            }
            let width = (envelope_w * fit * dpi + 2.0 * padding).ceil();
            let height = (envelope_h * fit * dpi + 2.0 * padding).ceil();
            if width > aw || height > ah {
                continue;
            }
            let left = f64::from(monitor.work.left) + margin;
            let top = f64::from(monitor.work.top) + margin;
            let rw = aw - width;
            let rh = ah - height;
            let same = monitor.work == reaction.event.work_area;
            let anchors = [
                (0.0, 0.0),
                (0.5, 0.0),
                (1.0, 0.0),
                (1.0, 0.5),
                (1.0, 1.0),
                (0.5, 1.0),
                (0.0, 1.0),
                (0.0, 0.5),
            ];
            for (zone, (ax, ay)) in anchors.into_iter().enumerate() {
                let stream = 100 + zone as u64 * 4;
                // Jitter stays inside each safe band, not across the whole desktop.
                let fx =
                    (ax + (f64::from(variation_unit(seed, stream)) - 0.5) * 0.18).clamp(0.0, 1.0);
                let fy = (ay + (f64::from(variation_unit(seed, stream + 1)) - 0.5) * 0.18)
                    .clamp(0.0, 1.0);
                let x = (left + rw * fx).round() as i32;
                let y = (top + rh * fy).round() as i32;
                let bounds = Bounds {
                    left: x,
                    top: y,
                    right: x.saturating_add(width as i32),
                    bottom: y.saturating_add(height as i32),
                };
                if bounds.left < monitor.work.left
                    || bounds.top < monitor.work.top
                    || bounds.right > monitor.work.right
                    || bounds.bottom > monitor.work.bottom
                {
                    continue;
                }
                if exclusions.iter().flatten().any(|b| intersects(bounds, *b)) {
                    continue;
                }
                // Related window controls remain excluded even if the window spans monitors.
                let title = Bounds {
                    bottom: reaction
                        .event
                        .bounds
                        .top
                        .saturating_add((40.0 * dpi) as i32),
                    ..reaction.event.bounds
                };
                if intersects(bounds, title) {
                    continue;
                }
                let mut cost = f64::from(variation_unit(seed, stream + 2)) * 0.6;
                // Proximity only chooses a zone within this monitor.
                if same {
                    let (cx, cy) = (
                        (f64::from(bounds.left) + f64::from(bounds.right)) * 0.5,
                        (f64::from(bounds.top) + f64::from(bounds.bottom)) * 0.5,
                    );
                    let (tx, ty) = match placement {
                        Placement::NearCursor => (
                            f64::from(reaction.event.point.x),
                            f64::from(reaction.event.point.y),
                        ),
                        Placement::WindowAnchor => (
                            f64::from(reaction.event.bounds.right),
                            f64::from(reaction.event.bounds.top),
                        ),
                        Placement::SafeRandom => (cx, cy),
                    };
                    cost += ((cx - tx).hypot(cy - ty) / (mw.hypot(mh))).min(1.0) * 0.5;
                }
                for recent in self.recent.iter().flatten() {
                    let age = reaction.start.saturating_sub(recent.at);
                    if age > 60_000 || recent.monitor != monitor.id {
                        continue;
                    }
                    let weight = 1.0 - age as f64 / 60_000.0;
                    if recent.zone == zone as u8 {
                        cost += weight * 3.0;
                    }
                    if intersects(bounds, recent.bounds) {
                        cost += weight * 2.0;
                    }
                }
                reaction.duration_ms = variation.duration_ms;
                let plan = ResolvedManifestation {
                    reaction,
                    viewport: bounds,
                    dpi: monitor.dpi,
                    variation,
                    draw_scale: fit as f32,
                    monitor: monitor.id,
                    zone: zone as u8,
                    motion,
                };
                if local_best.as_ref().is_none_or(|(score, _)| cost < *score) {
                    local_best = Some((cost, plan));
                }
            }
            // Equal chance per eligible monitor, independent of candidate count,
            // DPI, event proximity and recency. Stable IDs avoid enumeration bias.
            if let Some((_, plan)) = local_best {
                let rank = variation_unit(seed, monitor.id.wrapping_add(1000));
                if best.as_ref().is_none_or(|(old, previous)| {
                    rank < *old || (rank == *old && monitor.id < previous.monitor)
                }) {
                    best = Some((rank, plan));
                }
            }
        }
        let (_, plan) = best?;
        self.recent[self.next] = Some(Recent {
            monitor: plan.monitor,
            bounds: plan.viewport,
            zone: plan.zone,
            at: reaction.start,
        });
        self.next = (self.next + 1) % self.recent.len();
        self.sequence = self.sequence.wrapping_add(1);
        Some(plan)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use navi_state::{
        reactions::{DesktopEvent, DesktopKind, Intensity, Point},
        visual::Family,
    };
    fn reaction(start: u64) -> Reaction {
        Reaction {
            event: DesktopEvent {
                kind: DesktopKind::WindowFocus,
                source: 1,
                bounds: Bounds {
                    left: 100,
                    top: 50,
                    right: 1400,
                    bottom: 900,
                },
                work_area: Bounds {
                    left: 0,
                    top: 0,
                    right: 1920,
                    bottom: 1040,
                },
                point: Point { x: 700, y: 500 },
                own: false,
                suppressed: false,
                dpi: 96,
            },
            start,
            duration_ms: 850,
            intensity: Intensity::Experimental,
            family: Family::StackV2,
        }
    }
    fn monitors() -> [MonitorArea; 2] {
        [
            MonitorArea {
                id: 1,
                work: reaction(0).event.work_area,
                dpi: 96,
                blocked: false,
            },
            MonitorArea {
                id: 2,
                work: Bounds {
                    left: -2560,
                    top: -200,
                    right: 0,
                    bottom: 1240,
                },
                dpi: 144,
                blocked: false,
            },
        ]
    }
    #[test]
    fn deterministic_sequence_varies_monitor_zone_and_material() {
        let (mut a, mut b) = (OrganicPlanner::default(), OrganicPlanner::default());
        let mut choices = Vec::new();
        for i in 0..16 {
            let p = a
                .resolve(
                    reaction(i * 1000),
                    &monitors(),
                    &[None; 3],
                    42,
                    true,
                    Placement::WindowAnchor,
                )
                .unwrap();
            let q = b
                .resolve(
                    reaction(i * 1000),
                    &monitors(),
                    &[None; 3],
                    42,
                    true,
                    Placement::WindowAnchor,
                )
                .unwrap();
            assert_eq!(
                (p.viewport, p.monitor, p.zone, p.variation),
                (q.viewport, q.monitor, q.zone, q.variation)
            );
            if let Some((_, old)) = choices.last() {
                assert_ne!(*old, p.viewport);
            }
            choices.push((p.monitor, p.viewport));
        }
        assert!(choices.iter().any(|(m, _)| *m == 1));
        assert!(choices.iter().any(|(m, _)| *m == 2));
    }
    #[test]
    fn transformed_canvas_stays_inside_each_mixed_dpi_monitor() {
        let monitors = monitors();
        let mut planner = OrganicPlanner::default();
        for i in 0..100 {
            let p = planner
                .resolve(
                    reaction(i * 1000),
                    &monitors,
                    &[None; 3],
                    i,
                    true,
                    Placement::SafeRandom,
                )
                .unwrap();
            let m = monitors.iter().find(|m| m.id == p.monitor).unwrap();
            assert!(
                p.viewport.left >= m.work.left
                    && p.viewport.top >= m.work.top
                    && p.viewport.right <= m.work.right
                    && p.viewport.bottom <= m.work.bottom
            );
            assert_eq!(p.dpi, m.dpi);
        }
    }
    #[test]
    fn single_monitor_recency_uses_multiple_zones_at_all_supported_test_dpis() {
        for dpi in [96, 144, 192] {
            let monitor = MonitorArea {
                dpi,
                ..monitors()[0]
            };
            let mut planner = OrganicPlanner::default();
            let mut zones = [false; 8];
            let mut previous = None;
            for i in 0..16 {
                let p = planner
                    .resolve(
                        reaction(i * 1000),
                        &[monitor],
                        &[None; 3],
                        42,
                        true,
                        Placement::SafeRandom,
                    )
                    .unwrap();
                assert_ne!(previous, Some(p.viewport));
                previous = Some(p.viewport);
                zones[p.zone as usize] = true;
                let title = Bounds {
                    bottom: reaction(0).event.bounds.top + 40 * (dpi as i32) / 96,
                    ..reaction(0).event.bounds
                };
                assert!(!intersects(p.viewport, title));
            }
            assert!(zones.into_iter().filter(|used| *used).count() >= 4);
        }
    }
    #[test]
    fn exclusions_and_tiny_invalid_monitors_omit_without_allocating() {
        let mut p = OrganicPlanner::default();
        let monitor = monitors()[0];
        assert!(p
            .resolve(
                reaction(0),
                &[monitor],
                &[Some(monitor.work), None, None],
                42,
                true,
                Placement::SafeRandom
            )
            .is_none());
        assert!(p
            .resolve(
                reaction(0),
                &[MonitorArea { dpi: 0, ..monitor }],
                &[None; 3],
                42,
                true,
                Placement::SafeRandom
            )
            .is_none());
        assert!(p
            .resolve(
                reaction(0),
                &[MonitorArea {
                    work: Bounds {
                        left: 0,
                        top: 0,
                        right: 20,
                        bottom: 20
                    },
                    ..monitor
                }],
                &[None; 3],
                42,
                true,
                Placement::SafeRandom
            )
            .is_none());
        assert_eq!(p.sequence, 0);
        assert!(p
            .resolve(
                reaction(0),
                &[MonitorArea {
                    blocked: true,
                    ..monitor
                }],
                &[None; 3],
                42,
                true,
                Placement::SafeRandom
            )
            .is_none());
    }
    #[test]
    fn monitor_lottery_is_balanced_and_independent_of_event_order_dpi_and_recency() {
        let monitors = monitors();
        let reversed = [monitors[1], monitors[0]];
        let mut counts = [0usize; 2];
        for seed in 0..4096 {
            let resolve = |areas: &[MonitorArea], event_work: Bounds| {
                OrganicPlanner::default()
                    .resolve(
                        Reaction {
                            event: DesktopEvent {
                                work_area: event_work,
                                ..reaction(0).event
                            },
                            ..reaction(0)
                        },
                        areas,
                        &[None; 3],
                        seed,
                        true,
                        Placement::WindowAnchor,
                    )
                    .unwrap()
                    .monitor
            };
            let chosen = resolve(&monitors, monitors[0].work);
            assert_eq!(chosen, resolve(&reversed, monitors[1].work));
            counts[(chosen - 1) as usize] += 1;
        }
        // Wide statistical margin; equal eligibility must not inherit focus bias.
        assert!(
            counts.iter().all(|n| (1843..=2253).contains(n)),
            "{counts:?}"
        );
        let mut planner = OrganicPlanner::default();
        let mut repeated_monitor = false;
        let mut previous = 0;
        for i in 0..128 {
            let chosen = planner
                .resolve(
                    reaction(i * 1000),
                    &monitors,
                    &[None; 3],
                    42,
                    true,
                    Placement::SafeRandom,
                )
                .unwrap()
                .monitor;
            repeated_monitor |= chosen == previous;
            previous = chosen;
        }
        // Organic randomness permits repeats; no forced cross-monitor alternation.
        assert!(repeated_monitor);
    }

    #[test]
    fn lottery_skips_fully_excluded_monitors_and_keeps_the_other_eligible() {
        let monitors = monitors();
        for seed in 0..128 {
            let p = OrganicPlanner::default()
                .resolve(
                    reaction(0),
                    &monitors,
                    &[Some(monitors[0].work), None, None],
                    seed,
                    true,
                    Placement::SafeRandom,
                )
                .unwrap();
            assert_eq!(p.monitor, 2);
        }
    }
}
