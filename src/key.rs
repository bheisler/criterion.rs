use std::str::MaybeOwned;

use display::Display;
use Script;

#[deriving(Clone)]
pub struct Properties {
    boxed: bool,
    hidden: bool,
    justification: Option<Justification>,
    order: Option<Order>,
    position: Option<Position>,
    stack: Option<Stack>,
    title: Option<MaybeOwned<'static>>,
}

impl Properties {
    // NB I dislike the visibility rules within the same crate
    #[doc(hidden)]
    pub fn _new() -> Properties {
        Properties {
            boxed: false,
            hidden: false,
            justification: None,
            order: None,
            position: None,
            stack: None,
            title: None,
        }
    }

    /// Surrounds the key with a box
    ///
    /// **Note** The key is unboxed by default
    pub fn boxed(&mut self) -> &mut Properties {
        self.boxed = true;
        self
    }

    /// Hides the key
    pub fn hide(&mut self) -> &mut Properties {
        self.hidden = true;
        self
    }

    /// Changes the justification of the text of each entry
    ///
    /// **Note** The text is `RightJustified` by default
    pub fn justification(&mut self, justification: Justification) -> &mut Properties {
        self.justification = Some(justification);
        self
    }

    /// How to order each entry
    ///
    /// **Note** The default order is `TextSample`
    pub fn order(&mut self, order: Order) -> &mut Properties {
        self.order = Some(order);
        self
    }

    /// Selects where to place the key
    ///
    /// **Note** By default, the key is placed `Inside(Top, Right)`
    pub fn position(&mut self, position: Position) -> &mut Properties {
        self.position = Some(position);
        self
    }

    /// Shows the key
    ///
    /// **Note** The key is shown by default
    pub fn show(&mut self) -> &mut Properties {
        self.hidden = false;
        self
    }

    /// Changes how the entries of the key are stacked
    pub fn stack(&mut self, stack: Stack) -> &mut Properties {
        self.stack = Some(stack);
        self
    }

    /// Sets the title of the key
    pub fn title<S: IntoMaybeOwned<'static>>(&mut self, title: S) -> &mut Properties {
        self.title = Some(title.into_maybe_owned());
        self
    }
}

#[doc(hidden)]
impl Script for Properties {
    fn script(&self) -> String {
        let mut script = if self.hidden {
            return "set key off\n".to_string();
        } else {
            "set key on ".to_string()
        };

        match self.position {
            None => {},
            Some(Inside(v, h)) => {
                script.push_str(format!("inside {} {} ", v.display(), h.display()).as_slice())
            },
            Some(Outside(v, h)) => {
                script.push_str(format!("outside {} {} ", v.display(), h.display()).as_slice())
            },
        }

        match self.stack {
            Some(stack) => {
                script.push_str(stack.display());
                script.push(' ');
            },
            None => {},
        }

        match self.justification {
            Some(justification) => {
                script.push_str(justification.display());
                script.push(' ');
            },
            None => {},
        }

        match self.order {
            Some(order) => {
                script.push_str(order.display());
                script.push(' ');
            },
            None => {},
        }

        match self.title {
            Some(ref title) => script.push_str(format!("title '{}' ", title).as_slice()),
            None => {},
        }

        if self.boxed {
            script.push_str("box ")
        }

        script.push('\n');
        script
    }
}

#[deriving(Clone)]
pub enum HorizontalPosition {
    Center,
    Left,
    Right,
}

#[doc(hidden)]
impl Display<&'static str> for HorizontalPosition {
    fn display(&self) -> &'static str {
        match *self {
            Center => "center",
            Left => "left",
            Right => "right",
        }
    }
}

#[deriving(Clone)]
pub enum Justification {
    LeftJustified,
    RightJustified,
}

impl Display<&'static str> for Justification {
    fn display(&self) -> &'static str {
        match *self {
            LeftJustified => "Left",
            RightJustified => "Rigth",
        }
    }
}

#[deriving(Clone)]
pub enum Order {
    SampleText,
    TextSample,
}

#[doc(hidden)]
impl Display<&'static str> for Order {
    fn display(&self) -> &'static str {
        match *self {
            TextSample => "noreverse",
            SampleText => "reverse",
        }
    }
}

// TODO XY position
#[deriving(Clone)]
pub enum Position {
    Inside(VerticalPosition, HorizontalPosition),
    Outside(VerticalPosition, HorizontalPosition),
}

/// How the entries of the key are stacked
#[deriving(Clone)]
pub enum Stack {
    Horizontal,
    Vertical,
}

#[doc(hidden)]
impl Display<&'static str> for Stack {
    fn display(&self) -> &'static str {
        match *self {
            Horizontal => "horizontal",
            Vertical => "vertical",
        }
    }
}

#[deriving(Clone)]
pub enum VerticalPosition {
    Bottom,
    Middle,
    Top,
}

#[doc(hidden)]
impl Display<&'static str> for VerticalPosition {
    fn display(&self) -> &'static str {
        match *self {
            Bottom => "bottom",
            Middle => "center",
            Top => "top",
        }
    }
}
