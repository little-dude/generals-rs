import QUnit from "qunit";

import { Grid, Coordinates } from "../src/grid";
import { Direction } from "../src/direction";

QUnit.module("Grid", {
  beforeEach: function(assert) {
    this.grid = new Grid(document.getElementById("grid"));
    this.grid.init(3, 4);
    this.table = this.grid.table;
  }
});

QUnit.test("height", function(assert) {
  assert.equal(this.grid.height(), 3);
});

QUnit.test("width", function(assert) {
  assert.equal(this.grid.width(), 4);
});

QUnit.test("length", function(assert) {
  assert.equal(this.grid.length(), 12);
});

QUnit.test("coordinates", function(assert) {
  let grid = this.grid;
  assert.deepEqual(grid.coordinates(0), new Coordinates(0, 0), "(0, 0)");
  assert.deepEqual(grid.coordinates(4), new Coordinates(1, 0), "(1, 0)");
  assert.deepEqual(grid.coordinates(6), new Coordinates(1, 2), "(1, 2)");
  assert.deepEqual(grid.coordinates(11), new Coordinates(2, 3), "(2, 3)");
});

QUnit.test("index", function(assert) {
  let grid = this.grid;
  assert.equal(grid.index(new Coordinates(0, 0)), 0, "(0, 0)");
  assert.equal(grid.index(new Coordinates(1, 0)), 4, "(1, 0)");
  assert.equal(grid.index(new Coordinates(1, 2)), 6, "(1, 2)");
  assert.equal(grid.index(new Coordinates(2, 3)), 11, "(2, 3)");
});

QUnit.test("isValidIndex", function(assert) {
  let grid = this.grid;
  for (let i = 0; i < 12; i++) {
    assert.equal(grid.isValidIndex(i), true);
  }
  assert.equal(grid.isValidIndex(-1), false);
  assert.equal(grid.isValidIndex(13), false);
  assert.equal(grid.isValidIndex(true), false);
  assert.equal(grid.isValidIndex("1"), false);
  assert.equal(grid.isValidIndex(null), false);
  assert.equal(grid.isValidIndex(undefined), false);
});

QUnit.test("getCell", function(assert) {
  let grid = this.grid;
  for (let i = 0; i < 12; i++) {
    assert.equal(grid.getCell(i).index(), i);
  }

  function assertInvalid(value) {
    assert.throws(
      function() {
        grid.getCell(value);
      },
      RangeError,
      "RangeError thrown for " + value
    );
  }
  assertInvalid(-1);
  assertInvalid(13);
  assertInvalid(false);
  assertInvalid("");
  assertInvalid("1");
  assertInvalid(null);
  assertInvalid(undefined);
});

QUnit.test("getCellSafe", function(assert) {
  let grid = this.grid;
  for (let i = 0; i < 12; i++) {
    assert.equal(grid.getCellSafe(i).index(), i);
  }

  assert.equal(grid.getCellSafe(-1), null);
  assert.equal(grid.getCellSafe(13), null);
  assert.equal(grid.getCellSafe(false), null);
  assert.equal(grid.getCellSafe(""), null);
  assert.equal(grid.getCellSafe("1"), null);
  assert.equal(grid.getCellSafe(null), null);
  assert.equal(grid.getCellSafe(undefined), null);
});

QUnit.test("select", function(assert) {
  let grid = this.grid;
  assert.equal(grid.selected, null);
  grid.select(3);
  assert.equal(grid.selected, 3);
  grid.select(9);
  assert.equal(grid.selected, 9);
});

QUnit.test("getNeighborCell", function(assert) {
  let grid = this.grid;
  assert.equal(grid.getNeighborCell(5, Direction.Up).index(), 1, "ok up");
  assert.equal(grid.getNeighborCell(5, Direction.Left).index(), 4, "ok left");
  assert.equal(grid.getNeighborCell(5, Direction.Right).index(), 6, "ok right");
  assert.equal(grid.getNeighborCell(5, Direction.Down).index(), 9, "ok down");

  assert.equal(grid.getNeighborCell(1, Direction.Up), null, "null up");
  assert.equal(grid.getNeighborCell(4, Direction.Left), null, "null left");
  assert.equal(grid.getNeighborCell(3, Direction.Right), null, "null right");
  assert.equal(grid.getNeighborCell(10, Direction.Down), null, "null down");

  // from an invalid index
  assert.equal(grid.getNeighborCell(42, Direction.Down), null, "null invalid");
  assert.equal(grid.getNeighborCell(-1, Direction.Down), null, "null invalid");
});
