// CellType

QUnit.module("CellType", {});

QUnit.test("fromString", function(assert) {
  assert.equal(CellType.fromString("mountain"), CellType.Mountain);
  assert.equal(CellType.fromString("open"), CellType.Open);
  assert.equal(CellType.fromString("city"), CellType.City);
  assert.equal(CellType.fromString("general"), CellType.General);
});

QUnit.test("toString", function(assert) {
  assert.equal(CellType.toString(CellType.Mountain), "mountain");
  assert.equal(CellType.toString(CellType.Open), "open");
  assert.equal(CellType.toString(CellType.City), "city");
  assert.equal(CellType.toString(CellType.General), "general");
});


// Direction

QUnit.module("Direction", {});

QUnit.test("fromString", function(assert) {
  assert.equal(Direction.fromString("up"), Direction.Up);
  assert.equal(Direction.fromString("left"), Direction.Left);
  assert.equal(Direction.fromString("right"), Direction.Right);
  assert.equal(Direction.fromString("down"), Direction.Down);
});

QUnit.test("toString", function(assert) {
  assert.equal(Direction.toString(Direction.Up), "up");
  assert.equal(Direction.toString(Direction.Left), "left");
  assert.equal(Direction.toString(Direction.Right), "right");
  assert.equal(Direction.toString(Direction.Down), "down");
});

