import assert from 'node:assert/strict'

import * as bindingModule from '../index.js'
const binding = bindingModule.default ?? bindingModule

const output = binding.transform(binding.koKr(), '<p>"abc"</p>')
assert.ok(output.includes('&ldquo;abc&rdquo;'))

const markdownConfig = { ...binding.koKr(), contentType: 'text/markdown' }
const markdownOutput = binding.transform(markdownConfig, '"abc"')
assert.ok(markdownOutput.includes('“abc”'))
assert.ok(!markdownOutput.includes('&ldquo;'))
