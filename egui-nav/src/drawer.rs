use egui::{LayerId, Order};

use crate::{render_bg, render_fg, Drag, DragDirection, NavAction, State};

pub struct NavDrawer<'a, Route: Clone> {
    id_source: Option<egui::Id>,
    bg_route: &'a Route,
    drawer_route: &'a Route,
    drawer_end_offset: f32,
    navigating: bool,
    returning: bool,
    drawer_focused: bool,
}

impl<'a, Route: Clone> NavDrawer<'a, Route> {
    pub fn new(bg_route: &'a Route, drawer_route: &'a Route) -> Self {
        Self {
            id_source: None,
            bg_route,
            drawer_route,
            drawer_end_offset: 0.0,
            navigating: false,
            returning: false,
            drawer_focused: false,
        }
    }

    pub fn opened_offset(mut self, drawer_end_x: f32) -> Self {
        self.drawer_end_offset = drawer_end_x;
        self
    }

    pub fn id_source(mut self, id: egui::Id) -> Self {
        self.id_source = Some(id);
        self
    }

    /// Call this when you have just pushed a new value to your route and
    /// you want to animate to this new view
    pub fn navigating(mut self, navigating: bool) -> Self {
        self.navigating = navigating;
        self
    }

    /// Call this when you have just invoked an action to return to the
    /// previous view
    pub fn returning(mut self, returning: bool) -> Self {
        self.returning = returning;
        self
    }

    pub fn drawer_focused(mut self, focused: bool) -> Self {
        self.drawer_focused = focused;
        self
    }

    fn id(&self, ui: &egui::Ui) -> egui::Id {
        ui.id().with(("nav-drawer", self.id_source))
    }

    pub fn drag_id(&self, ui: &egui::Ui) -> egui::Id {
        self.id(ui).with("drag")
    }

    pub fn show<F, R>(&self, ui: &mut egui::Ui, show_route: F) -> DrawerResponse<R>
    where
        F: Fn(&mut egui::Ui, &Route) -> R,
    {
        let mut show_route = show_route;

        self.show_internal(ui, &mut show_route)
    }

    pub fn show_mut<F, R>(&self, ui: &mut egui::Ui, mut show_route: F) -> DrawerResponse<R>
    where
        F: FnMut(&mut egui::Ui, &Route) -> R,
    {
        self.show_internal(ui, &mut show_route)
    }

    fn show_internal<F, R>(&self, ui: &mut egui::Ui, show_route: &mut F) -> DrawerResponse<R>
    where
        F: FnMut(&mut egui::Ui, &Route) -> R,
    {
        let id = self.id(ui);
        let mut state = State::load(ui.ctx(), id).unwrap_or_default();

        let get_rest_or_max = |get_rest| {
            if get_rest {
                0.0
            } else {
                self.drawer_end_offset
            }
        };
        let rest_offset = if self.drawer_focused {
            get_rest_or_max(false)
        } else {
            get_rest_or_max(true)
        };

        let (drawer_rect, bg_rect) = ui
            .available_rect_before_wrap()
            .split_left_right_at_x(state.offset);

        let drag_content_rect = if drawer_rect.width() > 0.0 {
            drawer_rect
        } else {
            ui.available_rect_before_wrap()
        };

        let mut drag = Drag::new(
            self.drag_id(ui),
            DragDirection::LeftToRight,
            drag_content_rect,
            state.offset - rest_offset,
        );

        if let Some(action) = drag.handle(ui) {
            state.action = Some(action);
        }

        if self.navigating {
            if state.action != Some(NavAction::Navigating) {
                state.offset = rest_offset;
                state.action = Some(NavAction::Navigating);
            }
        } else if self.returning && !matches!(state.action, Some(NavAction::Returning(_))) {
            state.offset = rest_offset;
            state.action = Some(NavAction::Returning(crate::ReturnType::Click));
        }

        let max_offset = if self.drawer_focused {
            get_rest_or_max(true)
        } else {
            get_rest_or_max(false)
        };

        if let Some(action) = state.action {
            action.handle(
                ui,
                &mut state,
                DragDirection::LeftToRight,
                rest_offset,
                max_offset,
            );
        }

        if state.offset == rest_offset {
            show_route(ui, self.bg_route);
            state.store(ui.ctx(), id);
            return DrawerResponse {
                drawer_response: None,
                action: state.action,
            };
        }

        let avail_rect = ui.available_rect_before_wrap();
        let alpha = if state.offset <= rest_offset {
            None
        } else {
            let t = ((max_offset - state.offset) / rest_offset)
                .abs()
                .clamp(0.0, 1.0);
            Some((t * 200.0).round() as u8)
        };

        let _ = render_bg(ui, None, bg_rect, avail_rect, alpha, |ui| {
            show_route(ui, self.bg_route);
        });
        let bg_resp = ui.allocate_rect(bg_rect, egui::Sense::click());

        if bg_resp.clicked() {
            state.action = Some(NavAction::Returning(crate::ReturnType::Click));
        }

        let drawer_response = Some(render_fg(
            ui,
            id.with("fg"),
            LayerId::new(Order::Foreground, id.with("fg")),
            None,
            drawer_rect,
            drawer_rect,
            |ui| show_route(ui, self.drawer_route),
        ));

        state.store(ui.ctx(), id);

        DrawerResponse {
            drawer_response,
            action: state.action,
        }
    }
}

pub struct DrawerResponse<R> {
    pub drawer_response: Option<R>,
    pub action: Option<NavAction>,
}
