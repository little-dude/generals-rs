"use strict";

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

export { Direction };
