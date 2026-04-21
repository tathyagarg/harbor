use crate::css::properties::{parse_url_function, prop_imports};

prop_imports!();

#[derive(Debug, Clone)]
pub enum Image {
    FromUrl(String),
    None,
}

impl CSSParseable for Image {
    fn from_cv(cvs: &mut InputStream<ComponentValue>) -> Option<Self>
    where
        Self: Sized,
    {
        match cvs.peek() {
            Some(tok) => match tok {
                ComponentValue::Token(CSSToken::Ident(ident)) if ident == "none" => {
                    cvs.consume();
                    Some(Image::None)
                }
                _ => Image::parse_definite_image(cvs),
            },
            None => None,
        }
    }
}

impl Image {
    fn parse_definite_image(cvs: &mut InputStream<ComponentValue>) -> Option<Image> {
        if let Some(url) = parse_url_function(cvs) {
            return Some(Image::FromUrl(url));
        }

        None
    }

    pub fn parse_multiple_images(cvs: &mut InputStream<ComponentValue>) -> Vec<Image> {
        let mut cvs = InputStream::new(
            &cvs.finish()
                .iter()
                .filter(|cv| match cv {
                    ComponentValue::Token(token) => match token {
                        CSSToken::Whitespace | CSSToken::Comma => false,
                        _ => true,
                    },
                    _ => true,
                })
                .cloned()
                .collect::<Vec<ComponentValue>>()[..],
        );

        let mut images = Vec::new();

        while let Some(image) = Image::from_cv(&mut cvs) {
            images.push(image);
        }

        images
    }
}
