"use strict";

/**
 * A cell in a <tt>&lt;tr></tt> DOM element.
 * @external HTMLTableCellElement
 * @see {@link https://developer.mozilla.org/en-US/docs/Web/API/HTMLTableCellElement HTMLTableCellElement}
 */

import { CellType } from "./celltype.js";

/**
 * Exception thrown when setting/reading an invalid attribute on/from a cell.
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

export { Cell, ValidationError };
