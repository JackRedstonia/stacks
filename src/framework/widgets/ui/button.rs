use super::super::{
    backgrounded::Backgrounded,
    layout::{Margin, MarginContainer},
    shapes::Rectangle,
    Font, FontStyle, Text, TextLayoutMode,
};
use crate::prelude::*;

pub struct Button {
    background: Wrap<Rectangle>,
    label: Wrap<MarginContainer>,
    glow: scalar,
    glow_paint: Paint,
    size: Size,

    on_click_fns: Vec<Box<dyn FnMut()>>,
}

impl Button {
    pub fn new(
        label: String,
        label_size: Option<scalar>,
        background: Paint,
        label_paint: Paint,
    ) -> Wrap<Self> {
        let background = Rectangle::new(
            LayoutSize::ZERO.expand_width().expand_height(),
            background,
        );
        let label_aa = label_paint.is_anti_alias();
        let label = ButtonLabel::new(label, label_size, label_paint);
        let label = MarginContainer::new(Margin::all(15.0)).with_child(label);
        Self {
            background: background.clone(),
            label: label.clone(),
            glow: 0.0,
            glow_paint: Paint::new_color4f(1.0, 1.0, 1.0, 1.0)
                .with_anti_alias(label_aa),
            size: Size::default(),
            on_click_fns: vec![],
        }
        .wrap()
        .with_child(
            Backgrounded::new().with_child(background).with_child(label),
        )
    }

    /// Adds a function that will be called whenever this button is pressed.
    /// Note that this should not be used to mutate widgets parenting this, as
    /// that obviously violates the exclusive mutability rule of RefCell.
    pub fn on_click<F: FnMut() + 'static>(&mut self, f: F) {
        self.on_click_fns.push(Box::new(f));
    }
}

impl Widget for Button {
    fn input(&mut self, state: &mut WidgetState, event: &InputEvent) -> bool {
        let r = Rect::from_size(self.size);
        match event {
            InputEvent::MouseDown(MouseButton::Left, position)
                if r.contains(*position) =>
            {
                state.grab_focus();
                return true;
            }
            InputEvent::MouseUp(MouseButton::Left, position) => {
                state.release_focus();
                if r.contains(*position) {
                    self.glow = 1.0;
                    for f in &mut self.on_click_fns {
                        f();
                    }
                    return true;
                }
            }
            _ => {}
        }
        state.child().map(|e| e.input(event)).unwrap_or(false)
    }

    fn size(&mut self, state: &mut WidgetState) -> (LayoutSize, bool) {
        state.child().map(|e| e.size()).unwrap_or_default()
    }

    fn set_size(&mut self, state: &mut WidgetState, size: Size) {
        self.size = size;
        if let Some(child) = state.child() {
            child.set_size(size);
        }
    }

    fn draw(&mut self, _state: &mut WidgetState, canvas: &mut Canvas) {
        self.background.draw(canvas);

        let t = State::last_update_time_draw().as_secs_f32() * 4.0;
        self.glow = (self.glow - t).max(0.0);
        self.glow_paint.set_alpha_f(self.glow * 0.25);
        canvas.draw_rect(Rect::from_size(self.size), &self.glow_paint);

        self.label.draw(canvas);
    }
}

struct ButtonLabel {
    label: Wrap<Text>,
}

impl ButtonLabel {
    fn new(
        label: String,
        label_size: Option<scalar>,
        label_paint: Paint,
    ) -> Wrap<Self> {
        let l = Text::new(
            LayoutSize::ZERO.expand_width().expand_height(),
            Some(TextLayoutMode::OneLine),
            label,
            Font::Default,
            FontStyle::Regular,
            label_size,
            label_paint,
        );
        Self { label: l.clone() }.wrap().with_child(l)
    }
}

impl Widget for ButtonLabel {
    fn set_size(&mut self, _state: &mut WidgetState, size: Size) {
        self.label.set_size(size)
    }

    fn size(&mut self, _state: &mut WidgetState) -> (LayoutSize, bool) {
        let s = self.label.inner().bounds().size();
        (LayoutSize::min(s.width, s.height), false)
    }

    fn draw(&mut self, state: &mut WidgetState, canvas: &mut Canvas) {
        if let Some(child) = state.child() {
            child.draw(canvas);
        }
    }
}
