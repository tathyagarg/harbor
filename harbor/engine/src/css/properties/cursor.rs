use winit::window::CursorIcon as WCursor;

use crate::css::properties::prop_imports;

prop_imports!();

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Cursor {
    #[default]
    Auto,
    Default,
    None,
    ContextMenu,
    Help,
    Pointer,
    Progress,
    Wait,
    Cell,
    Crosshair,
    Text,
    VerticalText,
    Alias,
    Copy,
    Move,
    NoDrop,
    NotAllowed,
}

impl CSSParseable for Cursor {
    fn from_cv(cvs: &mut InputStream<ComponentValue>) -> Option<Self>
    where
        Self: Sized,
    {
        if let Some(tok) = cvs.consume() {
            match tok {
                ComponentValue::Token(CSSToken::Ident(ident)) => match ident.as_str() {
                    "auto" => return Some(Cursor::Auto),
                    "default" => return Some(Cursor::Default),
                    "none" => return Some(Cursor::None),
                    "context-menu" => return Some(Cursor::ContextMenu),
                    "help" => return Some(Cursor::Help),
                    "pointer" => return Some(Cursor::Pointer),
                    "progress" => return Some(Cursor::Progress),
                    "wait" => return Some(Cursor::Wait),
                    "cell" => return Some(Cursor::Cell),
                    "crosshair" => return Some(Cursor::Crosshair),
                    "text" => return Some(Cursor::Text),
                    "vertical-text" => return Some(Cursor::VerticalText),
                    "alias" => return Some(Cursor::Alias),
                    "copy" => return Some(Cursor::Copy),
                    "move" => return Some(Cursor::Move),
                    "no-drop" => return Some(Cursor::NoDrop),
                    "not-allowed" => return Some(Cursor::NotAllowed),
                    _ => {
                        todo!("Handle more cursor values")
                    }
                },
                _ => {}
            }
        }
        cvs.reconsume();
        None
    }
}

impl Resolvable<WCursor> for Cursor {
    fn resolve(&mut self, _parents: &Vec<Rc<RefCell<Element>>>) -> WCursor {
        match self {
            Cursor::Auto => WCursor::Default,
            Cursor::Default => WCursor::Default,
            Cursor::ContextMenu => WCursor::ContextMenu,
            Cursor::Help => WCursor::Help,
            Cursor::Pointer => WCursor::Pointer,
            Cursor::Progress => WCursor::Progress,
            Cursor::Wait => WCursor::Wait,
            Cursor::Cell => WCursor::Cell,
            Cursor::Crosshair => WCursor::Crosshair,
            Cursor::Text => WCursor::Text,
            Cursor::VerticalText => WCursor::VerticalText,
            Cursor::Alias => WCursor::Alias,
            Cursor::Copy => WCursor::Copy,
            Cursor::Move => WCursor::Move,
            Cursor::NoDrop => WCursor::NoDrop,
            Cursor::NotAllowed => WCursor::NotAllowed,
            Cursor::None => WCursor::Default, // TODO: Implement None cursor
        }
    }

    fn resolve_with_curr(
        &mut self,
        parents: &Vec<Rc<RefCell<Element>>>,
        _current: &ComputedStyle,
        _viewport_size: (f64, f64),
    ) -> WCursor {
        self.resolve(parents)
    }

    fn resolved(&self) -> WCursor {
        self.clone().resolve(&vec![])
    }
}
