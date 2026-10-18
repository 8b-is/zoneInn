const assert = require('node:assert/strict');
const { readFileSync } = require('node:fs');
const { join } = require('node:path');
const { test } = require('node:test');
const vm = require('node:vm');

// Exercise the shipped handler with fake DOM/Bluetooth objects. No radio,
// permission prompt, network request or browser process is used by these tests.
class Element {
  constructor() { this.children = []; this.htmlWrites = []; this.text = ''; }
  set textContent(value) { this.text = String(value); this.children = []; }
  get textContent() { return this.text + this.children.map(child => child.textContent).join(''); }
  set innerHTML(value) { this.htmlWrites.push(value); this.textContent = value; }
  replaceChildren(...children) { this.text = ''; this.children = children; }
}

for (const page of ['web/index.html', 'docs/index.html']) {
  for (const name of [
    'Soundcore Select 4 GO <img src=x onerror="unexpected()">',
    'Speaker <b>with a literal name</b>',
  ]) {
    test(`${page} renders device name as text: ${name}`, async () => {
      const html = readFileSync(join(__dirname, '..', page), 'utf8');
      const core = html.match(/<script id="zi-core">([\s\S]*?)<\/script>/);
      assert.ok(core, 'shipped core script exists');
      const start = html.indexOf('  async function connectWebBluetooth(');
      const end = html.indexOf('  async function refreshAudioOutputSinks(', start);
      assert.ok(start >= 0 && end > start, 'shipped Bluetooth handler exists');
      const status = new Element();
      const header = new Element();
      const context = vm.createContext({
        navigator: { bluetooth: { requestDevice: async () => ({ name }) } },
        stBt: status,
        hdrSpeaker: header,
        document: { createElement: () => new Element() },
        refreshAudioOutputSinks: async () => {},
      });
      context.window = context;
      vm.runInContext(core[1], context);
      context.core = context.zoneInnCore;
      vm.runInContext(html.slice(start, end), context);
      await vm.runInContext('connectWebBluetooth(true)', context);
      assert.ok(status.textContent.includes(name), 'literal device name remains visible');
      assert.equal(status.htmlWrites.length, 0, 'device status must never parse HTML');
      if (name.startsWith('Soundcore')) {
        assert.equal(status.children[0].className, 'badge-FirstClass');
        assert.ok(header.textContent.includes('Soundcore Select 4 GO'));
      }
    });
  }
}
