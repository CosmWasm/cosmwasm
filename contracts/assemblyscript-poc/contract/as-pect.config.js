/* eslint-env node */
module.exports = {
  include: ["src/**/*.spec.ts"],
  /** Compiler flags */
  flags: {
    "--validate": [],
    "--debug": [],
    /** Required for testing */
    "--binaryFile": ["output.wasm"],
    // "--textFile": ["output.wat"],
    "--runtime": ["stub"],
    "--baseDir": process.cwd(),
  },
};
