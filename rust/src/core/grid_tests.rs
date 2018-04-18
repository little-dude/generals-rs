use super::grid::*;

#[test]
fn test_grid() {
    let grid = Grid::<u8>::new(|idx| idx as u8, 3, 4);
    assert_eq!(grid.width(), 3);
    assert_eq!(grid.height(), 4);
    assert_eq!(grid.len(), 12);
    assert!(!grid.is_empty());
    assert_eq!(grid.tiles(), &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
}

#[test]
fn test_indices() {
    let grid = Grid::<u8>::new(|idx| idx as u8, 3, 4);
    for i in 0..=11 {
        println!("checking that {} is a valid index", i);
        assert!(grid.is_valid_index(i));
    }
    assert!(!grid.is_valid_index(12));
}

#[test]
fn test_directions() {
    let grid = Grid::<u8>::new(|idx| idx as u8, 3, 4);
    assert!(grid.up(0).is_none());
    assert_eq!(grid.up(11), Some(8));

    assert!(grid.down(10).is_none());
    assert_eq!(grid.down(1), Some(4));

    assert!(grid.left(3).is_none());
    assert_eq!(grid.left(7), Some(6));

    assert!(grid.right(2).is_none());
    assert_eq!(grid.right(7), Some(8));

    assert_eq!(grid.right(0), Some(1));
    assert_eq!(grid.down(0), Some(3));
}

// 0  1  2
// 3  4  5
// 6  7  8
// 9  10 11
//
// We go up/left/right/diwn for neighbors
#[test]
fn test_direct_neighbors() {
    let grid = Grid::<u8>::new(|idx| idx as u8, 3, 4);

    let neighbors: Vec<usize> = grid.direct_neighbors(0).collect();
    assert_eq!(neighbors.as_slice(), &[1, 3]);

    let neighbors: Vec<usize> = grid.direct_neighbors(7).collect();
    assert_eq!(neighbors.as_slice(), &[4, 6, 8, 10]);
}

// 0  1  2
// 3  4  5
// 6  7  8
// 9  10 11
//
// We go up-left/up/up-right/left/right/down-left/down/down-right for neighbors
#[test]
fn test_extended_neighbors() {
    let grid = Grid::<u8>::new(|idx| idx as u8, 3, 4);

    let neighbors: Vec<usize> = grid.extended_neighbors(0).collect();
    assert_eq!(neighbors.as_slice(), &[1, 3, 4]);

    let neighbors: Vec<usize> = grid.extended_neighbors(7).collect();
    assert_eq!(neighbors.as_slice(), &[3, 4, 5, 6, 8, 9, 10, 11]);
}
