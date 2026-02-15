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

test('transform output differs by content type for koKr', () => {
  const htmlOutput = binding.transform(binding.koKr(), '<p>"abc"</p>')
  assert.ok(htmlOutput.includes('&ldquo;abc&rdquo;'))

  const markdownConfig = { ...binding.koKr(), contentType: 'text/markdown' }
  const markdownOutput = binding.transform(markdownConfig, '"abc"')
  assert.ok(markdownOutput.includes('“abc”'))
  assert.ok(!markdownOutput.includes('&ldquo;'))
})
