const fs = require("fs"); const path = require("path"); const oylPath = path.join(__dirname, "oyl-sdk/bin/oyl.js"); fs.writeFileSync("../check_result.txt", `File exists: ${fs.existsSync(oylPath)}`);
