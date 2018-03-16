QUnit.module("Cell", {
  beforeEach: function(assert) {
    this.grid = new Grid(document.getElementById("grid"));
    this.grid.init(3, 3);
    this.cell = this.grid.getCell(4);
    this.td = this.cell.td;
  }
});

QUnit.test("default properties", function(assert) {
  assert.equal(this.cell.units, null);
  assert.equal(this.cell.type, CellType.Open);
  assert.equal(this.cell.visible, false);
  assert.equal(this.cell.owner, null);
});

QUnit.test("index", function(assert) {
  for (let i = 0; i < 9; i++) {
    assert.equal(this.grid.getCell(i).index(), i);
  }
});

QUnit.test("html", function(assert) {
  assert.equal(this.cell.html(), this.td);
});

QUnit.test("units", function(assert) {
  let cell = this.cell;
  let td = this.td
  assert.equal(cell.units, null);
  assert.equal(td.innerText, "");

  cell.units = 99;
  assert.equal(cell.units, 99);
  assert.equal(td.innerText, "99");

  td.innerText = "42";
  assert.equal(cell.units, 42);

  cell.units = null;
  assert.equal(cell.units, null);
  assert.equal(td.innerText, "");

  function assertInvalid(value) {
    assert.throws(
      function() {
        cell.units = value;
      },
      ValidationError,
      "Validation error thrown for " + value);
  }
  assertInvalid(true);
  assertInvalid("");
  assertInvalid(-1);
  assertInvalid(1.3);
  assertInvalid(undefined);
});

QUnit.test("type", function(assert) {
  let cell = this.cell;
  let td = this.td
  assert.equal(cell.type, CellType.Open);
  assert.equal(td.dataset.type, undefined);

  td.dataset.type = "mountain";
  assert.equal(cell.type, CellType.Mountain);

  cell.type = CellType.City;
  assert.equal(td.dataset.type, "city");
  assert.equal(cell.type, CellType.City);

  cell.type = null;
  assert.equal(cell.type, CellType.Open);
  assert.equal(td.dataset.type, undefined);

  function assertInvalid(value) {
    assert.throws(
      function() {
        cell.type = value;
      },
      ValidationError,
      "Validation error thrown for " + value);
  }
  assertInvalid("city");
  assertInvalid(0);
  assertInvalid(5);
  assertInvalid(undefined);
});

QUnit.test("visible", function(assert) {
  let cell = this.cell;
  let td = this.td

  assert.equal(cell.visible, false);
  assert.equal(td.dataset.visible, undefined);

  td.dataset.visible = true;
  assert.equal(cell.visible, true);
  assert.equal(td.dataset.visible, "true");

  td.dataset.visible = false;
  assert.equal(cell.visible, false);
  assert.equal(td.dataset.visible, "false");

  this.cell.visible = null;
  assert.equal(this.cell.visible, false);
  assert.equal(this.td.dataset.visible, undefined);

  function assertInvalid(value) {
    assert.throws(
      function() {
        cell.visible = value;
      },
      ValidationError,
      "Validation error thrown for " + value);
  }

  assertInvalid("true");
  assertInvalid("false");
  assertInvalid(0);
  assertInvalid(1);
  assertInvalid(undefined);
});

QUnit.test("owner", function(assert) {
  let cell = this.cell;
  let td = this.td

  assert.equal(cell.owner, null);
  assert.equal(td.dataset.owner, undefined);

  td.dataset.owner = 0;
  assert.equal(cell.owner, 0);

  cell.owner = 5;
  assert.equal(td.dataset.owner, "5");
  assert.equal(cell.owner, 5);

  cell.owner = null;
  assert.equal(cell.owner, null);
  assert.equal(td.dataset.owner, undefined);

  function assertInvalid(value) {
    assert.throws(
      function() {
        cell.owner = value;
      },
      ValidationError,
      "Validation error thrown for " + value);
  }
  assertInvalid("1");
  assertInvalid(-1);
  assertInvalid(true);
  assertInvalid(1.3);
  assertInvalid(undefined);
});

QUnit.test("selected", function(assert) {
  let cell = this.cell;
  let td = this.td

  assert.equal(cell.selected, false);
  assert.equal(td.dataset.selected, undefined);

  td.dataset.selected = true;
  assert.equal(cell.selected, true);
  assert.equal(td.dataset.selected, "true");

  td.dataset.selected = false;
  assert.equal(cell.selected, false);
  assert.equal(td.dataset.selected, "false");

  this.cell.selected = null;
  assert.equal(this.cell.selected, false);
  assert.equal(this.td.dataset.selected, undefined);

  function assertInvalid(value) {
    assert.throws(
      function() {
        cell.selected = value;
      },
      ValidationError,
      "Validation error thrown for " + value);
  }

  assertInvalid("true");
  assertInvalid("false");
  assertInvalid(0);
  assertInvalid(1);
  assertInvalid(undefined);
});
