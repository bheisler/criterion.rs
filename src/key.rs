use std::str::MaybeOwned;

use {Default, Display, Script};

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

impl Default for Properties {
    fn default() -> Properties {
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
}

impl Properties {
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
    pub fn title<S>(&mut self, title: S) -> &mut Properties where S: IntoMaybeOwned<'static> {
        self.title = Some(title.into_maybe_owned());
        self
    }
}

impl Script for Properties {
    fn script(&self) -> String {
        let mut script = if self.hidden {
            return "set key off\n".to_string();
        } else {
            "set key on ".to_string()
        };

        match self.position {
            None => {},
            Some(Position::Inside(v, h)) => {
                script.push_str(format!("inside {} {} ", v.display(), h.display())[])
            },
            Some(Position::Outside(v, h)) => {
                script.push_str(format!("outside {} {} ", v.display(), h.display())[])
            },
        }

        if let Some(stack) =  self.stack {
            script.push_str(stack.display());
            script.push(' ');
        }

        if let Some(justification) = self.justification {
            script.push_str(justification.display());
            script.push(' ');
        }

        if let Some(order) =  self.order {
            script.push_str(order.display());
            script.push(' ');
        }

        if let Some(ref title) = self.title {
            script.push_str(format!("title '{}' ", title)[])
        }

        if self.boxed {
            script.push_str("box ")
        }

        script.push('\n');
        script
    }
}

/// Horizontal position of the key
#[deriving(Clone)]
pub enum Horizontal {
    Center,
    Left,
    Right,
}

/// Text justification of the key
#[deriving(Clone)]
pub enum Justification {
    Left,
    Right,
}

/// Order of the elements of the key
#[deriving(Clone)]
pub enum Order {
    SampleText,
    TextSample,
}

/// Position of the key
// TODO XY position
#[deriving(Clone)]
pub enum Position {
    Inside(Vertical, Horizontal),
    Outside(Vertical, Horizontal),
}

/// How the entries of the key are stacked
#[deriving(Clone)]
pub enum Stack {
    Horizontal,
    Vertical,
}

/// Vertical position of the key
#[deriving(Clone)]
pub enum Vertical{
    Bottom,
    Center,
    Top,
}
