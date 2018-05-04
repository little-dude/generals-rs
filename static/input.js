"use strict";

/**
 * A DOM event.
 * @external Event
 * @see {@link https://developer.mozilla.org/en-US/docs/Web/API/Event}
 */

import { CellType } from "./celltype.js";
import { Direction } from "./direction.js";
import { Cell } from "./cell.js";

class InputEventsHandler {
  /**
   * A  class to handle users input events.
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

export { InputEventsHandler };
