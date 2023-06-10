use std::fmt::{Debug, Display};
use std::io::BufRead;
use std::iter::FromIterator;
use std::str::FromStr;

use log::debug;
use log::warn;

/// Parse a series of items from lines in a buffer.
///
/// Empty lines are skipped, and lines are trimmed before parsing.
pub fn buffer<B, Item, F>(buf: B) -> anyhow::Result<F>
where
    B: BufRead,
    Item: Debug + FromStr,
    Item::Err: Into<anyhow::Error> + Display,
    F: FromIterator<Item>,
{
    buf.lines()
        .filter_map(|rl| match rl {
            Err(e) => {
                warn!("  Error getting line: {}", e);
                Some(Err(e.into()))
            }
            Ok(l) => {
                let trimmed = l.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    let fd = Item::from_str(trimmed);
                    match fd {
                        Ok(ref i) => debug!("  Parsed line '{}' -> {:?}", trimmed, i),
                        Err(ref e) => warn!("  Error parsing line '{}': {}", trimmed, e),
                    }
                    Some(fd.map_err(|e| e.into()))
                }
            }
        })
        .collect()
}
