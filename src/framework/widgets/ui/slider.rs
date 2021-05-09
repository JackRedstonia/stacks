use super::super::{
    layout::{CenterContainer, Margin, MarginContainer},
    shapes::Rectangle,
    Font, FontStyle, Text, TextLayoutMode,
};
use crate::prelude::*;

type SliderLabelFormatter = dyn for<'r> FnMut(&'r str, scalar) -> String;

// #[derive(Clone, Copy, Hash, PartialEq, Eq)]
// pub enum SliderChangeTrigger {
//     OnMove,
//     OnRelease,
// }

pub struct Slider {
    background: Wrap<Rectangle>,
    background_size: LayoutSize,
    button: Wrap<Rectangle>,
    button_layout_size: LayoutSize,
    button_size: Size,
    label: Wrap<CenterContainer<MarginContainer<Text>>>,
    label_text: String,
    label_size: LayoutSize,
    label_inner: Wrap<Text>,

    layout_width: LayoutDimension,
    size: Size,

    on_change_fns: Vec<Box<dyn FnMut(scalar)>>,
    formatter: Box<SliderLabelFormatter>,

    value: scalar,
    button_offset: scalar,
}

impl Slider {
    pub fn new(
        label: String,
        label_size: Option<scalar>,
        layout_width: LayoutDimension,
        background: Paint,
        button_paint: Paint,
        label_paint: Paint,
    ) -> Wrap<Self> {
        let label_text = label.clone();
        let label = Text::new(
            LayoutSize::ZERO,
            Some(TextLayoutMode::OneLine),
            label,
            Font::Default,
            FontStyle::Regular,
            label_size,
            label_paint,
        );
        let label_inner = label.clone();
        let label = CenterContainer::new(MarginContainer::new(
            label,
            Margin::vertical(10.0),
        ));

        let formatter: Box<SliderLabelFormatter> =
            Box::new(|label, value| format!("{}: {}", label, value));

        Self {
            background: Rectangle::new(
                LayoutSize::ZERO.expand_width().expand_height(),
                background,
            ),
            background_size: LayoutSize::default(),
            button: Rectangle::new(
                LayoutSize::min(15.0, 0.0).expand_height(),
                button_paint,
            ),
            button_layout_size: LayoutSize::default(),
            button_size: Size::default(),
            label,
            label_text,
            label_size: LayoutSize::default(),
            label_inner,
            layout_width,
            size: Size::default(),
            on_change_fns: vec![],
            formatter,
            value: 0.0,
            button_offset: 0.0,
        }
        .wrap()
    }

    pub fn on_change<F: FnMut(scalar) + 'static>(&mut self, f: F) {
        self.on_change_fns.push(Box::new(f));
    }

    pub fn set_formatter<F: 'static + FnMut(&str, scalar) -> String>(
        &mut self,
        f: F,
    ) {
        self.formatter = Box::new(f);
        self.slide_to_val(self.value);
    }

    fn slide_to(&mut self, pos: Vector) {
        let width = self.size.width - self.button_size.width;
        let x = (pos.x - self.button_size.width * 0.5).clamp(0.0, width);
        let pos = x / width;
        // LINT SUPPRESSION: For the sake of absolute correctness, we do indeed
        // want strict comparison here.
        #[allow(clippy::float_cmp)]
        if self.value != pos {
            self.slide_to_val(pos);
            self.button_offset = x.round();
        }
    }

    fn slide_to_val(&mut self, val: scalar) {
        for f in &mut self.on_change_fns {
            f(val);
        }
        self.label_inner
            .inner_mut()
            .set_text(self.formatter.as_mut()(&self.label_text, val));
        self.label_inner.inner_mut().force_build_paragraph();
        self.value = val;
    }
}

impl Widget for Slider {
    fn load(&mut self, _state: &mut WidgetState, stack: &mut ResourceStack) {
        self.background.load(stack);
        self.button.load(stack);
        self.label.load(stack);
        self.slide_to_val(self.value);
    }

    fn update(&mut self, _state: &mut WidgetState) {
        self.background.update();
        self.button.update();
        self.label.update();
    }

    fn input(&mut self, state: &mut WidgetState, event: &InputEvent) -> bool {
        match event {
            InputEvent::MouseUp(MouseButton::Left, pos) => {
                if state.is_focused() {
                    state.release_focus();
                    self.slide_to(*pos);
                }
                Rect::from_size(self.size).contains(*pos)
            }
            InputEvent::MouseDown(MouseButton::Left, pos) => {
                let c = Rect::from_size(self.size).contains(*pos);
                if c {
                    state.grab_focus();
                    self.slide_to(*pos);
                }
                c
            }
            InputEvent::MouseMove(pos) => {
                if state.is_focused() {
                    self.slide_to(*pos);
                }
                Rect::from_size(self.size).contains(*pos)
            }
            _ => false,
        }
    }

    fn size(&mut self, _state: &mut WidgetState) -> (LayoutSize, bool) {
        let (bgz, a) = self.background.size();
        let ac = self.background_size == bgz;
        self.background_size = bgz;
        let (bz, b) = self.button.size();
        let bc = self.button_layout_size == bz;
        self.button_layout_size = bz;
        let (mut lz, c) = self.label.size();
        lz.width = self.layout_width;
        let cc = self.label_size == lz;
        self.label_size = lz;

        (lz, a || ac || b || bc || c || cc)
    }

    fn set_size(&mut self, _state: &mut WidgetState, size: Size) {
        self.background
            .set_size(self.background_size.layout_one(size));
        let button_size = self.button_layout_size.layout_one(size);
        self.button.set_size(button_size);
        self.button_size = button_size;
        self.label.set_size(self.label_size.layout_one(size));

        self.size = size;
    }

    fn draw(&mut self, _state: &mut WidgetState, canvas: &mut Canvas) {
        self.background.draw(canvas);
        canvas.save();
        canvas.translate((self.button_offset, 0.0));
        self.button.draw(canvas);
        canvas.restore();
        self.label.draw(canvas);
    }
}
