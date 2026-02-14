const { readdirSync } = require('node:fs')
const { join } = require('node:path')

const nodeBinary = readdirSync(__dirname).find((name) => name.endsWith('.node'))
if (!nodeBinary) {
  throw new Error('Cannot find built .node binary. Run `npm run build` first.')
}

module.exports = require(join(__dirname, nodeBinary))
