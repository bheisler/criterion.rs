//! Coordinate axis

use std::iter::IntoIterator;
use std::string::CowString;

use map;
use {Axis, Default, Display, Grid, Label, Range, Scale, Script, TicLabels, grid};
use traits::{Configure, Data, Set};

/// Properties of the coordinate axes
#[derive(Clone)]
pub struct Properties {
    grids: map::grid::Map<grid::Properties>,
    hidden: bool,
    label: Option<CowString<'static>>,
    logarithmic: bool,
    range: Option<(f64, f64)>,
    tics: Option<String>,
}

impl Default for Properties {
    fn default() -> Properties {
        Properties {
            grids: map::grid::Map::new(),
            hidden: false,
            label: None,
            logarithmic: false,
            range: None,
            tics: None,
        }
    }
}

impl Properties {
    /// Hides the axis
    ///
    /// **Note** The `TopX` and `RightY` axes are hidden by default
    pub fn hide(&mut self) -> &mut Properties {
        self.hidden = true;
        self
    }

    /// Makes the axis visible
    ///
    /// **Note** The `BottomX` and `LeftY` axes are visible by default
    pub fn show(&mut self) -> &mut Properties {
        self.hidden = false;
        self
    }
}

impl Configure<Grid> for Properties {
    type Properties = grid::Properties;

    /// Configures the gridlines
    fn configure<F>(&mut self, grid: Grid, configure: F) -> &mut Properties where
        F: FnOnce(&mut grid::Properties) -> &mut grid::Properties,
    {
        if self.grids.contains_key(grid) {
            configure(self.grids.get_mut(grid).unwrap());
        } else {
            let mut properties = Default::default();
            configure(&mut properties);
            self.grids.insert(grid, properties);
        }

        self
    }
}

impl Set<Label> for Properties {
    /// Attaches a label to the axis
    fn set(&mut self, label: Label) -> &mut Properties {
        self.label = Some(label.0);
        self
    }
}

impl Set<Range> for Properties {
    /// Changes the range of the axis that will be shown
    ///
    /// **Note** All axes are auto-scaled by default
    fn set(&mut self, range: Range) -> &mut Properties {
        self.hidden = false;

        match range {
            Range::Auto => self.range = None,
            Range::Limits(low, high) => self.range = Some((low, high)),
        }

        self
    }
}

impl Set<Scale> for Properties {
    /// Sets the scale of the axis
    ///
    /// **Note** All axes use a linear scale by default
    fn set(&mut self, scale: Scale) -> &mut Properties {
        self.hidden = false;

        match scale {
            Scale::Linear => self.logarithmic = false,
            Scale::Logarithmic => self.logarithmic = true,
        }

        self
    }
}

// FIXME(rust-lang/rust#20300) move bounds to where clauses
impl<P: IntoIterator, L: IntoIterator> Set<TicLabels<P, L>> for Properties where
    <P::IntoIter as Iterator>::Item: Data,
    <L::IntoIter as Iterator>::Item: Str,
{
    /// Attaches labels to the tics of an axis
    fn set(&mut self, tics: TicLabels<P, L>) -> &mut Properties {
        let TicLabels { positions, labels } = tics;

        let pairs = positions.into_iter().zip(labels.into_iter()).map(|(pos, label)| {
            format!("'{}' {}", label.as_slice(), pos.f64())
        }).collect::<Vec<_>>();

        if pairs.len() == 0 {
            self.tics = None
        } else {
            self.tics = Some(pairs.connect(", "));
        }

        self
    }
}

impl<'a> Script for (Axis, &'a Properties) {
    fn script(&self) -> String {
        let &(axis, properties) = self;
        let axis_ = axis.display();

        let mut script = if properties.hidden {
            return format!("unset {}tics\n", axis_);
        } else {
            format!("set {}tics nomirror ", axis_)
        };

        if let Some(ref tics) = properties.tics {
            script.push_str(&format!("({})", tics))
        }

        script.push('\n');

        if let Some(ref label) = properties.label {
            script.push_str(&format!("set {}label '{}'\n", axis_, label))
        }

        if let Some((low, high)) = properties.range {
            script.push_str(&format!("set {}range [{}:{}]\n", axis_, low, high))
        }

        if properties.logarithmic {
            script.push_str(&format!("set logscale {}\n", axis_));
        }

        for (grid, properties) in properties.grids.iter() {
            script.push_str(&(axis, grid, properties).script());
        }

        script
    }
}
