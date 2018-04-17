/**
 * A table in the DOM tree.
 * @external HTMLTableElement
 * @see {@link https://developer.mozilla.org/en-US/docs/Web/API/HTMLTableElement HTMLTableElement}
 */

/**
 * Represent the possible types of cell:
 *
 * <pre>
 * CellType.Wall    # represent a wall cell
 * CellType.Open    # represent an empty cell
 * CellType.City    # represent a cell containing a city
 * CellType.General # represent a cell containing a general
 * </pre>
 */
let CellType = Object.freeze({ Wall: 1, Open: 2, City: 3, General: 4 });

/**
 * Exception thrown when setting/reading an invalid "data-" attribute on/from a cell.
 */
function InvalidDataAttribute(attribute, value) {
  this.message = "Invalid data-" + attribute + " attribute: '" + value + "'";
}

class Cell {
  /**
   * A type to manipulate a cell. Underneath, a cell is a &lt;td> DOM node with
   * a bunch of "data-" attributes:
   *
   * <ul>
   * <li>
   *
   *    <tt>data-type</tt>: it represents which type of cell it is and can be
   *    set to "wall", "open", "city" or "general". If not set, it is
   *    considered to be "open". See also: {@link CellType}. It can be set with
   *    {@link Cell#type}
   * </li>
   *
   * <li>
   *    <tt>data-units</tt>: it carries the number of units on a cell and is an
   *    integer greater than or equal to 0
   * </li>
   *
   * <li>
   *    <tt>data-owner</tt>: the owner of the tile. It is a string that can be
   *    empty.
   * </li>
   *
   * <li>
   *    <tt>data-visible</tt>: whether or not the cell if visible. It can be
   *    "true" or "false" and default to "false"
   * </li>
   * </ul>
   *
   * @param {external:HTMLTableCellElement} [td] - A &lt;td> element
   *
   */
  constructor(td) {
    if (td instanceof HTMLTableCellElement) {
      this.cell = td;
    } else if (td === undefined) {
      this.cell = document.createElement("td");
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
   */
  get type() {
    let type = this.cell.dataset.type;
    if (type === undefined) {
      return CellType.Open;
    }
    switch (type) {
      case "open":
        return CellType.Open;
      case "wall":
        return CellType.Wall;
      case "city":
        return CellType.City;
      case "general":
        return CellType.General;
      default:
        throw new InvalidDataAttribute("type", type);
    }
  }

  set type(type) {
    let text = "open";
    switch (type) {
      case CellType.Wall:
        text = "wall";
        break;
      case CellType.Open:
        text = "open";
        break;
      case CellType.City:
        text = "city";
        break;
      case CellType.General:
        text = "general";
        break;
      case null:
        delete this.cell.dataset.type;
        return;
      default:
        throw InvalidDataAttribute("type", type);
    }
    this.cell.setAttribute("data-type", text);
  }

  /**
   *
   * Get or set the number of units as text in the cell. It must an integer
   * greater than or equal to 0. It defaults to <tt>0</tt>.
   *
   */
  get units() {
    let units = this.cell.innerText;
    if (units === "" || units === undefined || units === null) {
      return 0;
    }
    let parsed = parseInt(units);
    if (isNaN(parsed)) {
      throw new InvalidDataAttribute("units", units);
    }
    return parsed;
  }

  set units(number) {
    if (number === null) {
      delete this.cell.dataset.units;
      return;
    }
    if (!Number.isInteger(number)) {
      throw TypeError("Expected integer got " + typeof number);
    }
    this.cell.innerText = number;
  }

  /**
   * Get or set the <tt>data-visible</tt> attribute. It must be a boolean or a
   * Boolean object. It defaults to <tt>false</tt>.
   */
  get visible() {
    let visible = this.cell.dataset.visible;
    if (visible === undefined) {
      return false;
    }
    switch (visible) {
      case "false":
        return false;
      case "true":
        return true;
      default:
        throw new InvalidDataAttribute("visible", visible);
    }
  }

  set visible(visible) {
    if (visible === null) {
      delete this.cell.dataset.visible;
      return;
    }

    // FIXME: is this too strict? In javascript, maybe we should accept inputs
    // that can be parsed as valid number, like "42"?
    if (!(typeof visible === typeof true || visible instanceof Boolean)) {
      throw TypeError("Expected boolean got " + typeof visible);
    }
    this.cell.setAttribute("data-visible", visible);
  }

  /**
   * Get or set the <tt>data-owner</tt> attribute. It must be an integer.
   * It defaults to <tt>null</tt>.
   */
  get owner() {
    let owner = this.cell.dataset.owner;
    if (owner === undefined) {
      return null;
    }
    if (Number.isInteger(owner)) {
      return owner;
    }
    throw new InvalidDataAttribute("owner", owner);
  }

  set owner(value) {
    if (value === null) {
      delete this.cell.dataset.owner;
      return;
    }
    if (Number.isInteger(value)) {
      this.cell.dataset.owner = value;
    } else {
      throw InvalidDataAttribute("owner", value);
    }
  }

  /**
   * @return {external:HTMLTableCellElement} The inner &lt;td> element
   */
  html() {
    return this.cell;
  }

  index() {
    let tr = this.cell.parentNode;
    // that should never happen, but adding that check to make debugging
    // easier, just in case.
    if (!(tr instanceof HTMLTableRowElement)) {
      throw "Parent of <td> is not a <tr> ???";
    }
    let col = this.cell.cellIndex;
    let row = this.tr.rowIndex;
    return col * row;
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
   *
   * Create a new {@link Grid}, that wraps the given {@link
   * external:HTMLTableElement}. A {@link Grid} provide helpers to update
   * individual {@link Cell}s. A {@link Grid} maintain the following invariants:
   *
   * <ul>
   *  <li>each cell of the {@link Grid} is a valid {@link Cell}</li>
   *  <li>each row has the same number of {@link Cell}s</li>
   * </ul>
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
  }

  /**
   * @return {number} the number of rows.
   */
  height() {
    return this.table.rows.length;
  }

  /**
   * @return {number} the number of columns.
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
   */
  length() {
    return this.height() * this.width();
  }

  /**
   * @return {Cell} the cell at the given index.
   * @throws {RangeError} is the index is invalid.
   */
  getCell(index) {
    if (!this.isValidIndex(index)) {
      throw new RangeError("the grid has only " + this.length() + " cells");
    }
    let coordinates = this.coordinates(index);
    let row = this.table.rows[coordinates.row];
    let cell = row.cells[coordinates.col];
    return new Cell(cell);
  }

  /**
   * @param {number} index
   * @return {boolean} whether the given index is valid, or out-of-bounds
   */
  isValidIndex(index) {
    return index < this.length();
  }

  /**
   * @param {number} index
   * @return {Coordinates} the cartesian coordinates corresponding to the given index
   */
  coordinates(index) {
    return new Coordinates(
      Math.floor(index / this.width()),
      index % this.width()
    );
  }

  /**
   * Create a grid of empty cell with the given number of row and columns.
   *
   * @param {n_rows} number of rows
   * @param {n_cols} number of columns
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
   * Remove all the cell from the grid, leaving only &lt;tabl>&lt;/table> in the DOM.
   */
  clear() {
    while (this.table.firstChild) {
      this.table.firstChild.remove();
    }
  }
}

function isValid(json) {
  let keys = ["height", "width", "players", "tiles", "turn"];
  for (let i = 0; i < keys.length; i++) {
    if (!(keys[i] in json)) {
      console.warn(keys[i] + "missing from message");
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
      cell.type = CellType.Wall;
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
    switch (this.data.kind) {
      case "general":
        cell.type = CellType.General;
        break;
      case "fortress":
        cell.type = CellType.City;
        break;
      case "normal":
        cell.type = CellType.Open;
        break;
      case "wall":
        cell.type = CellType.Wall;
        break;
      case undefined:
        cell.type = null;
        break;
    }
  }
}

class EventHandler {
  constructor(ws, grid) {
    this.ws = ws;
    this.grid = grid;
  }

  click(event) {
    let cell = this.getCell(event);
    if (cell === null) {
      return;
    }
  }

  getCell(event) {
    // our <td> don't have any child, but `closest` makes things slightly more
    // future proof, in case we want to add children later
    let td = event.target.closest("td");
    if (td === null) {
      return null;
    }
    return new Cell(td);
  }
}

function main() {
  let connection = new WebSocket("ws://localhost:8080");

  let grid = new Grid(document.getElementById("grid"));

  connection.onmessage = function(event) {
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
}

main();
