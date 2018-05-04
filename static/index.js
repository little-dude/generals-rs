"use strict";

import { Grid } from "./grid";
import { updateGrid } from "./update";
import { InputEventsHandler } from "./input";

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
    updateGrid(grid, update);
  };

  // This is kind of weird. There's no blocking call anywhere in this function.
  // Yet, when it returns, it seems that the event handler and the websocket
  // are not destroyed, so everthing works correctly.
  new InputEventsHandler(connection, grid);
}
