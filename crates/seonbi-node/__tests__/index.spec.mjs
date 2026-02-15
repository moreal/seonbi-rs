import test from 'node:test'
import assert from 'node:assert/strict'

const binding = await import('../index.js')

test('koKr preset is available after build', () => {
  const cfg = binding.koKr()
  assert.equal(typeof cfg, 'object')
  assert.equal(cfg.preset, 'ko-kr')
})

test('transform performs quote replacement', () => {
  const output = binding.transform(binding.koKr(), '<p>"abc"</p>')
  assert.ok(output.includes('&ldquo;abc&rdquo;'))
})

test('transform covers quote/contentType matrix for koKr', () => {
  const cases = [
    {
      quote: 'CurvedQuotes',
      contentType: 'text/html',
      input: '<p>"abc"</p>',
      expected: '<p>&ldquo;abc&rdquo;</p>',
    },
    {
      quote: 'CurvedQuotes',
      contentType: 'application/xhtml+xml',
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
      quote: 'CurvedQuotes',
      contentType: 'text/plain',
      input: '"abc"',
      expected: '“abc”',
    },
    {
      quote: 'Guillemets',
      contentType: 'text/html',
      input: '<p>"abc"</p>',
      expected: '<p>&#x300a;abc&#x300b;</p>',
    },
    {
      quote: 'Guillemets',
      contentType: 'application/xhtml+xml',
      input: '<p>"abc"</p>',
      expected: '<p>&#x300a;abc&#x300b;</p>',
    },
    {
      quote: 'Guillemets',
      contentType: 'text/markdown',
      input: '"abc"',
      expected: '《abc》\n',
    },
    {
      quote: 'Guillemets',
      contentType: 'text/plain',
      input: '"abc"',
      expected: '《abc》',
    },
  ]

  for (const c of cases) {
    const config = { ...binding.koKr(), quote: c.quote, contentType: c.contentType }
    const output = binding.transform(config, c.input)
    assert.equal(output, c.expected, `quote=${c.quote}, contentType=${c.contentType}`)
  }
})
