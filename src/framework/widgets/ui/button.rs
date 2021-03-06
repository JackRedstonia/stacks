use super::super::{
    layout::{Margin, MarginContainer},
    shapes::Rectangle,
    FontName, FontStyle, Text, TextLayoutMode,
};
use crate::prelude::*;

pub struct Button {
    rect: Wrap<Rectangle>,
    label: Wrap<MarginContainer<Text>>,

    size: Size,
    glow: scalar,
    glow_paint: Paint,

    on_click_fns: Vec<Box<dyn FnMut()>>,
}

impl Button {
    const HOVER_GLOW: scalar = 0.3;
    const FULL_GLOW: scalar = 1.0;
    const GLOW_TO_ALPHA_MUL: scalar = 0.25;

    pub fn new(
        label: String,
        label_font: Option<FontName>,
        label_style: FontStyle,
        label_size: Option<scalar>,
        background: Paint,
        label_paint: Paint,
    ) -> Wrap<Self> {
        FrameworkState::request_load();

        let rect = Rectangle::new(
            LayoutSize::ZERO.expand_width().expand_height(),
            background,
        );
        let label_aa = label_paint.is_anti_alias();
        let label = Text::new(
            LayoutSize::ZERO,
            Some(TextLayoutMode::OneLine),
            label,
            label_font,
            label_style,
            label_size,
            label_paint,
        );
        let label = MarginContainer::new(label, Margin::all(10.0));
        Self {
            rect,
            label,
            size: Size::default(),
            glow: 0.0,
            glow_paint: Paint::new_color4f(1.0, 1.0, 1.0, 1.0)
                .with_anti_alias(label_aa),
            on_click_fns: vec![],
        }
        .wrap()
    }

    /// Adds a function that will be called whenever this button is pressed.
    /// Note that this should not be used to mutate widgets parenting this, as
    /// that obviously violates the exclusive mutability rule of RefCell.
    pub fn on_click<F: FnMut() + 'static>(&mut self, f: F) {
        self.on_click_fns.push(Box::new(f));
    }
}

impl Widget for Button {
    fn load(&mut self, _state: &mut WidgetState, stack: &mut ResourceStack) {
        self.rect.load(stack);
        self.label.load(stack);
    }

    fn update(&mut self, _state: &mut WidgetState) {
        self.rect.update();
        self.label.update();
    }

    fn input(&mut self, _state: &mut WidgetState, event: &InputEvent) -> bool {
        let r = Rect::from_size(self.size);
        match event {
            InputEvent::MouseMove(position) if r.contains(*position) => {
                return true;
            }
            InputEvent::MouseDown(MouseButton::Left, position)
                if r.contains(*position) =>
            {
                return true;
            }
            InputEvent::MouseUp(MouseButton::Left, position) => {
                if r.contains(*position) {
                    self.glow = Self::FULL_GLOW;
                    for f in &mut self.on_click_fns {
                        f();
                    }
                    return true;
                }
            }
            _ => {}
        }
        false
    }

    fn size(&mut self, _state: &mut WidgetState) -> (LayoutSize, bool) {
        self.label.size()
    }

    fn set_size(&mut self, _state: &mut WidgetState, size: Size) {
        self.size = size;
        self.label.set_size(size);
        self.rect.set_size(size);
    }

    fn draw(&mut self, state: &mut WidgetState, canvas: &mut Canvas) {
        self.rect.draw(canvas);

        let t = State::last_update_time_draw().as_secs_f32() * 4.0;
        if state.is_hovered() {
            if self.glow > Self::HOVER_GLOW {
                self.glow = (self.glow - t).max(Self::HOVER_GLOW);
            } else {
                self.glow = (self.glow + t).min(Self::HOVER_GLOW);
            }
        } else {
            self.glow = (self.glow - t).max(0.0);
        }
        self.glow_paint
            .set_alpha_f(self.glow * Self::GLOW_TO_ALPHA_MUL);
        canvas.draw_rect(Rect::from_size(self.size), &self.glow_paint);

        self.label.draw(canvas);
    }
}
