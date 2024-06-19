const path = require("path");
const { generateIdl } = require("@metaplex-foundation/shank-js");

const idlDir = path.join(__dirname, "..", "idls");
const binaryInstallDir = path.join(__dirname, "..", ".crates");
const programDir = path.join(__dirname, "..", "programs");

generateIdl({
  generator: "shank",
  programName: "mplx_rewards",
  programId: "BF5PatmRTQDgEKoXR7iHRbkibEEi83nVM38cUKWzQcTR",
  idlDir,
  binaryInstallDir,
  programDir: path.join(programDir, "rewards"),
});
