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
  assert.ok(output.includes('“abc”'))
})
