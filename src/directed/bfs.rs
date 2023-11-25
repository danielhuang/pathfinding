//! Compute a shortest path using the [breadth-first search
//! algorithm](https://en.wikipedia.org/wiki/Breadth-first_search).

use super::reverse_path;
use crate::{FxIndexMap, FxIndexSet};
use indexmap::map::Entry::Vacant;
use std::hash::Hash;
use std::iter::FusedIterator;
use std::usize;

/// Compute a shortest path using the [breadth-first search
/// algorithm](https://en.wikipedia.org/wiki/Breadth-first_search).
///
/// The shortest path starting from `start` up to a node for which `success` returns `true` is
/// computed and returned in a `Some`. If no path can be found, `None`
/// is returned instead.
///
/// - `start` is the starting node.
/// - `successors` returns a list of successors for a given node.
/// - `success` checks whether the goal has been reached. It is not a node as some problems require
/// a dynamic solution instead of a fixed node.
///
/// A node will never be included twice in the path as determined by the `Eq` relationship.
///
/// The returned path comprises both the start and end node.
///
/// # Example
///
/// We will search the shortest path on a chess board to go from (1, 1) to (4, 6) doing only knight
/// moves.
///
/// The first version uses an explicit type `Pos` on which the required traits are derived.
///
/// ```
/// use pathfinding::prelude::bfs;
///
/// #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
/// struct Pos(i32, i32);
///
/// impl Pos {
///   fn successors(&self) -> Vec<Pos> {
///     let &Pos(x, y) = self;
///     vec![Pos(x+1,y+2), Pos(x+1,y-2), Pos(x-1,y+2), Pos(x-1,y-2),
///          Pos(x+2,y+1), Pos(x+2,y-1), Pos(x-2,y+1), Pos(x-2,y-1)]
///   }
/// }
///
/// static GOAL: Pos = Pos(4, 6);
/// let result = bfs(&Pos(1, 1), |p| p.successors(), |p| *p == GOAL);
/// assert_eq!(result.expect("no path found").len(), 5);
/// ```
///
/// The second version does not declare a `Pos` type, makes use of more closures,
/// and is thus shorter.
///
/// ```
/// use pathfinding::prelude::bfs;
///
/// static GOAL: (i32, i32) = (4, 6);
/// let result = bfs(&(1, 1),
///                  |&(x, y)| vec![(x+1,y+2), (x+1,y-2), (x-1,y+2), (x-1,y-2),
///                                 (x+2,y+1), (x+2,y-1), (x-2,y+1), (x-2,y-1)],
///                  |&p| p == GOAL);
/// assert_eq!(result.expect("no path found").len(), 5);
/// ```
pub fn bfs<N, FN, IN, FS>(
    starts: impl IntoIterator<Item = N>,
    successors: FN,
    success: FS,
) -> Option<Vec<N>>
where
    N: Eq + Hash + Clone,
    FN: FnMut(&N) -> IN,
    IN: IntoIterator<Item = N>,
    FS: FnMut(&N) -> bool,
{
    bfs_core(starts, successors, success, true)
}

fn bfs_core<N, FN, IN, FS>(
    starts: impl IntoIterator<Item = N>,
    mut successors: FN,
    mut success: FS,
    check_first: bool,
) -> Option<Vec<N>>
where
    N: Eq + Hash + Clone,
    FN: FnMut(&N) -> IN,
    IN: IntoIterator<Item = N>,
    FS: FnMut(&N) -> bool,
{
    let mut i = 0;
    let mut parents: FxIndexMap<N, usize> = FxIndexMap::default();
    for start in starts {
        if check_first && success(&start) {
            return Some(vec![start.clone()]);
        }
        parents.insert(start.clone(), usize::max_value());
    }
    while let Some((node, _)) = parents.get_index(i) {
        for successor in successors(node) {
            if success(&successor) {
                let mut path = reverse_path(&parents, |&p| p, i);
                path.push(successor);
                return Some(path);
            }
            if let Vacant(e) = parents.entry(successor) {
                e.insert(i);
            }
        }
        i += 1;
    }
    None
}

/// Return one of the shortest loop from start to start if it exists, `None` otherwise.
///
/// - `start` is the starting node.
/// - `successors` returns a list of successors for a given node.
///
/// Except the start node which will be included both at the beginning and the end of
/// the path, a node will never be included twice in the path as determined
/// by the `Eq` relationship.
pub fn bfs_loop<N, FN, IN>(start: N, successors: FN) -> Option<Vec<N>>
where
    N: Eq + Hash + Clone,
    FN: FnMut(&N) -> IN,
    IN: IntoIterator<Item = N>,
{
    bfs_core([start.clone()], successors, |n| n == &start, false)
}

/// Visit all nodes that are reachable from a start node. The node will be visited
/// in BFS order, starting from the `start` node and following the order returned
/// by the `successors` function.
///
/// # Examples
///
/// The iterator stops when there are no new nodes to visit:
///
/// ```
/// use pathfinding::prelude::bfs_reach;
///
/// let all_nodes = bfs_reach(3, |_| (1..=5)).collect::<Vec<_>>();
/// assert_eq!(all_nodes, vec![3, 1, 2, 4, 5]);
/// ```
///
/// The iterator can be used as a generator. Here are for examples
/// the multiples of 2 and 3 (although not in natural order but in
/// the order they are discovered by the BFS algorithm):
///
/// ```
/// use pathfinding::prelude::bfs_reach;
///
/// let mut it = bfs_reach(1, |&n| vec![n*2, n*3]).skip(1);
/// assert_eq!(it.next(), Some(2));  // 1*2
/// assert_eq!(it.next(), Some(3));  // 1*3
/// assert_eq!(it.next(), Some(4));  // (1*2)*2
/// assert_eq!(it.next(), Some(6));  // (1*2)*3
/// // (1*3)*2 == 6 which has been seen already
/// assert_eq!(it.next(), Some(9));  // (1*3)*3
/// assert_eq!(it.next(), Some(8));  // ((1*2)*2)*2
/// assert_eq!(it.next(), Some(12)); // ((1*2)*2)*3
/// ```
pub fn bfs_reach<N, FN, IN>(
    starts: impl IntoIterator<Item = N>,
    successors: FN,
) -> BfsReachable<N, FN>
where
    N: Eq + Hash + Clone,
    FN: FnMut(&N) -> IN,
    IN: IntoIterator<Item = N>,
{
    let mut seen = FxIndexSet::default();
    for start in starts {
        seen.insert(start);
    }
    BfsReachable {
        i: 0,
        seen,
        successors,
    }
}

/// Struct returned by [`bfs_reach`](crate::directed::bfs::bfs_reach).
pub struct BfsReachable<N, FN> {
    i: usize,
    seen: FxIndexSet<N>,
    successors: FN,
}

impl<N, FN> BfsReachable<N, FN> {
    /// Return a lower bound on the number of remaining reachable
    /// nodes. Not all nodes are necessarily known in advance, and
    /// new reachable nodes may be discovered while using the iterator.
    pub fn remaining_nodes_low_bound(&self) -> usize {
        self.seen.len() - self.i
    }
}

impl<N, FN, IN> Iterator for BfsReachable<N, FN>
where
    N: Eq + Hash + Clone,
    FN: FnMut(&N) -> IN,
    IN: IntoIterator<Item = N>,
{
    type Item = N;

    fn next(&mut self) -> Option<Self::Item> {
        let n = self.seen.get_index(self.i)?.clone();
        for s in (self.successors)(&n) {
            self.seen.insert(s);
        }
        self.i += 1;
        Some(n)
    }
}

impl<N, FN, IN> FusedIterator for BfsReachable<N, FN>
where
    N: Eq + Hash + Clone,
    FN: FnMut(&N) -> IN,
    IN: IntoIterator<Item = N>,
{
}
