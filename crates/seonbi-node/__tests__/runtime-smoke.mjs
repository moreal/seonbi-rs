import assert from 'node:assert/strict'

import binding from '../index.js'

const output = binding.transform(binding.koKr(), '<p>"abc"</p>')
assert.ok(output.includes('“abc”'))
