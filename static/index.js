"use strict";

/**
 * A table in the DOM tree.
 * @external HTMLTableElement
 * @see {@link https://developer.mozilla.org/en-US/docs/Web/API/HTMLTableElement HTMLTableElement}
 */

/**
 * A cell in a <tt>&lt;tr></tt> DOM element.
 * @external HTMLTableCellElement
 * @see {@link https://developer.mozilla.org/en-US/docs/Web/API/HTMLTableCellElement HTMLTableCellElement}
 */

/**
 * A DOM event.
 * @external Event
 * @see {@link https://developer.mozilla.org/en-US/docs/Web/API/Event}
 */

/**
 * Represent the possible types of cell:
 *
 * @example
 * CellType.Mountain // represent a mountain cell
 * CellType.Open     // represent an empty cell
 * CellType.City     // represent a cell containing a city
 * CellType.General  // represent a cell containing a general
 * </pre>
 */
let CellType = Object.freeze({
  Mountain: 1,
  Open: 2,
  City: 3,
  General: 4,

  /**
   * Return the cell type as a string.
   *
   * @param {Number} value an integer representing a <tt>CellType</tt>
   * @throws <tt>RangerError</tt> if the input is not a valid <tt>CellType</tt>
   * @returns {String} a string representing the given cell type
   *
   * @example
   * CellType.toString(CellType.Mountain) // "mountain"
   * CellType.toString(CellType.Open)     // "open"
   * CellType.toString(CellType.City)     // "city"
   * CellType.toString(CellType.General)  // "general"
   */
  toString: function(value) {
    if (!Number.isInteger(value)) {
      throw TypeError("Expected integer got " + typeof value);
    }
    switch (value) {
      case CellType.Mountain:
        return "mountain";
      case CellType.Open:
        return "open";
      case CellType.City:
        return "city";
      case CellType.General:
        return "general";
      default:
        throw RangeError("Not a valid CellType: " + value);
    }
  },

  /**
   * Parse a string into a <tt>CellType</tt>.
   *
   * @returns {CellType} if the input string correspond to a
   * <tt>CellType</tt> variant, it is returned.
   *
   * @param {String} string a string that can be parsed into a <tt>CellType</tt>
   *
   * @throws An error is thrown if the input string does not correspond to a
   * <tt>CellType</tt> variant.
   
   * @example
   * CellType.fromString("mountain")  // CellType.Mountain
   * CellType.fromString("open")      // CellType.Open
   * CellType.fromString("city")      // CellType.City
   * CellType.fromString("general")   // CellType.General
   * // This method is case sensitive, so this throws an error:
   * CellType.fromString("General")
   */
  fromString: function(string) {
    switch (string) {
      case "general":
        return CellType.General;
      case "city":
        return CellType.City;
      case "open":
        return CellType.Open;
      case "mountain":
        return CellType.Mountain;
      default:
        throw "Not a valid CellType: " + string;
    }
  }
});

/**
 * Represent the possible move directions:
 *
 * @example
 * Direction.Up    // represent a move up
 * Direction.Down  // represent a move down
 * Direction.Left  // represent a move to the left
 * Direction.Right // represent a move to the right
 */
let Direction = Object.freeze({
  Up: 1,
  Down: 2,
  Left: 3,
  Right: 4,

  /**
   * Return the direction as a string.
   *
   * @param {Number} value an integer representing a <tt>Direction</tt>
   * @throws <tt>RangerError</tt> if the input is not a valid <tt>Direction</tt>
   * @returns {String} a string representing the given direction
   *
   * @example
   * Direction.toString(Direction.Up)    // "up"
   * Direction.toString(Direction.Down)  // "down"
   * Direction.toString(Direction.Left)  // "left"
   * Direction.toString(Direction.Right) // "right"
   */
  toString: function(obj) {
    if (!Number.isInteger(obj)) {
      throw TypeError("Expected integer got " + typeof obj);
    }
    switch (obj) {
      case Direction.Up:
        return "up";
      case Direction.Down:
        return "down";
      case Direction.Left:
        return "left";
      case Direction.Right:
        return "right";
      default:
        throw RangeError(
          "Not a valid Direction: " + obj + "    " + Direction.Up
        );
    }
  },

  /**
   * Parse a string into a <tt>Direction</tt>.
   *
   * @returns {Direction} if the input string correspond to a
   * <tt>Direction</tt> variant, it is returned.
   *
   * @param {String} string a string that can be parsed into a <tt>Direction</tt>
   *
   * @throws An error is thrown if the input string does not correspond to a
   * <tt>Direction</tt> variant.
   
   * @example
   * Direction.fromString("up")  // Direction.Up
   * Direction.fromString("down")      // Direction.Down
   * Direction.fromString("left")      // Direction.Left
   * Direction.fromString("right")   // Direction.Right
   * // This method is case sensitive, so this throws an error:
   * Direction.fromString("Up")
   */
  fromString: function(string) {
    switch (string) {
      case "up":
        return Direction.Up;
      case "down":
        return Direction.Down;
      case "left":
        return Direction.Left;
      case "right":
        return Direction.Right;
      default:
        throw "Not a valid Direction: " + string;
    }
  }
});

/**
 * Exception thrown when setting/reading an invalid "data-" attribute on/from a cell.
 */
class ValidationError extends Error {
  constructor(attribute, value, message) {
    super(
      "Invalid value '" +
        value +
        "' for attribute '" +
        attribute +
        "': " +
        message
    );
    this.name = "ValidationError";
  }
}

class Cell {
  /**
   * A wrapper around {@link external:HTMLTableCellElement} (aka
   * <tt>&lt;td></tt>), that represents a cell on the map. The various
   * attributes are properties that manipulate the underlying {@link
   * external:HTMLTableCellElement}.
   *
   * <ul>
   * <li>
   *    the {@link Cell#units} attribute represents the number of units
   *    positioned on the cell. If set, the text of the underlying
   *    <tt>&lt;td</tt> node is set to the number of units.
   * </li>
   * <li>
   *    the {@link Cell#type} attribute represents the type of cell (mountain,
   *    open, city or general), with a {@link CellType} variant. If set, the
   *    <tt>data-type</tt> attribute is set to the corresponding value on the
   *    underlying <tt>&lt;td></tt> node.
   * </li>
   *
   * <li>
   *    the {@link Cell#owner} attribute represents the player that owns the
   *    cell. It corresponds to the the <tt>data-owner</tt> attribute on the
   *    underlying <tt>&lt;td></tt> node.  empty.
   * </li>
   *
   * <li>
   *    the {@link Cell#visible} represents whether the cell is visible by the
   *    play or not. If not, cells appear darker, and the player cannot see
   *    whether they are occupied or not. On the <tt>&lt;td></tt> node, it
   *    corresponds to the <tt>data-visible</tt> attribute
   * </li>
   * <li>
   *    the {@link Cell#selected} attribute represents whether the cell has
   *    been selected. In a grid, only one cell at a time can be selected. A
   *    cell is selected when it is clicked. Internally, this attribute set the
   *    <tt>data-selected</tt> attribute.
   * </li>
   * </ul>
   *
   * @example
   * // create a dummy table
   * let html = `<table>
   *   <tbody>
   *     <tr>
   *       <td></td>
   *       <td></td>
   *     </tr>
   *     <tr>
   *       <td></td>
   *       <td></td>
   *     </tr>
   *   </tbody>
   * </table>`;
   * document.body.innerHTML = html;
   *
   * // get the third cell in the table
   * let cell = new Cell(document.body.getElementsByTagName("td")[2]);
   *
   * // check the default attributes values
   * console.assert(cell.units === null);
   * console.assert(cell.type === CellType.Open);
   * console.assert(cell.visible === false);
   * console.assert(cell.owner === null);
   *
   * // Set the various cell attributes
   * cell.units = 42;           // <td>42</td>
   * cell.type = CellType.City; // <td data-type="city">42</td>
   * cell.visible = true;       // <td data-type="city" data-visible="true">42</td>
   * cell.owner = 1;            // <td data-type="city" data-visible="true" data-owner="1">42</td>
   * cell.selected = true;      // <td data-type="city" data-visible="true" data-owner="1" data-selected="true">42</td>
   *
   * // Unset the various cell attributes
   * cell.units = null          // <td data-type="city" data-visible="true" data-owner="1" data-selected="true"></td>
   * cell.type = null;          // <td data-visible="true" data-owner="1" data-selected="true"></td>
   * cell.visible = null;       // <td data-owner="1" data-selected="true"></td>
   * cell.owner = null;         // <td data-selected="true"></td>
   * cell.selected = true;      // <td></td>
   *
   * @param {external:HTMLTableCellElement} [td] - A &lt;td> element
   */
  constructor(td) {
    if (td instanceof HTMLTableCellElement) {
      this.td = td;
    } else if (td === undefined) {
      this.td = document.createElement("td");
      // this.visible = false;
      // this.owner = null;
      // this.units = 0;
      // this.type = CellType.Open;
    } else {
      throw new TypeError("Expected HTMLTableCellElement got " + typeof td);
    }
  }

  /**
   * Get or set the "data-type". It must be a {@link CellType} value, and
   * defaults to <tt>CellType.Open</tt>
   *
   * @example
   * let cell = new Cell(document.createElement("td")); // <td></td>
   * cell.type === CellType.Open;     // true
   * cell.type = CellType.Mountain ;  // <td data-type="mountain"></td>
   * cell.type = CellType.City;       // <td data-city="city"></td>
   * cell.type = null;                // <td></td>
   *
   * @method
   */
  get type() {
    let type = this.td.dataset.type;
    if (type === undefined) {
      return CellType.Open;
    }
    try {
      return CellType.fromString(type);
    } catch (e) {
      throw new ValidationError("type", type, "parsing failed (" + e + ")");
    }
  }

  set type(type) {
    if (type === null) {
      delete this.td.dataset.type;
      return;
    }
    try {
      this.td.dataset.type = CellType.toString(type);
    } catch (e) {
      throw new ValidationError("type", type, "parsing failed (" + e + ")");
    }
  }

  /**
   *
   * Get or set the number of units in the cell. The underneath
   * <tt>&lt;td></tt>'s inner text is set to the number of units. It must be a
   * positibe integer. It defaults to <tt>null</tt>.
   *
   * @example
   * let cell = new Cell(document.createElement("td")); // <td></td>
   * cell.units === null;  // true
   * cell.units = 1;       // <td>0</td>
   * cell.units = 1;       // <td>1</td>
   * cell.units = null;    // <td></td>
   *
   * @method
   */
  get units() {
    let units = this.td.innerText;
    if (units === "" || units === null) {
      return null;
    }
    let parsed = parseInt(units);
    if (isNaN(parsed) || parsed < 0) {
      throw new ValidationError("units", units, "not a positive integer");
    }
    return parsed;
  }

  set units(number) {
    if (number === null) {
      this.td.innerText = "";
      return;
    }
    if (!Number.isInteger(number) || number < 0) {
      throw new ValidationError("units", number, "not a positive integer");
    }
    this.td.innerText = number;
  }

  /**
   * Get or set the <tt>data-visible</tt> attribute. It must be a boolean or a
   * Boolean object. It defaults to <tt>false</tt>.
   *
   * @example
   * let cell = new Cell(document.createElement("td")); // <td></td>
   * cell.visible === false; // true
   * cell.visible = true;    // <td data-visible="true"></td>
   * cell.visible = false;   // <td data-visible="false"></td>
   * cell.visible = null;    // <td></td>
   *
   * @method
   */
  get visible() {
    let visible = this.td.dataset.visible;
    if (visible === undefined) {
      return false;
    }
    switch (visible) {
      case "false":
        return false;
      case "true":
        return true;
      default:
        throw new ValidationError("visible", visible, "parsing failed");
    }
  }

  set visible(visible) {
    if (visible === null) {
      delete this.td.dataset.visible;
      return;
    }

    if (!(typeof visible === typeof true || visible instanceof Boolean)) {
      throw new ValidationError("visible", visible, "not a boolean");
    }
    this.td.dataset.visible = visible;
  }

  /**
   * Get or set the <tt>data-owner</tt> attribute. It must be an integer.
   * It defaults to <tt>null</tt>.
   *
   * @example
   * let cell = new Cell(document.createElement("td")); // <td></td>
   * cell.owner === null; // true
   * cell.owner = 2; // <td data-owner="2"></td>
   * cell.owner = null; // <td></td>
   *
   * @method
   */
  get owner() {
    let owner = this.td.dataset.owner;
    if (owner === undefined) {
      return null;
    }

    let parsed = parseInt(owner);
    if (isNaN(parsed) || parsed < 0) {
      throw new ValidationError("owner", owner, "not a positive integer");
    }
    return parsed;
  }

  set owner(value) {
    if (value === null) {
      delete this.td.dataset.owner;
      return;
    }
    if (Number.isInteger(value) && value >= 0) {
      this.td.dataset.owner = value;
    } else {
      throw new ValidationError("owner", value, "parsing as an integer failed");
    }
  }

  /**
   * Get or set the <tt>data-selected</tt> attribute. It must be a boolean or a
   * Boolean object. It defaults to <tt>false</tt>.
   *
   * @example
   * let cell = new Cell(document.createElement("td")); // <td></td>
   * cell.selected === false; // true
   * cell.selected = true;    // <td data-selected="true"></td>
   * cell.selected = false;   // <td data-selected="false"></td>
   * cell.selected = null;    // <td></td>
   *
   * @method
   */
  get selected() {
    let selected = this.td.dataset.selected;
    if (selected === undefined) {
      return false;
    }
    switch (selected) {
      case "false":
        return false;
      case "true":
        return true;
      default:
        throw new ValidationError(
          "selected",
          selected,
          "parsing as a boolean failed"
        );
    }
  }

  set selected(selected) {
    if (selected === null) {
      delete this.td.dataset.selected;
      return;
    }

    // FIXME: is this too strict? In javascript, maybe we should accept inputs
    // that can be parsed as valid number, like "42"?
    if (!(typeof selected === typeof true || selected instanceof Boolean)) {
      throw new ValidationError("selected", selected, "not a boolean");
    }
    this.td.setAttribute("data-selected", selected);
  }

  /**
   * @return {external:HTMLTableCellElement} The inner &lt;td> element
   */
  html() {
    return this.td;
  }

  /**
   * @return {Number} the index of the cell in the {@link Grid}
   */
  index() {
    let tr = this.td.parentNode;
    // that should never happen, but adding that check to make debugging
    // easier, just in case.
    if (!(tr instanceof HTMLTableRowElement)) {
      throw "Parent of <td> is not a <tr> ???";
    }
    let col = this.td.cellIndex;
    let row = tr.rowIndex;
    let rowLength = tr.cells.length;
    let i = col + row * rowLength;
    return i;
  }
}

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

function isValid(json) {
  let keys = ["height", "width", "players", "tiles", "turn"];
  for (let key of keys) {
    if (!(key in json)) {
      return false;
    }
  }
  // TODO: other validations, for instance check for attributes set to
  // undefined, or that have invalid types
  return true;
}

class CellUpdator {
  constructor(data) {
    this.data = data;
  }

  updateCell(cell) {
    if (this.data === null) {
      cell.type = CellType.Mountain;
      cell.units = null;
      cell.owner = null;
      cell.visible = null;
      return;
    }

    if (!(typeof this.data === "object")) {
      throw "Invalid tile update";
    }

    this.updateType(cell);
    this.updateUnits(cell);
    this.updateOwner(cell);
  }

  updateOwner(cell) {
    if (this.data.owner === undefined) {
      cell.owner = null;
    } else if (Number.isInteger(this.data.owner)) {
      cell.owner = this.data.owner;
    } else {
      throw "Invalid 'owner' in tile update";
    }
  }

  updateUnits(cell) {
    if (this.data.units === undefined) {
      cell.units = null;
    } else if (Number.isInteger(this.data.units)) {
      cell.units = this.data.units;
    } else {
      throw "Invalid 'units' in tile update";
    }
  }

  updateType(cell) {
    try {
      cell.type = CellType.fromString(this.data.kind);
    } catch (_) {
      cell.type = null;
    }
  }
}

class EventHandler {
  /**
   * A  class to handle DOM events.
   */
  constructor(ws, grid) {
    this.ws = ws;
    this.grid = grid;

    // XXX: We cannot do:
    //
    //    `this.grid.table.addEventListener("click", this.click);`
    //
    // because in JS, this is unbount, and when the callback is called, `this`
    // wont be an instance of `EventHandler`. There are multiple ways for
    // solving that:
    //
    // 1. Creating copy of `this` in the current context, and create a lambda
    //    that captures it:
    //
    //        let current_this = this;
    //        this.grid.table.addEventListener("click", current_this.click);
    //
    // 2. Use an arrow function. I think it basically does the same thing than
    //    above. It's less explicit though, so I don't really like it.
    //
    // 3. Use `bind`. It's short and explicit, so we go for this one.
    //
    this.grid.table.addEventListener("click", this.click.bind(this));
    document.body.addEventListener("keydown", this.keydown.bind(this));
  }

  /**
   * Handle a click event: retrieve the {@link Cell} that has been clicked,
   * and select it if it belong to the player that clicked it.
   */
  click(event) {
    let cell = this.getCell(event);
    if (cell === null) {
      return;
    }
    let index = cell.index();
    this.grid.select(index);
  }

  /**
   * Handle a keypress event: if the key correspond to a movement key ("a",
   * "w", "s" or "d"), try to perform the move (see {@link EventHandler#move}).
   *
   */
  keydown(event) {
    let key = event.key.toLowerCase();
    switch (key) {
      case "w":
        this.move(Direction.Up);
        return;
      case "a":
        this.move(Direction.Left);
        return;
      case "d":
        this.move(Direction.Right);
        return;
      case "s":
        this.move(Direction.Down);
        return;
      default:
    }
  }

  /**
   * Check whether a move is valid, and if it is, notify the server about the
   * move, and select the target {@link Cell}.
   */
  move(direction) {
    if (this.grid.selected === null) {
      return;
    }
    let target = this.grid.getNeighborCell(this.grid.selected, direction);
    if (!target) {
      return;
    }
    if (target.type == CellType.Mountain) {
      return;
    }
    let action = {
      type: "move",
      from: this.grid.selected,
      direction: Direction.toString(direction)
    };
    this.send(action);
    this.grid.select(target.index());
  }

  /**
   * Return the {@link Cell} on which an {@link external:Event} occured. If the
   * event did not occur on a {@link Cell}, <tt>null</tt> null is returned.
   *
   * @return {Cell|null} the event target
   * @param {extern:Event} [event]
   */
  getCell(event) {
    // our <td> don't have any child, but `closest` makes things slightly more
    // future proof, in case we want to add children later
    let td = event.target.closest("td");
    if (td === null) {
      return null;
    }
    return new Cell(td);
  }

  /**
   * Send a message to the websocket, and log it in the console.
   *
   * @param {Object} [msg] an object that can be serialized as json.
   */
  send(msg) {
    let string = JSON.stringify(msg);
    console.log(">>> " + string);
    this.ws.send(string);
  }
}

/**
 * Connect to the server using a websocket, register a few event handlers to
 * handle the user's input as well as the messages coming from the server, and
 * update map based on these updates.
 */
function main() {
  let connection = new WebSocket("ws://localhost:8080");

  let grid = new Grid(document.getElementById("grid"));
  connection.onmessage = function(event) {
    console.log("<<< " + event.data);
    let update = JSON.parse(event.data);
    if (!isValid(update)) {
      throw "Invalid message: " + event.data;
    }

    if (grid.length() === 0) {
      grid.init(update.height, update.width);
    }

    for (let i = 0; i < update.tiles.length; i++) {
      let [index, tile] = update.tiles[i];
      let cell = grid.getCell(index);
      let cellUpdator = new CellUpdator(tile);
      cellUpdator.updateCell(cell);
    }
  };

  new EventHandler(connection, grid);
}
