import assert from 'node:assert/strict'

import * as bindingModule from '../index.js'
const binding = bindingModule.default ?? bindingModule

const cases = [
  {
    quote: 'CurvedQuotes',
    contentType: 'text/html',
    input: '<p>"abc"</p>',
    expected: '<p>&ldquo;abc&rdquo;</p>',
  },
  {
    quote: 'CurvedQuotes',
    contentType: 'text/markdown',
    input: '"abc"',
    expected: '“abc”\n',
  },
  {
    quote: 'Guillemets',
    contentType: 'text/html',
    input: '<p>"abc"</p>',
    expected: '<p>&#x300a;abc&#x300b;</p>',
  },
  {
    quote: 'Guillemets',
    contentType: 'text/markdown',
    input: '"abc"',
    expected: '《abc》\n',
  },
]

for (const c of cases) {
  const config = { ...binding.koKr(), quote: c.quote, contentType: c.contentType }
  const output = binding.transform(config, c.input)
  assert.equal(output, c.expected, `quote=${c.quote}, contentType=${c.contentType}`)
}
