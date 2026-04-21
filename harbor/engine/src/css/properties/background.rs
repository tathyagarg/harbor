use crate::css::properties::{Image, Origin, PositionValue, RepeatStyle, prop_imports};

prop_imports!();

#[derive(Default, Debug, Clone)]
pub struct Background {
    pub layers: Vec<BackgroundLayer>,
}

#[derive(Debug, Clone)]
pub struct BackgroundLayer {
    pub image: Image,
    pub color: Color,
    pub position: PositionValue,
    pub repeat_style: RepeatStyle,
    pub origin: Origin,
}

impl Default for BackgroundLayer {
    fn default() -> Self {
        BackgroundLayer {
            image: Image::None,
            color: Color::transparent(),
            position: PositionValue::default(),
            repeat_style: RepeatStyle::Repeat,
            origin: Origin::PaddingBox,
        }
    }
}

impl CSSParseable for Background {
    fn from_cv(cvs: &mut InputStream<ComponentValue>) -> Option<Self>
    where
        Self: Sized,
    {
        let mut bg = Background { layers: Vec::new() };

        let vec = cvs.finish();

        let bg_layers = Background::preprocess_tokens(&vec);

        for (i, layer) in bg_layers.iter().enumerate() {
            if i == bg_layers.len() - 1 {
                let mut layer_cvs = InputStream::new(layer);

                if let Some(parsed_layer) = BackgroundLayer::parse_bg_layer(&mut layer_cvs, true) {
                    bg.layers.push(parsed_layer);
                }
            } else {
                let mut layer_cvs = InputStream::new(layer);

                if let Some(parsed_layer) = BackgroundLayer::parse_bg_layer(&mut layer_cvs, false) {
                    bg.layers.push(parsed_layer);
                }
            }
        }

        Some(bg)
    }
}

impl Background {
    pub fn preprocess_tokens(cvs: &[ComponentValue]) -> Vec<Vec<ComponentValue>> {
        cvs.iter()
            .filter(|cv| match cv {
                ComponentValue::Token(token) => match token {
                    CSSToken::Whitespace => false,
                    _ => true,
                },
                _ => true,
            })
            .cloned()
            .collect::<Vec<_>>()
            .split(|cv| match cv {
                ComponentValue::Token(token) => match token {
                    CSSToken::Comma => true,
                    _ => false,
                },
                _ => false,
            })
            .map(|slice| slice.to_vec())
            .collect()
    }

    pub fn color(&self) -> Color {
        self.layers
            .last()
            .map_or(Color::transparent(), |layer| layer.color.clone())
    }

    fn ensure_layer(&mut self) -> &mut BackgroundLayer {
        if self.layers.is_empty() {
            self.layers.push(BackgroundLayer::default());
        }
        self.layers.last_mut().unwrap()
    }

    fn update_color(&mut self, color: Color) {
        let layer = self.ensure_layer();
        layer.color = color;
    }

    pub fn set_color(&mut self, color: Color) {
        let layer = self.ensure_layer();
        layer.color = color;
    }

    pub fn set_images(&mut self, images: Vec<Image>) {
        for (i, image) in images.into_iter().enumerate() {
            if i < self.layers.len() {
                self.layers[i].image = image;
            } else {
                let mut layer = BackgroundLayer::default();
                layer.image = image;
                self.layers.push(layer);
                self.update_color(self.color());
            }
        }
    }

    pub fn set_positions(&mut self, positions: Vec<PositionValue>) {
        for (i, position) in positions.into_iter().enumerate() {
            if i < self.layers.len() {
                self.layers[i].position = position;
            } else {
                let mut layer = BackgroundLayer::default();
                layer.position = position;
                self.layers.push(layer);
                self.update_color(self.color());
            }
        }
    }

    pub fn set_repeat_styles(&mut self, repeat_styles: Vec<RepeatStyle>) {
        for (i, repeat_style) in repeat_styles.into_iter().enumerate() {
            if i < self.layers.len() {
                self.layers[i].repeat_style = repeat_style;
            } else {
                let mut layer = BackgroundLayer::default();
                layer.repeat_style = repeat_style;
                self.layers.push(layer);
                self.update_color(self.color());
            }
        }
    }

    pub fn set_origins(&mut self, origins: Vec<Origin>) {
        for (i, origin) in origins.into_iter().enumerate() {
            if i < self.layers.len() {
                self.layers[i].origin = origin;
            } else {
                let mut layer = BackgroundLayer::default();
                layer.origin = origin;
                self.layers.push(layer);
                self.update_color(self.color());
            }
        }
    }
}

impl BackgroundLayer {
    fn parse_bg_layer(
        cvs: &mut InputStream<ComponentValue>,
        is_final: bool,
    ) -> Option<BackgroundLayer> {
        let mut layer = BackgroundLayer::default();

        while !cvs.is_eof {
            if let Some(image) = Image::from_cv(cvs) {
                layer.image = image;
                continue;
            }

            if let Some(position) = PositionValue::from_cv(cvs) {
                layer.position = position;
                continue;
            }

            if let Some(repeat_style) = RepeatStyle::from_cv(cvs) {
                layer.repeat_style = repeat_style;
                continue;
            }

            if let Some(origin) = Origin::from_cv(cvs) {
                layer.origin = origin;
                continue;
            }

            if is_final {
                if let Some(color) = Color::from_cv(cvs) {
                    layer.color = color;
                    continue;
                }
            }
        }

        Some(layer)
    }
}
