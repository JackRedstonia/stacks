use crate::prelude::*;

pub struct Backgrounded<B: Widget, F: Widget> {
    background: Wrap<B>,
    foreground: Wrap<F>,
    background_size: LayoutSize,
    foreground_size: LayoutSize,
    allow_background_input: bool,
}

impl<B: Widget, F: Widget> Backgrounded<B, F> {
    pub fn new(
        background: Wrap<B>,
        foreground: Wrap<F>,
        allow_background_input: bool,
    ) -> Wrap<Self> {
        Self {
            background,
            foreground,
            background_size: LayoutSize::default(),
            foreground_size: LayoutSize::default(),
            allow_background_input,
        }
        .into()
    }
}

impl<B: Widget, F: Widget> Widget for Backgrounded<B, F> {
    fn input(&mut self, state: &mut WidgetState, event: &InputEvent) -> bool {
        let b = self.allow_background_input
            && !event.is_consumable()
            && self.background.input(event);
        self.foreground.input(event) || b
    }

    fn size(&mut self, state: &mut WidgetState) -> (LayoutSize, bool) {
        let (b, bc) = self.background.size();
        let (f, fc) = self.foreground.size();
        self.background_size = b;
        self.foreground_size = f;

        (self.foreground_size, bc || fc)
    }

    fn set_size(&mut self, state: &mut WidgetState, size: Size) {
        self.background
            .set_size(self.background_size.layout_one(size));
        self.foreground
            .set_size(self.foreground_size.layout_one(size));
    }

    fn draw(&mut self, state: &mut WidgetState, canvas: &mut Canvas) {
        self.background.draw(canvas);
        self.foreground.draw(canvas);
    }

    fn load(&mut self, state: &mut WidgetState, stack: &mut ResourceStack) {
        self.background.load(stack);
        self.foreground.load(stack);
    }

    fn update(&mut self, state: &mut WidgetState) {
        self.background.update();
        self.foreground.update();
    }
}
