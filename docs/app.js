// The charts, drawn from docs/data/board.json.
//
// Hand written SVG and no library, for the same reason the harness has no dependencies: a benchmark
// whose whole claim is that its numbers can be trusted should not ask the reader to trust a
// megabyte of somebody else's JavaScript to see them. Everything here is one file, readable, and it
// degrades to the tables underneath if it breaks.

'use strict'

const COLOURS = {
  'rudb': 'var(--rudb)',
  'duckdb': 'var(--duckdb)',
  'duckdb-pinned': 'var(--duckdb-pinned)',
  'clickhouse-local': 'var(--clickhouse)',
  'clickhouse-server': 'var(--clickhouse)',
  'datafusion': 'var(--datafusion)',
  'polars': 'var(--polars)',
}

// What each measure is called, whether less of it is better, and how it is written out. Kept in one
// place so that a chart, a table header and an axis cannot drift apart from each other.
const MEASURES = {
  hot_ms: { label: 'Hot query time', unit: 'time', note: 'What the engine itself says the queries took, added over the hot runs, median of five. This is the column to read when comparing engines.' },
  wall_ms: { label: 'Wall time around the run', unit: 'time', note: 'The clock this harness holds around the whole subprocess, so it also pays for starting a process, linking it, opening a database and printing the answer. On a small rung most of this column is the process rather than the engine, which is exactly why the chart above reads the engine’s own clock instead.' },
  cold_ms: { label: 'Cold query time', unit: 'time', note: 'The first run of each query. Where the machine would not let the harness drop the page cache, this is a warm number taken before the others rather than a first pass off the device, and the run report for that rung says which it was.' },
  peak: { label: 'Peak resident set', unit: 'bytes', note: 'The worst peak resident set any query in the suite reached. This is the other half of the goal: an engine that is fast because it read everything into memory has not won anything.' },
  cpu_ms: { label: 'CPU over the hot runs', unit: 'time', note: 'CPU seconds against the wall clock beside it. An engine that uses eight cores to be twice as fast as one that uses one is a different result from the wall clock alone, and on a 32 thread box the wall clock hides it.' },
  load_ms: { label: 'Load time', unit: 'time', note: 'What it cost to build the data in the engine’s own format. Zero for an engine that reads the Parquet where it lies, which is not a fast load: it is a decode paid again inside every query time in the first chart.' },
  disk: { label: 'On disk after the load', unit: 'bytes', note: 'What the data takes after the load. A cell marked ¶ is the source Parquet every engine read rather than a format of the engine’s own, so it is the same number for every engine that has none.' },
}

const state = { board: null, suite: null, measure: 'hot_ms', log: true, off: new Set() }

const $ = (id) => document.getElementById(id)

function time (ms) {
  if (ms === null || ms === undefined) return null
  if (ms < 1) return `${(ms * 1000).toFixed(0)}µs`
  if (ms < 1000) return `${ms.toFixed(ms < 10 ? 1 : 0)}ms`
  return `${(ms / 1000).toFixed(ms < 10000 ? 2 : 1)}s`
}

function size (n) {
  if (n === null || n === undefined) return null
  const units = ['B', 'KiB', 'MiB', 'GiB', 'TiB']
  let at = 0
  let value = n
  while (value >= 1024 && at < units.length - 1) { value /= 1024; at += 1 }
  return `${at === 0 ? value : value.toFixed(2)} ${units[at]}`
}

function written (value, measure) {
  return MEASURES[measure].unit === 'bytes' ? size(value) : time(value)
}

function grouped (n) {
  return n === null || n === undefined ? '' : n.toLocaleString('en-US')
}

function ladder () {
  return state.board.ladders.find((l) => `${l.suite} on ${l.machine}` === state.suite)
}

function shown (ladder) {
  return ladder.engines.filter((e) => !state.off.has(e))
}

function value (rung, engine, measure) {
  const column = rung.columns.find((c) => c.engine === engine)
  if (!column) return null
  const got = column[measure]
  return got === null || got === undefined ? null : got
}

// An SVG element with its attributes, which is the whole of what this page needs from a framework.
function svg (tag, attrs, text) {
  const node = document.createElementNS('http://www.w3.org/2000/svg', tag)
  for (const [key, val] of Object.entries(attrs || {})) node.setAttribute(key, val)
  if (text !== undefined) node.textContent = text
  return node
}

// Where a value sits on the axis, in the range zero to one.
//
// A log scale by default and it is not a decoration: a ladder spans a thousand rows to ten million,
// and on a linear axis every rung but the largest is a line on the floor. The zero point of the log
// is the smallest value actually charted rather than zero, because a log has no zero and picking one
// silently would be picking how tall the smallest bar is.
function fraction (value, low, high) {
  if (!state.log) return high > 0 ? value / high : 0
  if (value <= 0 || low <= 0) return 0
  return Math.log(value / low) / Math.log(high / low)
}

function bars () {
  const at = $('bars')
  at.replaceChildren()
  const board = ladder()
  const engines = shown(board)
  const measure = state.measure
  const values = []
  for (const rung of board.rungs) {
    for (const engine of engines) {
      const got = value(rung, engine, measure)
      if (got !== null && got > 0) values.push(got)
    }
  }
  if (!values.length) {
    at.append(Object.assign(document.createElement('p'), { className: 'note', textContent: 'Nothing was measured here.' }))
    return
  }
  const high = Math.max(...values)
  const low = Math.min(...values)
  const rows = board.rungs.length
  const bar = 18
  const gap = 6
  const head = 26
  const block = head + engines.length * (bar + gap) + 18
  const height = rows * block + 20
  const left = 130
  const right = 90
  const width = 900
  const chart = svg('svg', { viewBox: `0 0 ${width} ${height}`, role: 'img' })
  chart.append(svg('title', {}, `${MEASURES[measure].label} for every engine at every size of ${board.suite}`))
  let y = 10
  for (const rung of board.rungs) {
    chart.append(svg('line', { class: 'grid', x1: 0, x2: width, y1: y, y2: y }))
    const rows = rung.rows ? `${grouped(rung.rows)} rows` : ''
    chart.append(svg('text', { class: 'rung', x: 0, y: y + 18 }, `${rung.size}${rows ? `  ·  ${rows}` : ''}`))
    y += head
    for (const engine of engines) {
      const got = value(rung, engine, measure)
      chart.append(svg('text', { class: 'axis', x: left - 8, y: y + bar - 5, 'text-anchor': 'end' }, engine))
      if (got === null) {
        chart.append(svg('text', { class: 'missing', x: left + 2, y: y + bar - 5 }, 'not run'))
      } else if (got === 0) {
        chart.append(svg('text', { class: 'missing', x: left + 2, y: y + bar - 5 }, measure === 'load_ms' ? 'reads the Parquet where it lies' : 'not read'))
      } else {
        const span = Math.max(2, fraction(got, low, high) * (width - left - right))
        chart.append(svg('rect', { x: left, y: y, width: span, height: bar, rx: 2, fill: COLOURS[engine] || 'var(--dim)' }))
        chart.append(svg('text', { class: 'value', x: left + span + 6, y: y + bar - 5 }, written(got, measure)))
      }
      y += bar + gap
    }
    y += 12
  }
  at.append(chart)
}

function trend () {
  const at = $('trend')
  at.replaceChildren()
  const board = ladder()
  const engines = shown(board).filter((e) => e !== 'duckdb')
  const rungs = board.rungs
  if (!engines.length || rungs.length < 2) {
    at.append(Object.assign(document.createElement('p'), { className: 'note', textContent: 'Two rungs and an engine that is not the reference are what this chart is made of.' }))
    return
  }
  const ratios = []
  for (const rung of rungs) {
    for (const engine of engines) {
      const got = value(rung, engine, 'ratio')
      if (got) ratios.push(got)
    }
  }
  const high = Math.max(1.1, ...ratios)
  const low = Math.min(0.9, ...ratios)
  const width = 900
  const height = 360
  const pad = { top: 20, right: 120, bottom: 46, left: 56 }
  const plot = { w: width - pad.left - pad.right, h: height - pad.top - pad.bottom }
  const chart = svg('svg', { viewBox: `0 0 ${width} ${height}`, role: 'img' })
  chart.append(svg('title', {}, `Hot query time as a multiple of DuckDB's, across the sizes of ${board.suite}`))
  const x = (index) => pad.left + (rungs.length === 1 ? plot.w / 2 : (index / (rungs.length - 1)) * plot.w)
  const y = (ratio) => pad.top + plot.h - (Math.log(ratio / low) / Math.log(high / low)) * plot.h
  for (const tick of ticks(low, high)) {
    const at = y(tick)
    if (at < pad.top || at > pad.top + plot.h) continue
    chart.append(svg('line', { class: tick === 1 ? 'one' : 'grid', x1: pad.left, x2: pad.left + plot.w, y1: at, y2: at }))
    chart.append(svg('text', { class: 'axis', x: pad.left - 8, y: at + 4, 'text-anchor': 'end' }, `${tick}x`))
  }
  rungs.forEach((rung, index) => {
    chart.append(svg('text', { class: 'axis', x: x(index), y: pad.top + plot.h + 18, 'text-anchor': 'middle' }, rung.size))
    if (rung.rows) chart.append(svg('text', { class: 'axis', x: x(index), y: pad.top + plot.h + 32, 'text-anchor': 'middle' }, `${grouped(rung.rows)} rows`))
  })
  for (const engine of engines) {
    const points = []
    rungs.forEach((rung, index) => {
      const ratio = value(rung, engine, 'ratio')
      if (ratio) points.push([x(index), y(ratio), ratio])
    })
    if (!points.length) continue
    const colour = COLOURS[engine] || 'var(--dim)'
    chart.append(svg('polyline', {
      fill: 'none', stroke: colour, 'stroke-width': engine === 'rudb' ? 3 : 2,
      'stroke-linejoin': 'round', points: points.map(([px, py]) => `${px},${py}`).join(' '),
    }))
    for (const [px, py, ratio] of points) {
      chart.append(svg('circle', { cx: px, cy: py, r: engine === 'rudb' ? 4 : 3, fill: colour }))
      chart.append(svg('title', {}, `${engine}: ${ratio.toFixed(2)}x`))
    }
    const [lx, ly] = points[points.length - 1]
    chart.append(svg('text', { class: 'value', x: lx + 8, y: ly + 4, fill: colour }, `${engine} ${points[points.length - 1][2].toFixed(2)}x`))
  }
  at.append(chart)
}

// Round ratios to put a line through, which is a small fixed set rather than a computed one: the
// axis is always a ratio and the numbers a reader wants on it are always these.
function ticks (low, high) {
  return [0.1, 0.25, 0.5, 1, 2, 5, 10, 25, 50, 100].filter((t) => t >= low * 0.9 && t <= high * 1.1)
}

function numbers () {
  const board = ladder()
  const measure = state.measure
  const table = $('numbers')
  table.replaceChildren()
  const head = table.appendChild(document.createElement('thead')).insertRow()
  head.appendChild(document.createElement('th')).textContent = 'size'
  for (const engine of board.engines) {
    head.appendChild(document.createElement('th')).textContent = engine
  }
  head.appendChild(document.createElement('th')).textContent = 'queries compared'
  const body = table.appendChild(document.createElement('tbody'))
  for (const rung of board.rungs) {
    const row = body.insertRow()
    const first = row.insertCell()
    first.textContent = rung.rows ? `${rung.size} (${grouped(rung.rows)} rows)` : rung.size
    const got = board.engines.map((e) => value(rung, e, measure)).filter((v) => v !== null && v > 0)
    const best = got.length ? Math.min(...got) : null
    for (const engine of board.engines) {
      const cell = row.insertCell()
      const column = rung.columns.find((c) => c.engine === engine)
      const at = value(rung, engine, measure)
      if (!column) { cell.textContent = 'not run'; continue }
      if (at === null) { cell.textContent = 'not read'; continue }
      if (at === 0 && (measure === 'load_ms')) { cell.textContent = 'none'; continue }
      cell.textContent = written(at, measure)
      if (measure === 'hot_ms' || measure === 'cold_ms') {
        cell.title = `${column.ratio.toFixed(2)}x duckdb, over ${rung.shared} shared queries`
      }
      if (measure === 'disk' && !column.converted) cell.classList.add('parquet')
      if (at === best) cell.classList.add('best')
      if (engine === 'rudb') cell.classList.add('rudb')
    }
    row.insertCell().textContent = `${rung.shared} of ${Math.max(...rung.columns.map((c) => c.queries))}`
  }
  const foot = document.createElement('caption')
  foot.className = 'note'
  foot.style.captionSide = 'bottom'
  foot.style.textAlign = 'left'
  foot.textContent = MEASURES[measure].note
  table.append(foot)
}

function versions () {
  const table = $('versions')
  table.replaceChildren()
  const head = table.appendChild(document.createElement('thead')).insertRow()
  for (const name of ['engine', 'version', 'machine', 'rungs']) {
    head.appendChild(document.createElement('th')).textContent = name
  }
  const seen = new Map()
  for (const board of state.board.ladders) {
    for (const rung of board.rungs) {
      for (const column of rung.columns) {
        const key = `${column.engine}\u0000${column.version}\u0000${board.machine}`
        seen.set(key, (seen.get(key) || 0) + 1)
      }
    }
  }
  const body = table.appendChild(document.createElement('tbody'))
  for (const [key, count] of [...seen.entries()].sort()) {
    const row = body.insertRow()
    for (const part of key.split('\u0000')) row.insertCell().textContent = part
    row.insertCell().textContent = count
  }
}

function legend () {
  const at = $('legend')
  at.replaceChildren()
  for (const engine of ladder().engines) {
    const button = document.createElement('button')
    button.type = 'button'
    button.setAttribute('aria-pressed', String(!state.off.has(engine)))
    const swatch = document.createElement('span')
    swatch.className = 'swatch'
    swatch.style.background = COLOURS[engine] || 'var(--dim)'
    button.append(swatch, document.createTextNode(engine))
    button.addEventListener('click', () => {
      if (state.off.has(engine)) state.off.delete(engine)
      else state.off.add(engine)
      draw()
    })
    at.append(button)
  }
}

// What the page says about where its numbers came from, which is not optional furniture. Rule seven
// is the reason this sits at the top in a colour nobody scrolls past.
function provenance () {
  const board = ladder()
  const days = [...new Set(board.rungs.map((r) => r.recorded))].sort()
  const commits = [...new Set(board.rungs.map((r) => r.commit))]
  const mixed = days.length > 1
    ? ` These rungs were not all measured on the same day (${days.join(', ')}), so the slope through them is partly whatever changed in between.`
    : ''
  $('provenance').textContent =
    `${board.suite} on ${board.machine}, measured ${days.length > 1 ? 'between ' + days[0] + ' and ' + days[days.length - 1] : 'on ' + days[0]}` +
    ` by rudb-bench at ${commits.join(', ')}. This is a machine the project owns and not a reporting machine, so nothing here is comparable to a published ClickBench or TPC-H number.${mixed}`
  $('generated').textContent = `board rendered from ${state.board.ladders.reduce((n, l) => n + l.rungs.length, 0)} rungs`
}

function draw () {
  $('measure-note').textContent = MEASURES[state.measure].note
  provenance()
  legend()
  bars()
  trend()
  numbers()
  versions()
}

async function start () {
  let board
  try {
    const answer = await fetch('data/board.json', { cache: 'no-cache' })
    if (!answer.ok) throw new Error(`${answer.status} ${answer.statusText}`)
    board = await answer.json()
  } catch (e) {
    $('provenance').textContent = `The board could not be loaded: ${e.message}. The numbers live in data/board.json.`
    return
  }
  state.board = board
  const choose = $('suite')
  for (const ladder of board.ladders) {
    const name = `${ladder.suite} on ${ladder.machine}`
    choose.append(new Option(name, name))
  }
  state.suite = choose.value
  choose.addEventListener('change', () => { state.suite = choose.value; state.off.clear(); draw() })
  $('measure').addEventListener('change', (e) => { state.measure = e.target.value; draw() })
  $('log').addEventListener('change', (e) => { state.log = e.target.checked; draw() })
  draw()
}

start()
