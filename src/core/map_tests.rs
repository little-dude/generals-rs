use std::cell::RefCell;

use super::common::{Direction, Move, Tile};
use super::grid::Grid;
use super::map::Map;

const PLAYER_1: usize = 1;
const PLAYER_2: usize = 2;

const EMPTY_1: usize = 1;
const EMPTY_2: usize = 2;
const OPEN_1: usize = 4;
const GENERAL: usize = 5;
const FORTRESS: usize = 6;
const OPEN_2: usize = 9;
const EMPTY_3: usize = 11;

/// Return a map with two players, that looks like:
///
/// ```
/// Mountain  Empty       Empty     Mountain
/// Open1[1]  General[2]  City[1]   Mountain
/// Mountain  Open2[2]    Mountain  Empty
/// ```
///
/// `Open1` and `City1` belong to player 1.
/// `Gereral1` and `Open2` belong to player 2.
fn get_map() -> Map {
    let grid = Grid::new(|_| RefCell::new(Tile::new()), 4, 3);

    {
        for index in &[EMPTY_1, EMPTY_2] {
            let mut tile = grid.get(*index).borrow_mut();
            tile.make_open();
            tile.reveal_to(1);
            tile.reveal_to(2);
        }
        let mut tile = grid.get(EMPTY_3).borrow_mut();
        tile.make_open();
        tile.reveal_to(1);

        // Open1 (P1, 20 units)
        let mut tile = grid.get(OPEN_1).borrow_mut();
        tile.make_open();
        tile.set_owner(Some(1));
        tile.set_units(20);
        tile.reveal_to(1);
        tile.reveal_to(2);
        tile.set_clean();

        // General (P2, 10 units)
        let mut tile = grid.get(GENERAL).borrow_mut();
        tile.make_general();
        tile.set_owner(Some(2));
        tile.set_units(10);
        tile.reveal_to(1);
        tile.reveal_to(2);
        tile.set_clean();

        // City (P1, 8 units)
        let mut tile = grid.get(FORTRESS).borrow_mut();
        tile.make_city();
        tile.set_owner(Some(1));
        tile.set_units(8);
        tile.reveal_to(1);
        tile.reveal_to(2);
        tile.set_clean();

        // Open2 (P2, 4 units)
        let mut tile = grid.get(OPEN_2).borrow_mut();
        tile.make_open();
        tile.set_owner(Some(2));
        tile.set_units(4);
        tile.reveal_to(1);
        tile.reveal_to(2);
        tile.set_clean();
    }

    Map::from_grid(grid)
}

#[test]
fn test_transfer_units() {
    let mut map = get_map();
    map.perform_move(Move {
        player: PLAYER_2,
        from: GENERAL,
        direction: Direction::Down,
    }).unwrap();
    let src = map.get(GENERAL);
    let dst = map.get(OPEN_2);
    assert_eq!(src.units(), 1);
    assert_eq!(dst.units(), 13);
    assert_eq!(src.owner(), Some(2));
    assert_eq!(dst.owner(), Some(2));
    assert!(src.is_dirty());
    assert!(dst.is_dirty());
}

#[test]
fn test_conquer_city() {
    let mut map = get_map();
    map.perform_move(Move {
        player: PLAYER_2,
        from: GENERAL,
        direction: Direction::Right,
    }).unwrap();
    let src = map.get(GENERAL);
    let dst = map.get(FORTRESS);
    assert_eq!(src.owner(), Some(2));
    assert_eq!(dst.owner(), Some(2));
    assert_eq!(src.units(), 1);
    assert_eq!(dst.units(), 1);
    assert!(src.is_dirty());
    assert!(dst.is_dirty());

    // Make sure Empty3 become visible to player2 and invisible to player1.
    assert!(map.get(EMPTY_3).is_visible_by(2));
    assert!(!map.get(EMPTY_3).is_visible_by(1));
}

#[test]
fn test_conquer_general() {
    let mut map = get_map();
    map.perform_move(Move {
        player: PLAYER_1,
        from: OPEN_1,
        direction: Direction::Right,
    }).unwrap();
    let src = map.get(OPEN_1);
    let dst = map.get(GENERAL);
    assert_eq!(src.owner(), Some(1));
    assert_eq!(dst.owner(), Some(1));
    assert_eq!(src.units(), 1);
    assert_eq!(dst.units(), 9);
    assert!(src.is_dirty());
    assert!(dst.is_dirty());

    // Make sure tiles belonging to player2 now belong to player1
    assert_eq!(map.get(OPEN_2).owner(), Some(1));

    // Make sure tiles are now visible by player1 only
    for index in &[EMPTY_1, EMPTY_2, OPEN_1, GENERAL, FORTRESS, OPEN_2, EMPTY_3] {
        let mut tile = map.get(*index);
        println!("{:?}", tile);
        println!("{} should be visible by 1", index);
        assert!(tile.is_visible_by(1));
        println!("{} should be invisible to 2", index);
        assert!(!tile.is_visible_by(2));
    }
}

#[test]
fn test_conquer_reinforce() {
    let mut map = get_map();
    assert_eq!(map.get(EMPTY_1).units(), 0);
    assert_eq!(map.get(EMPTY_2).units(), 0);
    assert_eq!(map.get(OPEN_1).units(), 20);
    assert_eq!(map.get(GENERAL).units(), 10);
    assert_eq!(map.get(FORTRESS).units(), 8);
    assert_eq!(map.get(OPEN_2).units(), 4);
    assert_eq!(map.get(EMPTY_3).units(), 0);

    map.reinforce(true);
    assert_eq!(map.get(EMPTY_1).units(), 0);
    assert_eq!(map.get(EMPTY_2).units(), 0);
    assert_eq!(map.get(OPEN_1).units(), 21);
    assert_eq!(map.get(GENERAL).units(), 11);
    assert_eq!(map.get(FORTRESS).units(), 9);
    assert_eq!(map.get(OPEN_2).units(), 5);
    assert_eq!(map.get(EMPTY_3).units(), 0);
}
