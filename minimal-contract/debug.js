#!/usr/bin/env node
const { spawn } = require("child_process");
const path = require("path");

const oylPath = path.join(__dirname, "oyl-sdk/bin/oyl.js");
const args = ["alkane", "execute", "-data", "2,702,1", "-p", "oylnet"];

const child = spawn("node", [oylPath, ...args]);

child.stdout.on("data", (data) => {
  require("fs").appendFileSync("../debug_output.txt", data);
});

child.stderr.on("data", (data) => {
  require("fs").appendFileSync("../debug_output.txt", data);
});

child.on("close", (code) => {
  require("fs").appendFileSync("../debug_output.txt", `\nProcess exited with code ${code}\n`);
});
