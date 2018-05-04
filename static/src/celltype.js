"use strict";

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

export { CellType };
