use egui::Pos2;

use crate::{NavAction, ReturnType};

#[derive(Clone, PartialEq, Eq, Debug)]
pub(crate) enum DragDirection {
    LeftToRight,
    RightToLeft,
    Vertical,
}

pub(crate) struct Drag {
    pub(crate) id: egui::Id,
    content_rect: egui::Rect,
    direction: DragDirection,
    offset_from_rest: f32,
    found_capture: bool,
}

impl Drag {
    pub(crate) fn new(
        id: egui::Id,
        direction: DragDirection,
        content_rect: egui::Rect,
        offset_from_rest: f32,
    ) -> Self {
        Drag {
            id,
            content_rect,
            direction,
            offset_from_rest,
            found_capture: false,
        }
    }

    fn content_size(&self) -> f32 {
        match self.direction {
            DragDirection::LeftToRight | DragDirection::RightToLeft => self.content_rect.width(),
            DragDirection::Vertical => self.content_rect.height(),
        }
    }

    pub(crate) fn handle(&mut self, ui: &mut egui::Ui) -> Option<NavAction> {
        if ui.ctx().drag_stopped_id().is_some() {
            remove_state(ui.ctx());

            // we've stopped dragging, check to see if the offset is
            // passed a certain point, to determine if we should return
            // or animate back
            return if self.offset_from_rest > self.content_size() / 4.0 {
                Some(NavAction::Returning(ReturnType::Drag))
            } else {
                if self.offset_from_rest > 0.0 {
                    Some(NavAction::Resetting)
                } else {
                    None
                }
            };
        }

        if ui.ctx().dragged_id().is_none() {
            return None;
        }

        let start_state = match get_state(ui.ctx()) {
            Some(state) => state,
            None => self.insert_new_state(ui.ctx())?,
        };

        if !self.content_rect.contains(start_state.start_pos) {
            // the start position isn't in the content rect, the interaction doesn't pertain to this widget
            return None;
        }

        let cur_pos = ui.ctx().pointer_latest_pos()?;
        let cur_direction = cur_direction(start_state.start_pos, cur_pos);

        if cur_direction != self.direction {
            // the direction isn't desired for this widget, reset
            return if self.offset_from_rest > 0.0 {
                Some(NavAction::Resetting)
            } else {
                None
            };
        }

        // // the current direction is correct! set it to capture
        self.found_capture = true;

        // Drag contents to transition back.
        // We must do this BEFORE adding content to the `Nav`,
        // or we will steal input from the widgets we contain.
        Some(NavAction::Dragging)
    }

    pub(crate) fn should_capture(&self, ctx: &egui::Context) -> Option<CaptureAction> {
        let capture_drag =
            ctx.dragged_id().is_none() && ctx.input(|i| i.pointer.is_decidedly_dragging());
        // we are dragging, but the drag id has not been set. This indicates the fg widget doesn't care about dragging, so we should capture it

        if self.found_capture {
            println!("Capture due to Drag::handle");
        }

        if capture_drag {
            println!("Capture due to no fg drag");
        }

        if self.found_capture || capture_drag {
            Some(CaptureAction {
                also_capture_drag_id: capture_drag,
            })
        } else {
            None
        }
    }

    fn insert_state(&mut self, ctx: &egui::Context, state: DragState) {
        ctx.data_mut(|d| d.insert_temp(state_id(), state));
    }

    fn insert_new_state(&mut self, ctx: &egui::Context) -> Option<DragState> {
        let cur_pos = ctx.pointer_latest_pos()?;
        let new_state = DragState { start_pos: cur_pos };
        self.insert_state(ctx, new_state.clone());
        Some(new_state)
    }
}

pub struct CaptureAction {
    also_capture_drag_id: bool,
}

impl CaptureAction {
    pub fn capture(self, ui: &mut egui::Ui, id: egui::Id, rect: egui::Rect) {
        if !ui.ctx().input(|i| i.pointer.primary_down()) {
            ui.ctx().stop_dragging();
            return;
        }

        println!("CAPTURING");

        let _ = ui.interact(rect, id, egui::Sense::drag());
        if self.also_capture_drag_id {
            ui.ctx().set_dragged_id(id);
        }
    }
}

fn state_id() -> egui::Id {
    egui::Id::new("nav-drag-state")
}

pub fn get_state(ctx: &egui::Context) -> Option<DragState> {
    let id = state_id();
    ctx.data(|d| d.get_temp(id))
}

fn remove_state(ctx: &egui::Context) {
    ctx.data_mut(|d| d.remove::<DragState>(state_id()));
}

#[derive(Clone, Debug)]
pub struct DragState {
    pub(crate) start_pos: Pos2,
}

fn cur_direction(start: Pos2, cur_pos: Pos2) -> DragDirection {
    let dx = start.x - cur_pos.x;
    let dy = start.y - cur_pos.y;

    if dy.abs() > dx.abs() {
        DragDirection::Vertical
    } else if dx >= 0.0 {
        DragDirection::RightToLeft
    } else {
        DragDirection::LeftToRight
    }
}

pub(crate) fn drag_delta(ui: &mut egui::Ui, direction: DragDirection) -> f32 {
    match direction {
        DragDirection::LeftToRight | DragDirection::RightToLeft => {
            ui.input(|input| input.pointer.delta()).x
        }
        DragDirection::Vertical => ui.input(|input| input.pointer.delta()).y,
    }
}
