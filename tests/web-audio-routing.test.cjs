const assert = require('node:assert/strict');
const { readFileSync } = require('node:fs');
const { join } = require('node:path');
const { test } = require('node:test');
const vm = require('node:vm');

// Run the shipped routing handler with synthetic browser APIs, without audio,
// permissions, networking, device access or third-party dependencies.
const cases = [
  { name: 'idle without routing APIs', context: null, media: 'unsupported', playing: false },
  { name: 'idle media configuration cannot confirm future synth routing', context: null, media: 'ok', playing: false, configured: true },
  { name: 'idle context configuration is not live playback', context: 'ok', media: 'ok', playing: false, configured: true },
  { name: 'synth without routing APIs', context: 'unsupported', media: 'unsupported' },
  { name: 'inactive media success cannot confirm synth routing', context: 'unsupported', media: 'ok', configured: true },
  { name: 'supported synth does not require media support', context: 'ok', media: 'unsupported', routed: true, via: 'Web Audio' },
  { name: 'synth rejection preserves confirmed speaker', context: 'reject', media: 'ok', rejection: true },
  { name: 'synth success survives unrelated media rejection', context: 'ok', media: 'reject', routed: true, via: 'Web Audio', rejection: true },
  { name: 'direct media uses media sink despite unsupported context', context: 'unsupported', media: 'ok', paused: false, routed: true, via: 'media playback' },
  { name: 'context success cannot confirm rejected direct media', context: 'ok', media: 'reject', paused: false, rejection: true },
  { name: 'direct media missing API preserves confirmed speaker', context: 'ok', media: 'unsupported', paused: false },
  { name: 'connected media uses context destination', context: 'ok', media: 'unsupported', paused: false, connected: true, routed: true, via: 'Web Audio' },
  { name: 'connected media cannot use media success to hide context rejection', context: 'reject', media: 'ok', paused: false, connected: true, rejection: true },
  { name: 'both APIs succeed for connected media', context: 'ok', media: 'ok', paused: false, connected: true, routed: true, via: 'Web Audio' },
  { name: 'captured but disconnected media cannot confirm routing', context: 'ok', media: 'ok', paused: false, captured: true },
  { name: 'missing active context cannot confirm synth routing', context: null, media: 'ok' },
];

for (const page of ['web/index.html', 'docs/index.html']) {
  const html = readFileSync(join(__dirname, '..', page), 'utf8');
  const start = html.indexOf('  async function applyAudioSink(');
  const end = html.indexOf('  btnBtSelect4.addEventListener(', start);
  assert.ok(start >= 0 && end > start);
  for (const scenario of cases) {
    test(`${page}: ${scenario.name}`, async () => {
      const calls = [];
      function device(name, behavior) {
        if (behavior === null) return null;
        const value = {};
        if (behavior !== 'unsupported') value.setSinkId = async id => {
          calls.push([name, id]);
          if (behavior === 'reject') throw new Error(name + ' denied');
        };
        return value;
      }
      const media = device('media', scenario.media);
      media.paused = scenario.paused !== false;
      const status = { textContent: '' };
      const header = { textContent: 'Previously confirmed speaker' };
      const context = vm.createContext({
        sinkRoutingQueue: Promise.resolve(),
        audioCtx: device('context', scenario.context), streamAudio: media,
        playing: scenario.playing !== false, mediaElementConnected: !!scenario.connected,
        mediaElementSource: scenario.connected || scenario.captured ? {} : null,
        sinkSelect: { selectedIndex: 0, options: [{ value: 'test-sink', text: 'Test speaker <literal>' }] },
        stBt: status, hdrSpeaker: header,
      });
      vm.runInContext(html.slice(start, end), context);
      await vm.runInContext('applyAudioSink("test-sink")', context);
      assert.equal(status.textContent.startsWith('audio routed'), !!scenario.routed);
      assert.equal(header.textContent, scenario.routed ? 'Test speaker <literal>' : 'Previously confirmed speaker');
      if (scenario.routed) assert.ok(status.textContent.includes('(' + scenario.via + ')'));
      if (scenario.rejection) assert.match(status.textContent, /rejected: (context|media) denied/);
      if (scenario.playing === false) assert.match(status.textContent, /start playback to verify routing/);
      if (scenario.configured) assert.match(status.textContent, /configured/);
      assert.equal(calls.length, [scenario.context, scenario.media].filter(x => x === 'ok' || x === 'reject').length);
      assert.ok(calls.every(([, id]) => id === 'test-sink'));
    });
  }
  for (const playback of ['stream', 'favourite']) {
    for (const failsAt of ['none', 'create', 'connect']) {
      test(`${page}: ${playback} graph connection ${failsAt}`, async () => {
        const graphStart = html.indexOf(playback === 'stream'
          ? '  async function startStreamUrl(' : '  async function playFavouriteTrack(');
        const graphEnd = html.indexOf(playback === 'stream'
          ? '  async function tuneAndPlay(' : '  // Auto-advance', graphStart);
        assert.ok(graphStart >= 0 && graphEnd > graphStart);
        const media = { paused: true, play: async () => { media.paused = false; } };
        const gain = {};
        const context = vm.createContext({
          streamAudio: media, mediaElementSource: null, mediaElementConnected: false,
          streamCorsBlocked: false, masterGain: gain, stMode: { textContent: '' },
          ensureAudio: async () => {}, stopActiveSynth: () => {},
          core: { FAVOURITE_MUSICS: [{ slug: 'synthetic', emoji: '', title: 'test', mp3_url: 'synthetic-url' }] },
          favForceSynth: false, hdrStation: { textContent: '' }, dialCards: [], renderFavCrate: () => {},
          btnPlay: { textContent: '', classList: { add: () => {} } }, updateMediaSession: () => {},
          audioCtx: { createMediaElementSource: element => {
            assert.equal(element, media);
            if (failsAt === 'create') throw new Error('capture failed');
            return { connect: target => {
              assert.equal(target, gain);
              if (failsAt === 'connect') throw new Error('connection failed');
            } };
          } },
        });
        vm.runInContext(html.slice(graphStart, graphEnd), context);
        await vm.runInContext(playback === 'stream'
          ? 'startStreamUrl("synthetic-url", "synthetic station")' : 'playFavouriteTrack(0)', context);
        assert.equal(context.mediaElementConnected, failsAt === 'none');
        assert.equal(context.streamCorsBlocked, failsAt !== 'none');
        assert.equal(context.mediaElementSource === null, failsAt === 'create');
      });
    }
  }
  test(`${page}: consecutive sink selections finish in requested order`, async () => {
    const calls = [];
    let release;
    const gate = new Promise(resolve => { release = resolve; });
    const context = vm.createContext({
      sinkRoutingQueue: Promise.resolve(), playing: true, mediaElementSource: null, mediaElementConnected: false,
      audioCtx: { setSinkId: async id => { calls.push('context:' + id); if (id === 'first') await gate; } },
      streamAudio: { paused: true, setSinkId: async id => { calls.push('media:' + id); } },
      sinkSelect: { selectedIndex: 0, options: [{ value: 'first', text: 'First speaker' }, { value: 'second', text: 'Second speaker' }] },
      stBt: { textContent: '' }, hdrSpeaker: { textContent: 'Old speaker' },
    });
    vm.runInContext(html.slice(start, end), context);
    const first = vm.runInContext('applyAudioSink("first")', context);
    await Promise.resolve();
    context.sinkSelect.selectedIndex = 1;
    const second = vm.runInContext('applyAudioSink("second")', context);
    await Promise.resolve();
    try { assert.deepEqual(calls, ['context:first']); } finally { release(); }
    await Promise.all([first, second]);
    assert.deepEqual(calls, ['context:first', 'media:first', 'context:second', 'media:second']);
    assert.equal(context.hdrSpeaker.textContent, 'Second speaker');
  });
  test(`${page}: playback paused during sink request cannot announce live routing`, async () => {
    let release;
    const gate = new Promise(resolve => { release = resolve; });
    const context = vm.createContext({
      sinkRoutingQueue: Promise.resolve(), playing: true, mediaElementSource: null, mediaElementConnected: false,
      audioCtx: { setSinkId: () => gate }, streamAudio: { paused: true },
      sinkSelect: { selectedIndex: 0, options: [{ text: 'New speaker' }] },
      stBt: { textContent: '' }, hdrSpeaker: { textContent: 'Old speaker' },
    });
    vm.runInContext(html.slice(start, end), context);
    const pending = vm.runInContext('applyAudioSink("new")', context);
    context.playing = false;
    release();
    await pending;
    assert.equal(context.hdrSpeaker.textContent, 'Old speaker');
    assert.ok(!context.stBt.textContent.startsWith('audio routed'));
  });
  test(`${page}: confirmed label matches requested device, not stale selection`, async () => {
    const context = vm.createContext({
      sinkRoutingQueue: Promise.resolve(), playing: true, mediaElementSource: null, mediaElementConnected: false,
      audioCtx: { setSinkId: async () => {} }, streamAudio: { paused: true },
      sinkSelect: { selectedIndex: 0, options: [
        { value: 'old', text: 'Old speaker' }, { value: 'new', text: 'New speaker' },
      ] },
      stBt: { textContent: '' }, hdrSpeaker: { textContent: 'Old speaker' },
    });
    vm.runInContext(html.slice(start, end), context);
    await vm.runInContext('applyAudioSink("new")', context);
    assert.equal(context.hdrSpeaker.textContent, 'New speaker');
  });
}
