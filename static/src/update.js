"use strict";

import { CellType } from "./celltype.js";

function validate(json) {
  let keys = ["height", "width", "players", "tiles", "turn"];
  for (let key of keys) {
    if (!(key in json)) {
      throw new InvalidUpdate(key + " missing");
    }
  }
  // TODO: other validations, for instance check for attributes set to
  // undefined, or that have invalid types
}

function updateOwner(cell, update) {
  if (update.owner === undefined) {
    cell.owner = null;
  } else if (Number.isInteger(update.owner)) {
    cell.owner = update.owner;
  } else {
    throw new InvalidUpdate("Invalid 'owner' in tile update");
  }
}

function updateUnits(cell, update) {
  if (update.units === undefined) {
    cell.units = null;
  } else if (Number.isInteger(update.units)) {
    cell.units = update.units;
  } else {
    throw new InvalidUpdate("Invalid 'units' in tile update");
  }
}

function updateType(cell, update) {
  try {
    cell.type = CellType.fromString(update.kind);
  } catch (_) {
    cell.type = null;
  }
}

function updateCell(cell, update) {
  if (!(typeof update === "object")) {
    throw new InvalidUpdate("invalid tile");
  }

  updateType(cell, update);
  updateUnits(cell, update);
  updateOwner(cell, update);
}

function updateGrid(grid, update) {
  validate(update);

  if (grid.length() === 0) {
    grid.init(update.height, update.width);
  }

  for (let i = 0; i < update.tiles.length; i++) {
    let [index, tile] = update.tiles[i];
    let cell = grid.getCell(index);
    updateCell(cell, tile);
  }
}

/**
 * Exception thrown upon receiving an invalid update
 */
class InvalidUpdate extends Error {
  constructor(msg) {
    if (msg === undefined) {
      super("Invalid update");
    } else {
      super("Invalid update" + msg);
    }
    this.name = "InvalidUpdate";
  }
}

export { updateGrid, InvalidUpdate };
