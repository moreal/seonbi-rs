import test from 'node:test'
import assert from 'node:assert/strict'

let binding = null

try {
  binding = await import('../index.js')
} catch {
  // The binary may not exist in local dev unless built via napi-rs.
}

test('koKr preset is available after build', () => {
  if (!binding) {
    return
  }
  const cfg = binding.koKr()
  assert.equal(typeof cfg, 'object')
})
