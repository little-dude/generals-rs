"use strict";

/**
 * A table in the DOM tree.
 * @external HTMLTableElement
 * @see {@link https://developer.mozilla.org/en-US/docs/Web/API/HTMLTableElement HTMLTableElement}
 */

import { Direction } from "./direction.js";
import { Cell } from "./cell.js";

class Coordinates {
  /**
   * Represent cartisian coordinates of a {@link Cell} in a {@link Grid}.
   *
   * @param {number} row row index
   * @param {number} col column index
   */
  constructor(row, col) {
    if (!(Number.isSafeInteger(row) && Number.isSafeInteger(col))) {
      throw TypeError(
        "Invalid arguments (invalid coordinates): '" + row + "', '" + col + "'"
      );
    }
    this.row = row;
    this.col = col;
  }
}

class Grid {
  /**
   * Create a new {@link Grid}, that wraps the given {@link
   * external:HTMLTableElement}. A {@link Grid} provide helpers to update
   * individual {@link Cell}s. For a {@link Grid} to work properly, the
   * inderlying table (<tt>&lt;table></tt>) 's rows (<tt>&lt;tr></tt>) must all have the
   * same number of cells (<tt>&lt;td></tt>).
   *
   * The {@link Grid} keeps track of which {@link Cell} is selected, and
   * provides a method {@link Grid#select} to select another {@link Cell}.
   *
   * @param {external:HTMLTableElement}
   */
  constructor(table) {
    if (!(table instanceof HTMLTableElement)) {
      throw TypeError(
        "Cannot instantiate grid from " + table + ": not a <table>"
      );
    }
    this.table = table;
    this.selected = null;
  }

  /**
   * Mark the {@link Cell} at the given index as "selected". Only one cell at a
   * time can be selected, so when a cell is selected, the cell that was
   * previously selected is un-marked.
   *
   * @example
   *
   * document.body.innerHTML = `<table id="grid"></table>`;
   * let grid = new Grid(document.getElementById("grid"));
   * grid.init(3,3);
   *
   * // select the second cell and check that the "selected" attribute is checked
   * grid.select(2);
   * let cell = grid.getCell(2);
   * console.assert(cell.selected);
   *
   * // select the third cell and check that the previous cell was unselected
   * // and that the new cell is selected.
   * grid.select(3);
   * console.assert(!cell.selected);
   * let cell = grid.getCell(3);
   * console.assert(cell.selected);
   */
  select(index) {
    // Deselect the current tile
    if (this.selected) {
      this.getCell(this.selected).selected = false;
    }
    let cell = this.getCell(index);
    if (!cell) {
      return;
    }
    cell.selected = true;
    this.selected = index;
  }

  /**
   * @return {number} the number of rows.
   *
   * @example
   *
   * document.body.innerHTML = `<table id="grid"></table>`;
   * let grid = new Grid(document.getElementById("grid"));
   * grid.init(3,3);
   * console.assert(grid.height() === 3);
   */
  height() {
    return this.table.rows.length;
  }

  /**
   * @return {number} the number of columns.
   *
   * @example
   *
   * document.body.innerHTML = `<table id="grid"></table>`;
   * let grid = new Grid(document.getElementById("grid"));
   * grid.init(3,3);
   * console.assert(grid.width() === 3);
   */
  width() {
    if (this.height() === 0) {
      return 0;
    }
    /*  Assume all the rows have the same length than the first row */
    return this.table.rows[0].cells.length;
  }

  /**
   * @return {number} the number of cells.
   *
   * @example
   *
   * document.body.innerHTML = `<table id="grid"></table>`;
   * let grid = new Grid(document.getElementById("grid"));
   * grid.init(3,3);
   * console.assert(grid.length() === 9);
   */
  length() {
    return this.height() * this.width();
  }

  /**
   * @return {Cell} the cell at the given index.
   * @throws {RangeError} is the index is invalid.
   *
   * @example
   *
   * document.body.innerHTML = `<table id="grid"></table>`;
   * let grid = new Grid(document.getElementById("grid"));
   * grid.init(3,3);
   * console.assert(grid.getCell(0).index() == 0);
   * console.assert(grid.getCell(8).index() == 8);
   * grid.getCell(-1); // throws RangeError
   * grid.getCell(0);  // throws RangeError
   */
  getCell(index) {
    if (!this.isValidIndex(index)) {
      throw new RangeError("Invalid index " + index);
    }
    let coordinates = this.coordinates(index);
    let row = this.table.rows[coordinates.row];
    let cell = row.cells[coordinates.col];
    return new Cell(cell);
  }

  /**
   * @return {Cell|null} the cell at the given index. Unline {@link
   * Grid#getCell}, if the index is invalid, no error is thrown. Instead,
   * <tt>null</tt>
   *
   * @example
   *
   * document.body.innerHTML = `<table id="grid"></table>`;
   * let grid = new Grid(document.getElementById("grid"));
   * grid.init(3,3);
   * console.assert(grid.getCellSafe(0) instanceof Cell);
   * console.assert(grid.getCellSafe(8) instanceof Cell);
   * console.assert(grid.getCellSafe(-1) === null);
   * console.assert(grid.getCellSafe(9) === null);
   */
  getCellSafe(index) {
    try {
      return this.getCell(index);
    } catch (e) {
      if (e instanceof RangeError) {
        return null;
      }
      throw e;
    }
  }

  /**
   * @param {number} index
   * @return {boolean} whether the given index is valid, or out-of-bounds
   *
   * @example
   *
   * document.body.innerHTML = `<table id="grid"></table>`;
   * let grid = new Grid(document.getElementById("grid"));
   * grid.init(3,3);
   * console.assert(grid.isValidIndex(0));
   * console.assert(grid.isValidIndex(8));
   * console.assert(!grid.isValidIndex(-1));
   * console.assert(!grid.isValidIndex(9));
   */
  isValidIndex(index) {
    return Number.isInteger(index) && index < this.length() && index >= 0;
  }

  /**
   * @param {number} index
   * @return {Coordinates} the cartesian coordinates corresponding to the given index
   *
   * @example
   *
   * document.body.innerHTML = `<table id="grid"></table>`;
   * let grid = new Grid(document.getElementById("grid"));
   * grid.init(3,3);
   * let coords = grid.coordinates(5);
   * console.assert(coords.row === 1);
   * console.assert(coords.col === 2);
   */
  coordinates(index) {
    if (!this.isValidIndex(index)) {
      throw new RangeError("Invalid index " + index);
    }
    return new Coordinates(
      Math.floor(index / this.width()),
      index % this.width()
    );
  }

  /**
   * @param {Coordinates} coordinates
   * @return {Number} the index cooresponding to the given coordinates
   *
   * @example
   *
   * document.body.innerHTML = `<table id="grid"></table>`;
   * let grid = new Grid(document.getElementById("grid"));
   * grid.init(3,3);
   * console.assert(grid.index(new Coordinates(1, 2)) == 5);
   * console.assert(grid.index(new Coordinates(3, 3)) == 9);
   */
  index(coordinates) {
    if (!(coordinates instanceof Coordinates)) {
      throw TypeError("Expected Coordinates got " + typeof coordinates);
    }
    return coordinates.row * this.width() + coordinates.col;
  }

  /**
   * Create a grid of empty cell with the given number of row and columns.
   *
   * @param {n_rows} number of rows
   * @param {n_cols} number of columns
   *
   * @example
   *
   * document.body.innerHTML = `<table id="grid"></table>`;
   * let grid = new Grid(document.getElementById("grid"));
   * grid.init(3,3);
   */
  init(n_rows, n_cols) {
    this.clear();

    for (let i = 0; i < n_rows; i++) {
      let row = document.createElement("tr");

      for (let j = 0; j < n_cols; j++) {
        let cell = new Cell();
        row.appendChild(cell.html());
        this.table.appendChild(row);
      }
    }
  }

  /**
   * Return the {@link Cell} adjacent to the {@link Cell} at the given index in
   * the given direction
   *
   * @param {Number} [index]
   * @param {Direction} [direction]
   *
   * @example
   *
   * document.body.innerHTML = `<table id="grid"></table>`;
   * let grid = new Grid(document.getElementById("grid"));
   * grid.init(3,3);
   *
   * console.assert(grid.getNeighborCell(4, Direction.Up) instanceof Cell);
   * console.assert(grid.getNeighborCell(4, Direction.Up).index() === 1);
   * console.assert(grid.getNeighborCell(4, Direction.Left).index() === 3);
   * console.assert(grid.getNeighborCell(4, Direction.Right).index() === 5);
   * console.assert(grid.getNeighborCell(4, Direction.Down).index() === 7);
   * console.assert(grid.getNeighborCell(0, Direction.Left) === null);
   *
   */
  getNeighborCell(index, direction) {
    if (!this.isValidIndex(index)) {
      return null;
    }
    let coord = this.coordinates(index);
    switch (direction) {
      case Direction.Right:
        if (coord.col === this.width() - 1) {
          return null;
        }
        return this.getCellSafe(index + 1);
      case Direction.Left:
        if (coord.col === 0) {
          return null;
        }
        return this.getCellSafe(index - 1);
      case Direction.Up:
        if (coord.row === 0) {
          return null;
        }
        return this.getCellSafe(index - this.width());
      case Direction.Down:
        if (coord.row === this.height() - 1) {
          return null;
        }
        return this.getCellSafe(index + this.width());
      default:
        throw "Unreachable: " + direction;
    }
  }

  /**
   * Return the list of {@link Cell}s adjacent to the {@link Cell} at the given
   * index.
   *
   * @param {Number} [index]
   *
   * @example
   *
   * document.body.innerHTML = `<table id="grid"></table>`;
   * let grid = new Grid(document.getElementById("grid"));
   * grid.init(3,3);
   * //TODO
   */
  getNeighborCells(index) {
    let neighbors = [];
    let directions = [
      Direction.Up,
      Direction.Down,
      Direction.Left,
      Direction.Right
    ];
    for (let direction of directions) {
      let cell = this.getNeighborCell(index, direction);
      if (cell) {
        neighbors.push(cell);
      }
    }
    return neighbors;
  }

  /**
   * Remove all the cells from the grid, leaving only
   * <tt>&lt;table>&lt;/table></tt> in the DOM.
   */
  clear() {
    while (this.table.firstChild) {
      this.table.firstChild.remove();
    }
  }
}

export { Grid };
