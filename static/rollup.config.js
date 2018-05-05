export default [
  {
    input: "src/index.js",
    output: {
      file: "bundle.js",
      format: "iife"
    }
  },
  {
    external: ["qunit"],
    input: "tests/cell.js",
    output: {
      file: "tests/bundles/cell.js",
      format: "iife"
    }
  },
  {
    external: ["qunit"],
    input: "tests/grid.js",
    output: {
      file: "tests/bundles/grid.js",
      format: "iife"
    }
  },
  {
    external: ["qunit"],
    input: "tests/misc.js",
    output: {
      file: "tests/bundles/misc.js",
      format: "iife"
    }
  }
];
