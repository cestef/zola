#import "@preview/cetz:0.3.1": *
#import "@preview/cetz-plot:0.1.0": plot

#let colred(x) = text(fill: red, $#x$)
#let colblue(x) = text(fill: blue, $#x$)
#let colgreen(x) = text(fill: green, $#x$)
#let colyellow(x) = text(fill: yellow, $#x$)

#let clipped-paths(points, low, high, fill: false) = {
  let (min-x, max-x) = (
    calc.min(low.at(0), high.at(0)),
    calc.max(low.at(0), high.at(0)),
  )
  let (min-y, max-y) = (
    calc.min(low.at(1), high.at(1)),
    calc.max(low.at(1), high.at(1)),
  )

  let in-rect(pt) = {
    return (pt.at(0) >= min-x and pt.at(0) <= max-x and pt.at(1) >= min-y and pt.at(1) <= max-y)
  }

  let interpolated-end(a, b) = {
    if in-rect(a) and in-rect(b) {
      return b
    }

    let (x1, y1, ..) = a
    let (x2, y2, ..) = b

    if x2 - x1 == 0 {
      return (x2, calc.min(max-y, calc.max(y2, min-y)))
    }

    if y2 - y1 == 0 {
      return (calc.min(max-x, calc.max(x2, min-x)), y2)
    }

    let m = (y2 - y1) / (x2 - x1)
    let n = y2 - m * x2

    let x = x2
    let y = y2

    y = calc.min(max-y, calc.max(y, min-y))
    x = (y - n) / m

    x = calc.min(max-x, calc.max(x, min-x))
    y = m * x + n

    return (x, y)
  }

  // Append path to paths and return paths
  //
  // If path starts or ends with a vector of another part, merge those
  // paths instead appending path as a new path.
  let append-path(paths, path) = {
    if path.len() <= 1 {
      return paths
    }

    let cmp(a, b) = {
      return a.map(calc.round.with(digits: 8)) == b.map(calc.round.with(digits: 8))
    }

    let added = false
    for i in range(0, paths.len()) {
      let p = paths.at(i)
      if cmp(p.first(), path.last()) {
        paths.at(i) = path + p
        added = true
      } else if cmp(p.first(), path.first()) {
        paths.at(i) = path.rev() + p
        added = true
      } else if cmp(p.last(), path.first()) {
        paths.at(i) = p + path
        added = true
      } else if cmp(p.last(), path.last()) {
        paths.at(i) = p + path.rev()
        added = true
      }
      if added { break }
    }

    if not added {
      paths.push(path)
    }
    return paths
  }

  let clamped-pt(pt) = {
    return (
      calc.max(min-x, calc.min(pt.at(0), max-x)),
      calc.max(min-y, calc.min(pt.at(1), max-y)),
    )
  }

  let paths = ()

  let path = ()
  let prev = points.at(0)
  let was-inside = in-rect(prev)
  if was-inside {
    path.push(prev)
  } else if fill {
    path.push(clamped-pt(prev))
  }

  for i in range(1, points.len()) {
    let prev = points.at(i - 1)
    let pt = points.at(i)

    let is-inside = in-rect(pt)

    let (x1, y1, ..) = prev
    let (x2, y2, ..) = pt

    // Ignore lines if both ends are outsides the x-window and on the
    // same side.
    if (x1 < min-x and x2 < min-x) or (x1 > max-x and x2 > max-x) {
      if fill {
        let clamped = clamped-pt(pt)
        if path.last() != clamped {
          path.push(clamped)
        }
      }
      was-inside = false
      continue
    }

    if is-inside {
      if was-inside {
        path.push(pt)
      } else {
        path.push(interpolated-end(pt, prev))
        path.push(pt)
      }
    } else {
      if was-inside {
        path.push(interpolated-end(prev, pt))
      } else {
        let (a, b) = (
          interpolated-end(pt, prev),
          interpolated-end(prev, pt),
        )
        if in-rect(a) and in-rect(b) {
          path.push(a)
          path.push(b)
        } else if fill {
          let clamped = clamped-pt(pt)
          if path.last() != clamped {
            path.push(clamped)
          }
        }
      }

      if path.len() > 0 and not fill {
        paths = append-path(paths, path)
        path = ()
      }
    }

    was-inside = is-inside
  }

  // Append clamped last point if filling
  if fill and not in-rect(points.last()) {
    path.push(clamped-pt(points.last()))
  }

  if path.len() > 1 {
    paths = append-path(paths, path)
  }

  return paths
}

#let compute-stroke-paths(points, x, y) = {
  clipped-paths(points, (x.min + 0.0001, y.min + 0.0001), (x.max - 0.0001, y.max - 0.0001), fill: false)
}

#let sampled-spline-data(points, tension, samples) = {
  assert(
    samples >= 1 and samples <= 100,
    message: "Must at least use 1 sample per curve",
  )

  let curves = bezier.catmull-to-cubic(points, tension)
  let pts = ()
  for c in curves {
    for t in range(0, samples + 1) {
      let t = t / samples
      pts.push(bezier.cubic-point(..c, t))
    }
  }
  return pts
}

#let linearized-data(data, epsilon) = {
  let pts = ()
  // Current slope, set to none if infinite
  let dx = none
  // Previous point, last skipped point
  let prev = none
  let skipped = none
  // Current direction
  let dir = 0

  let len = data.len()
  for i in range(0, len) {
    let pt = data.at(i)
    if prev != none and i < len - 1 {
      let new-dir = pt.at(0) - prev.at(0)
      if new-dir == 0 {
        // Infinite slope
        if dx != none {
          if skipped != none {
            pts.push(skipped)
            skipped = none
          }
          pts.push(pt)
        } else {
          skipped = pt
        }
        dx = none
      } else {
        // Push the previous and the current point
        // if slope or direction changed
        let new-dx = ((pt.at(1) - prev.at(1)) / new-dir)
        if dx == none or calc.abs(new-dx - dx) > epsilon or (new-dir * dir) < 0 {
          if skipped != none {
            pts.push(skipped)
            skipped = none
          }
          pts.push(pt)

          dx = new-dx
          dir = new-dir
        } else {
          skipped = pt
        }
      }
    } else {
      if skipped != none {
        pts.push(skipped)
        skipped = none
      }
      pts.push(pt)
    }

    prev = pt
  }

  return pts
}

// Get the default axis orientation
// depending on the axis name
#let get-default-axis-horizontal(name) = {
  return lower(name).starts-with("x")
}

// Setup axes dictionary
//
// - axis-dict (dictionary): Existing axis dictionary
// - options (dictionary): Named arguments
// - plot-size (tuple): Plot width, height tuple
#let setup-axes(ctx, axis-dict, options, plot-size) = {
  import "/src/axes.typ"

  // Get axis option for name
  let get-axis-option(axis-name, name, default) = {
    let v = options.at(axis-name + "-" + name, default: default)
    if v == auto { default } else { v }
  }

  for (name, axis) in axis-dict {
    if not "ticks" in axis { axis.ticks = () }
    axis.label = get-axis-option(name, "label", $#name$)

    // Configure axis bounds
    axis.min = get-axis-option(name, "min", axis.min)
    axis.max = get-axis-option(name, "max", axis.max)

    assert(
      axis.min not in (none, auto) and axis.max not in (none, auto),
      message: "Axis min and max must be set.",
    )
    if axis.min == axis.max {
      axis.min -= 1
      axis.max += 1
    }

    axis.mode = get-axis-option(name, "mode", "lin")
    axis.base = get-axis-option(name, "base", 10)

    // Configure axis orientation
    axis.horizontal = get-axis-option(name, "horizontal", get-default-axis-horizontal(name))

    // Configure ticks
    axis.ticks.list = get-axis-option(name, "ticks", ())
    axis.ticks.step = get-axis-option(name, "tick-step", axis.ticks.step)
    axis.ticks.minor-step = get-axis-option(name, "minor-tick-step", axis.ticks.minor-step)
    axis.ticks.decimals = get-axis-option(name, "decimals", 2)
    axis.ticks.unit = get-axis-option(name, "unit", [])
    axis.ticks.format = get-axis-option(name, "format", axis.ticks.format)

    // Axis break
    axis.show-break = get-axis-option(name, "break", false)
    axis.inset = get-axis-option(name, "inset", (0, 0))

    // Configure grid
    axis.ticks.grid = get-axis-option(name, "grid", false)

    axis-dict.at(name) = axis
  }

  // Set axis options round two, after setting
  // axis bounds
  for (name, axis) in axis-dict {
    let changed = false

    // Configure axis aspect ratio
    let equal-to = get-axis-option(name, "equal", none)
    if equal-to != none {
      assert.eq(type(equal-to), str, message: "Expected axis name.")
      assert(
        equal-to != name,
        message: "Axis can not be equal to itself.",
      )

      let other = axis-dict.at(equal-to, default: none)
      assert(
        other != none,
        message: "Other axis must exist.",
      )
      assert(
        other.horizontal != axis.horizontal,
        message: "Equal axes must have opposing orientation.",
      )

      let (w, h) = plot-size
      let ratio = if other.horizontal {
        h / w
      } else {
        w / h
      }
      axis.min = other.min * ratio
      axis.max = other.max * ratio

      changed = true
    }

    if changed {
      axis-dict.at(name) = axis
    }
  }

  for (name, axis) in axis-dict {
    axis-dict.at(name) = axes.prepare-axis(ctx, axis, name)
  }

  return axis-dict
}

#let find-contours(data, offset, op: auto, interpolate: true, contour-limit: 50) = {
  assert(
    data != none and type(data) == array,
    message: "Data must be of type array",
  )
  assert(
    type(offset) in (int, float),
    message: "Offset must be numeric",
  )

  let n-rows = data.len()
  let n-cols = data.at(0).len()
  if n-rows < 2 or n-cols < 2 {
    return ()
  }

  assert(
    op == auto or type(op) in (str, function),
    message: "Operator must be of type auto, string or function",
  )
  if op == auto {
    op = if offset < 0 { "<=" } else { ">=" }
  }
  if type(op) == str {
    assert(
      op in ("<", "<=", ">", ">=", "==", "!="),
      message: "Operator must be one of: <, <=, >, >=, != or ==",
    )
  }

  // Return if data is set
  let is-set = if type(op) == function {
    v => op(offset, v)
  } else if op == "==" {
    v => v == offset
  } else if op == "!=" {
    v => v != offset
  } else if op == "<" {
    v => v < offset
  } else if op == "<=" {
    v => v <= offset
  } else if op == ">" {
    v => v > offset
  } else if op == ">=" {
    v => v >= offset
  }

  // Build a binary map that has 0 for unset and 1 for set cells
  let bin-data = data.map(r => r.map(is-set))

  // Get binary data at x, y
  let get-bin(x, y) = {
    if x >= 0 and x < n-cols and y >= 0 and y < n-rows {
      return bin-data.at(y).at(x)
    }
    return false
  }

  // Get data point for x, y coordinate
  let get-data(x, y) = {
    if x >= 0 and x < n-cols and y >= 0 and y < n-rows {
      return float(data.at(y).at(x))
    }
    return none
  }

  // Get case (0 to 15)
  let get-case(tl, tr, bl, br) = {
    int(tl) * 8 + int(tr) * 4 + int(br) * 2 + int(bl)
  }

  let lerp(a, b) = {
    if a == b { return a } else if a == none { return 1 } else if b == none { return 0 }
    return (offset - a) / (b - a)
  }

  // List of all found contours
  let contours = ()

  let segments = ()
  for y in range(-1, n-rows) {
    for x in range(-1, n-cols) {
      let tl = get-bin(x, y)
      let tr = get-bin(x + 1, y)
      let bl = get-bin(x, y + 1)
      let br = get-bin(x + 1, y + 1)

      // Corner data
      //
      // nw-----ne
      // |       |
      // |       |
      // |       |
      // sw-----se
      let nw = get-data(x, y)
      let ne = get-data(x + 1, y)
      let se = get-data(x + 1, y + 1)
      let sw = get-data(x, y + 1)

      // Interpolated edge points
      //
      // +-- a --+
      // |       |
      // d       b
      // |       |
      // +-- c --+
      let a = (x + .5, y)
      let b = (x + 1, y + .5)
      let c = (x + .5, y + 1)
      let d = (x, y + .5)
      if interpolate {
        a = (x + lerp(nw, ne), y)
        b = (x + 1, y + lerp(ne, se))
        c = (x + lerp(sw, se), y + 1)
        d = (x, y + lerp(nw, sw))
      }

      let case = get-case(tl, tr, bl, br)
      if case in (1, 14) {
        segments.push((d, c))
      } else if case in (2, 13) {
        segments.push((b, c))
      } else if case in (3, 12) {
        segments.push((d, b))
      } else if case in (4, 11) {
        segments.push((a, b))
      } else if case == 5 {
        segments.push((d, a))
        segments.push((c, b))
      } else if case in (6, 9) {
        segments.push((c, a))
      } else if case in (7, 8) {
        segments.push((d, a))
      } else if case == 10 {
        segments.push((a, b))
        segments.push((c, d))
      }
    }
  }

  // Join lines to one or more contours
  // This is done by searching for the next line
  // that starts at the current contours head or tail
  // point. If found, push the other coordinate to
  // the contour. If no line could be found, push a
  // new contour.
  let contours = ()
  while segments.len() > 0 {
    if contours.len() == 0 {
      contours.push(segments.remove(0))
    }

    let found = false

    let i = 0
    while i < segments.len() {
      let (a, b) = segments.at(i)
      let (h, t) = (
        contours.last().first(),
        contours.last().last(),
      )
      if a == t {
        contours.last().push(b)
        segments.remove(i)
        found = true
      } else if b == t {
        contours.last().push(a)
        segments.remove(i)
        found = true
      } else if a == h {
        contours.last().insert(0, b)
        segments.remove(i)
        found = true
      } else if b == h {
        contours.last().insert(0, a)
        segments.remove(i)
        found = true
      } else {
        i += 1
      }
    }

    // Insert the next contour
    if not found {
      contours.push(segments.remove(0))
    }

    // Check limit
    assert(
      contours.len() <= contour-limit,
      message: "Countour limit reached! Raise contour-limit if you " + "think this is not an error",
    )
  }

  return contours
}

// Prepare line data
#let _prepare(self, ctx) = {
  let (x, y) = (ctx.x, ctx.y)

  self.contours = self.contours.map(c => {
    c.stroke-paths = compute-stroke-paths(c.line-data, x, y)

    if self.fill {
      c.fill-paths = compute-fill-paths(c.line-data, x, y)
    }
    return c
  })

  return self
}

// Stroke line data
#let _stroke(self, ctx) = {
  for c in self.contours {
    for p in c.stroke-paths {
      draw.line(..p, fill: none, close: p.first() == p.last())
    }
  }
}

#let sample-fn2(fn, x-domain, y-domain, x-samples, y-samples) = {
  assert(
    x-samples >= 2,
    message: "You must at least sample 2 x-values",
  )
  assert(
    y-samples >= 2,
    message: "You must at least sample 2 y-values",
  )
  assert(
    type(x-domain) == array and x-domain.len() == 2,
    message: "X-Domain must be a tuple",
  )
  assert(
    type(y-domain) == array and y-domain.len() == 2,
    message: "Y-Domain must be a tuple",
  )

  let (x-min, x-max) = x-domain
  let (y-min, y-max) = y-domain
  let y-pts = range(0, y-samples)
  let x-pts = range(0, x-samples)

  return y-pts.map(y => {
    let y = y / (y-samples - 1) * (y-max - y-min) + y-min
    return x-pts.map(x => {
      let x = x / (x-samples - 1) * (x-max - x-min) + x-min
      return float((fn)(x, y))
    })
  })
}

#let add-contour(
  data,
  label: none,
  z: (1,),
  x-domain: (0, 1),
  y-domain: (0, 1),
  x-samples: 25,
  y-samples: 25,
  interpolate: true,
  op: auto,
  axes: ("x", "y"),
  style: (:),
  fill: false,
  limit: 50,
) = {
  // Sample a x/y function
  if type(data) == function {
    data = sample-fn2(
      data,
      x-domain,
      y-domain,
      x-samples,
      y-samples,
    )
  }

  // Find matrix dimensions
  assert(type(data) == array)
  let (x-min, x-max) = x-domain
  let dx = (x-max - x-min) / (data.at(0).len() - 1)
  let (y-min, y-max) = y-domain
  let dy = (y-max - y-min) / (data.len() - 1)

  let contours = ()
  let z = if type(z) == array { z } else { (z,) }
  for z in z {
    for contour in find-contours(data, z, op: op, interpolate: interpolate, contour-limit: limit) {
      let line-data = contour.map(pt => {
        (
          pt.at(0) * dx + x-min,
          pt.at(1) * dy + y-min,
        )
      })

      contours.push((
        z: z,
        line-data: line-data,
      ))
    }
  }

  return (
    (
      type: "contour",
      label: label,
      contours: contours,
      axes: axes,
      x-domain: x-domain,
      y-domain: y-domain,
      style: style,
      fill: fill,
      mark: none,
      mark-style: none,
      plot-prepare: _prepare,
      plot-stroke: _stroke,
      plot-legend-preview: self => {
        if not self.fill { self.style.fill = none }
        draw.rect((0, 0), (1, 1), ..self.style)
      },
    ),
  )
}
