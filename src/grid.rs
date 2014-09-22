use Script;

use axis::Axis;
use display::Display;

#[deriving(Clone)]
pub struct Properties {
    hidden: bool
}

// TODO Lots of configuration pending: linetype, linewidth, etc
impl Properties {
    // NB I dislike the visibility rules within the same crate
    #[doc(hidden)]
    pub fn _new() -> Properties {
        Properties {
            hidden: true,
        }
    }

    /// Hides the gridlines
    ///
    /// **Note** Both `Major` and `Minor` gridlines are hidden by default
    pub fn hide(&mut self) -> &mut Properties {
        self.hidden = true;
        self
    }

    /// Shows the gridlines
    pub fn show(&mut self) -> &mut Properties {
        self.hidden = false;
        self
    }
}

impl<'a, 'b, 'c> Script for (&'a Axis, &'b Grid, &'c Properties) {
    fn script(&self) -> String {
        let &(axis, grid, properties) = self;
        let axis = axis.display();
        let grid = grid.display();

        if properties.hidden {
            String::new()
        } else {
            format!("set grid {}{}tics\n", grid, axis)
        }
    }
}

#[deriving(Clone, Eq, Ord, PartialEq, PartialOrd)]
pub enum Grid {
    Major,
    Minor,
}

impl Display<&'static str> for Grid {
    fn display(&self) -> &'static str {
        match *self {
            Major => "",
            Minor => "m",
        }
    }
}
