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

test('custom dictionary overrides built-in entries', () => {
  const config = {
    contentType: 'text/html',
    hanja: {
      rendering: 'HangulOnly',
      reading: {
        initialSoundLaw: false,
        useDictionaries: ['kr-stdict'],
        dictionary: { '漢字': '커스텀' },
      },
    },
  }
  const output = binding.transform(config, '<p>漢字</p>')
  assert.ok(output.includes('커스텀'), `expected custom reading but got: ${output}`)
})

test('transform works without dictionary field (backward compat)', () => {
  const config = {
    contentType: 'text/html',
    hanja: {
      rendering: 'HangulOnly',
      reading: {
        initialSoundLaw: true,
        useDictionaries: ['kr-stdict'],
      },
    },
  }
  const output = binding.transform(config, '<p>漢字</p>')
  assert.ok(output.includes('한자'), `expected default reading but got: ${output}`)
})
