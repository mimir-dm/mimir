//! Pan and zoom for a map scene, shared by the DM map and the player
//! display: one pointer pans, two pointers pinch, the wheel zooms at the
//! cursor. [`View`] is the pure transform (host-tested).

use std::collections::HashMap;

use leptos::html::Div;
use leptos::prelude::*;
use wasm_bindgen::JsCast;

/// Pan and zoom: screen = map × scale + (tx, ty).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct View {
    pub scale: f64,
    pub tx: f64,
    pub ty: f64,
}

pub const MIN_SCALE: f64 = 0.05;
pub const MAX_SCALE: f64 = 8.0;

impl View {
    pub const IDENTITY: View = View {
        scale: 1.0,
        tx: 0.0,
        ty: 0.0,
    };

    /// The map point under a screen point (screen relative to the scene box).
    pub fn to_map(self, sx: f64, sy: f64) -> (f64, f64) {
        ((sx - self.tx) / self.scale, (sy - self.ty) / self.scale)
    }

    /// Zoom by `factor`, keeping the map point under (sx, sy) in place.
    pub fn zoom_at(self, factor: f64, sx: f64, sy: f64) -> View {
        let scale = (self.scale * factor).clamp(MIN_SCALE, MAX_SCALE);
        let f = scale / self.scale;
        View {
            scale,
            tx: sx - (sx - self.tx) * f,
            ty: sy - (sy - self.ty) * f,
        }
    }

    /// The whole map in a box, centred.
    pub fn fit(map_w: f64, map_h: f64, box_w: f64, box_h: f64) -> View {
        if map_w <= 0.0 || map_h <= 0.0 || box_w <= 0.0 || box_h <= 0.0 {
            return View::IDENTITY;
        }
        let scale = (box_w / map_w)
            .min(box_h / map_h)
            .clamp(MIN_SCALE, MAX_SCALE);
        View {
            scale,
            tx: (box_w - map_w * scale) / 2.0,
            ty: (box_h - map_h * scale) / 2.0,
        }
    }

    pub fn transform(self) -> String {
        format!(
            "translate({:.2} {:.2}) scale({:.4})",
            self.tx, self.ty, self.scale
        )
    }
}

/// Pan or pinch in progress.
#[derive(Debug, Clone, Default)]
struct Gesture {
    pointers: HashMap<i32, (f64, f64)>,
    start_view: Option<View>,
    start_points: Vec<(f64, f64)>,
    moved: bool,
}

/// The view of a scene and the gestures on it. `Copy`: hand it to closures.
#[derive(Clone, Copy)]
pub struct Viewport {
    pub view: RwSignal<View>,
    /// The scene box: put `node_ref=viewport.scene` on it.
    pub scene: NodeRef<Div>,
    gesture: StoredValue<Gesture>,
}

impl Default for Viewport {
    fn default() -> Self {
        Self::new()
    }
}

impl Viewport {
    pub fn new() -> Self {
        Self {
            view: RwSignal::new(View::IDENTITY),
            scene: NodeRef::new(),
            gesture: StoredValue::new(Gesture::default()),
        }
    }

    fn rect(&self) -> Option<web_sys::DomRect> {
        self.scene
            .get_untracked()
            .map(|el| el.get_bounding_client_rect())
    }

    /// The map point under a client (page) point.
    pub fn to_map(&self, cx: f64, cy: f64) -> (f64, f64) {
        match self.rect() {
            Some(r) => self
                .view
                .get_untracked()
                .to_map(cx - r.left(), cy - r.top()),
            None => (0.0, 0.0),
        }
    }

    /// Fit a map of this size into the scene box.
    pub fn fit(&self, map_w: f64, map_h: f64) {
        if let Some(r) = self.rect() {
            self.view
                .set(View::fit(map_w, map_h, r.width(), r.height()));
        }
    }

    /// Zoom about the centre of the box.
    pub fn zoom(&self, factor: f64) {
        if let Some(r) = self.rect() {
            self.view
                .update(|v| *v = v.zoom_at(factor, r.width() / 2.0, r.height() / 2.0));
        }
    }

    pub fn down(&self, ev: &web_sys::PointerEvent) {
        if let Some(t) = ev
            .current_target()
            .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
        {
            let _ = t.set_pointer_capture(ev.pointer_id());
        }
        let view = self.view.get_untracked();
        let point = (f64::from(ev.client_x()), f64::from(ev.client_y()));
        self.gesture.update_value(|g| {
            g.pointers.insert(ev.pointer_id(), point);
            g.start_view = Some(view);
            g.start_points = g.pointers.values().copied().collect();
            g.moved = false;
        });
    }

    /// Pan or pinch with a move of a pointer that went down on the scene.
    pub fn moved(&self, ev: &web_sys::PointerEvent) {
        let point = (f64::from(ev.client_x()), f64::from(ev.client_y()));
        let rect = self.rect();
        let mut next = None;
        self.gesture.update_value(|g| {
            if !g.pointers.contains_key(&ev.pointer_id()) {
                return;
            }
            g.pointers.insert(ev.pointer_id(), point);
            let Some(start) = g.start_view else { return };
            let now: Vec<(f64, f64)> = g.pointers.values().copied().collect();
            if now.len() == 1 && g.start_points.len() == 1 {
                let (dx, dy) = (
                    now[0].0 - g.start_points[0].0,
                    now[0].1 - g.start_points[0].1,
                );
                if dx.abs() + dy.abs() > 3.0 {
                    g.moved = true;
                }
                next = Some(View {
                    tx: start.tx + dx,
                    ty: start.ty + dy,
                    ..start
                });
            } else if now.len() >= 2 && g.start_points.len() >= 2 {
                let d = |a: (f64, f64), b: (f64, f64)| {
                    ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt()
                };
                let (d0, d1) = (d(g.start_points[0], g.start_points[1]), d(now[0], now[1]));
                if let (true, Some(r)) = (d0 > 0.0, rect.as_ref()) {
                    g.moved = true;
                    let mid = ((now[0].0 + now[1].0) / 2.0, (now[0].1 + now[1].1) / 2.0);
                    next = Some(start.zoom_at(d1 / d0, mid.0 - r.left(), mid.1 - r.top()));
                }
            }
        });
        if let Some(v) = next {
            self.view.set(v);
        }
    }

    /// A pointer went up. True: it was a tap (one pointer, no movement).
    pub fn up(&self, ev: &web_sys::PointerEvent) -> bool {
        let view = self.view.get_untracked();
        let mut tapped = false;
        self.gesture.update_value(|g| {
            tapped = g.pointers.len() == 1 && !g.moved && g.pointers.contains_key(&ev.pointer_id());
            g.pointers.remove(&ev.pointer_id());
            g.start_view = Some(view);
            g.start_points = g.pointers.values().copied().collect();
        });
        tapped
    }

    pub fn wheel(&self, ev: &web_sys::WheelEvent) {
        ev.prevent_default();
        let Some(r) = self.rect() else { return };
        let factor = if ev.delta_y() < 0.0 { 1.15 } else { 1.0 / 1.15 };
        self.view.update(|v| {
            *v = v.zoom_at(
                factor,
                f64::from(ev.client_x()) - r.left(),
                f64::from(ev.client_y()) - r.top(),
            )
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zoom_keeps_the_point_under_the_cursor() {
        let v = View {
            scale: 1.0,
            tx: 10.0,
            ty: 20.0,
        };
        let before = v.to_map(110.0, 120.0);
        let z = v.zoom_at(2.0, 110.0, 120.0);
        assert_eq!(z.scale, 2.0);
        let after = z.to_map(110.0, 120.0);
        assert!((before.0 - after.0).abs() < 1e-9 && (before.1 - after.1).abs() < 1e-9);
        assert_eq!(v.zoom_at(1000.0, 0.0, 0.0).scale, MAX_SCALE);
        assert_eq!(v.zoom_at(0.0001, 0.0, 0.0).scale, MIN_SCALE);
    }

    #[test]
    fn fit_centres_the_whole_map() {
        let v = View::fit(2000.0, 1000.0, 1000.0, 1000.0);
        assert_eq!(v.scale, 0.5);
        assert_eq!((v.tx, v.ty), (0.0, 250.0));
        assert_eq!(View::fit(0.0, 10.0, 100.0, 100.0), View::IDENTITY);
    }
}
