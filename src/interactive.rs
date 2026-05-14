use core::u16;

use alloc::vec;
use alloc::vec::Vec;

use crate::position::Position;

pub(crate) struct FocusOrder<Key: Copy + Eq> {
    nodes: Vec<FocusNode<Key>>,
}

/// Finds the minimum and maximum values in an iterator where the maximum is positive
/// and the minimum is negative.
fn find_min_max<T>(
    elements: impl Iterator<Item = T>,
    key: impl Fn(&T) -> isize,
) -> (Option<T>, Option<T>) {
    let ((_, below), (_, above)) = elements.fold(
        ((0, None), (0, None)),
        |((min_distance, below), (max_distance, above)), value| {
            let k = key(&value);
            if k < min_distance {
                ((k, Some(value)), (max_distance, above))
            } else if k > max_distance {
                ((min_distance, below), (k, Some(value)))
            } else {
                ((min_distance, below), (max_distance, above))
            }
        },
    );
    (below, above)
}

impl<Key: Copy + Eq> FocusOrder<Key> {
    pub fn new(elements: Vec<FocusItem<Key>>) {
        match elements.len() {
            0 => FocusOrder { nodes: Vec::new() },
            1 => {
                let item = elements[0];
                let node = FocusNode {
                    item,
                    up: NO_INDEX,
                    down: NO_INDEX,
                    left: NO_INDEX,
                    right: NO_INDEX,
                };

                FocusOrder { nodes: vec![node] }
            }
            _ => {
                let nodes: Vec<FocusNode<Key>> = elements
                    .iter()
                    .map(|item| {
                        let (down, up) = {
                            let (down_option, up_option) =
                                find_min_max(elements.iter().enumerate(), |(_, other)| {
                                    other.position.y as isize - item.position.y as isize
                                });

                            (
                                down_option
                                    .map(|(index, _)| index as u16)
                                    .unwrap_or(NO_INDEX),
                                up_option.map(|(index, _)| index as u16).unwrap_or(NO_INDEX),
                            )
                        };

                        let (left, right) = {
                            let (left_option, right_option) =
                                find_min_max(elements.iter().enumerate(), |(_, other)| {
                                    other.position.x as isize - item.position.x as isize
                                });

                            (
                                left_option
                                    .map(|(index, _)| index as u16)
                                    .unwrap_or(NO_INDEX),
                                right_option
                                    .map(|(index, _)| index as u16)
                                    .unwrap_or(NO_INDEX),
                            )
                        };

                        FocusNode {
                            item: *item,
                            up,
                            down,
                            left,
                            right,
                        }
                    })
                    .collect();

                FocusOrder { nodes }
            }
        };
    }
}

#[derive(Clone, Copy)]
pub(crate) struct FocusItem<Key: Copy + Eq> {
    pub(crate) position: Position,
    pub(crate) key: Key,
}

pub(crate) type Index = u16;
const NO_INDEX: u16 = u16::MAX;

#[derive(Clone, Copy)]
pub(crate) struct FocusNode<Key: Copy + Eq> {
    item: FocusItem<Key>,
    up: Index,
    down: Index,
    left: Index,
    right: Index,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FocusState {
    Unfocused,
    Focused,
    Active,
}
