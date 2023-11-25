use std::collections::HashMap;

use itertools::Itertools;
use pathfinding::prelude::*;
use rand::{rngs, Rng};

fn build_network(size: usize) -> Matrix<usize> {
    let mut network = Matrix::new(size, size, 0);
    let mut rng = rngs::OsRng;
    for a in 0..size {
        for b in 0..size {
            if rng.gen_ratio(2, 3) {
                network[(a, b)] = rng.gen::<u16>() as usize;
            }
        }
    }
    network
}

fn neighbours(network: Matrix<usize>) -> impl FnMut(&usize) -> Vec<(usize, usize)> {
    move |&a| {
        (0..network.rows)
            .filter_map(|b| match network[(a, b)] {
                0 => None,
                p => Some((b, p)),
            })
            .collect()
    }
}

#[test]
fn all_paths() {
    const SIZE: usize = 30;
    let network = build_network(SIZE);
    for start in 0..SIZE {
        let paths = dijkstra_all([start], neighbours(network.clone()));
        for target in 0..SIZE {
            if let Some((path, cost)) =
                dijkstra([start], neighbours(network.clone()), |&n| n == target)
            {
                if start == target {
                    assert!(
                        !paths.contains_key(&target),
                        "path {start} -> {target} is present in {network:?}"
                    );
                } else {
                    assert!(
                        paths.contains_key(&target),
                        "path {start} -> {target} is not found in {network:?}"
                    );
                    assert_eq!(
                        cost, paths[&target].1,
                        "cost differ in path {start} -> {target} in {network:?}"
                    );
                    let other_path = build_path(&target, &paths);
                    // There might be several paths, but we know that internally we use the
                    // same algorithm so the comparison holds.
                    assert_eq!(path, other_path, "path {start} -> {target} differ in {network:?}: {path:?} vs {other_path:?}");
                }
            } else {
                assert!(
                    !paths.contains_key(&target),
                    "path {start} -> {target} is present in {network:?}"
                );
            }
        }
    }
}

#[test]
fn partial_paths() {
    const SIZE: usize = 100;
    let network = build_network(SIZE);
    for start in 0..SIZE {
        let (paths, reached) = dijkstra_partial([start], neighbours(network.clone()), |&n| {
            start != 0 && n != 0 && n != start && n % start == 0
        });
        if let Some(target) = reached {
            assert!(target % start == 0, "bad stop condition");
            // We cannot compare other paths since there is no guarantee that the
            // paths variable is up-to-date as the algorithm stopped prematurely.
            let cost = paths[&target].1;
            let (path, dcost) =
                dijkstra([start], neighbours(network.clone()), |&n| n == target).unwrap();
            assert_eq!(
                cost, dcost,
                "costs {start} -> {target} differ in {network:?}"
            );
            let other_path = build_path(&target, &paths);
            // There might be several paths, but we know that internally we use the
            // same algorithm so the comparison holds.
            assert_eq!(
                path, other_path,
                "path {start} -> {target} differ in {network:?}: {path:?} vs {other_path:?}"
            );
        } else if start != 0 && start <= (SIZE - 1) / 2 {
            for target in 1..(SIZE / start) {
                assert!(
                    dijkstra([start], neighbours(network.clone()), |&n| n == target).is_none(),
                    "path {start} -> {target} found in {network:?}"
                );
            }
        }
    }
}

#[test]
fn dijkstra_reach_numbers() {
    let reach = dijkstra_reach([0], |prev, _| vec![(prev + 1, 1), (prev * 2, *prev)])
        .take_while(|x| x.total_cost < 100)
        .collect_vec();
    // the total cost should equal to the node's value, since the starting node is 0 and the cost to reach a successor node is equal to the increase in the node's value
    assert!(reach.iter().all(|x| x.node == x.total_cost));
    assert!((0..100).all(|x| reach.iter().any(|y| x == y.total_cost)));

    // dijkstra_reach should return reachable nodes in order of cost
    assert!(reach
        .iter()
        .map(|x| x.total_cost)
        .tuple_windows()
        .all(|(a, b)| b >= a));
}

#[test]
fn dijkstra_reach_graph() {
    //    2     2
    // A --> B --> C
    // \__________/
    //       5
    let mut graph = HashMap::new();
    graph.insert("A", vec![("B", 2), ("C", 5)]);
    graph.insert("B", vec![("C", 2)]);
    graph.insert("C", vec![]);

    let mut costs = HashMap::new();

    let reach = dijkstra_reach(["A"], |prev, cost| {
        costs.insert(*prev, cost);
        graph[prev].clone()
    })
    .collect_vec();

    // need to make sure that a node won't be returned twice when a better path is found after the first candidate
    assert!(
        reach
            == vec![
                DijkstraReachableItem {
                    node: "A",
                    parent: None,
                    total_cost: 0,
                },
                DijkstraReachableItem {
                    node: "B",
                    parent: Some("A"),
                    total_cost: 2,
                },
                DijkstraReachableItem {
                    node: "C",
                    parent: Some("B"),
                    total_cost: 4,
                },
            ]
    );

    for item in reach {
        assert!(item.total_cost == costs[item.node]);
    }
}
