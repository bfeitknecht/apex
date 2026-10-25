/**
 * fx layer — grain, lichen, clock, cursor.
 *
 * Zero dependencies, respects prefers-reduced-motion (everything collapses
 * to a static frame or disables entirely), and silently no-ops when its
 * elements are missing. This file ships as-is to the browser: plain JS,
 * no build step, embedded into the server binary via a text import.
 *
 *   fx-lichen — canvas overlaying the sheet; a DOS-"lichen" colony of box
 *               glyphs that slowly spreads through the sheet's margins.
 *   fx-grain  — fullscreen canvas generating coarse monochrome noise frames.
 *   fx-clock  — ticking 24h clock (Zürich time) in the footer colophon.
 */
(function () {
  "use strict";

  const reduced = (() => {
    try {
      return matchMedia("(prefers-reduced-motion: reduce)").matches;
    } catch {
      return false;
    }
  })();

  function fgColor() {
    const style = getComputedStyle(document.documentElement);
    return style.getPropertyValue("--fx-color").trim() ||
      style.getPropertyValue("--fg").trim();
  }

  function shuffle(list) {
    for (let i = list.length - 1; i > 0; i--) {
      const j = (Math.random() * (i + 1)) | 0;
      const tmp = list[i];
      list[i] = list[j];
      list[j] = tmp;
    }
    return list;
  }

  /* ------------------------------------------------------------------ *
   * Lichen — the MS-DOS virus aesthetic: a colony of box-drawing        *
   * glyphs creeping over the sheet, drawn on a transparent canvas       *
   * above the background but below the content. It grows through the    *
   * margins (rails, bars, gutters) and stops before it swallows the     *
   * text. When it can no longer spread, it stops on its own.            *
   * ------------------------------------------------------------------ */

  const GLYPHS = ["█", "▓", "▒", "░", "▖", "▘", "▝", "▗", "▪", "·"];
  const CELL = 14; // px per glyph cell
  const TICK_MS = 380;
  const GIVE_UP_AFTER = 60; // dormant ticks before the colony is "done"

  function lichen() {
    const canvas = document.getElementById("fx-lichen");
    const ctx = canvas && canvas.getContext("2d");
    if (!ctx) return;

    let cols = 0;
    let rows = 0;
    let colony = null; // Uint8Array — 1 where a glyph has grown
    let frontier = []; // indices of living cells that may still spread
    let protectedRects = [];
    let dormantTicks = 0;
    let timer = null;
    let resizeTimer = null;

    function measure() {
      const root = canvas.parentElement;
      if (!root) return;
      const base = canvas.getBoundingClientRect();
      protectedRects = [];
      for (const node of root.querySelectorAll(".cell, footer.bar")) {
        const r = node.getBoundingClientRect();
        protectedRects.push({
          x0: r.left - base.left,
          y0: r.top - base.top,
          x1: r.right - base.left,
          y1: r.bottom - base.top,
        });
      }
    }

    function blocked(i) {
      const x = (i % cols) * CELL;
      const y = Math.floor(i / cols) * CELL;
      for (const r of protectedRects) {
        if (x + CELL > r.x0 && x < r.x1 && y + CELL > r.y0 && y < r.y1) {
          return true;
        }
      }
      return false;
    }

    function neighbors(i) {
      const x = i % cols;
      const out = [];
      if (x > 0) out.push(i - 1);
      if (x < cols - 1) out.push(i + 1);
      if (i >= cols) out.push(i - cols);
      if (i + cols < cols * rows) out.push(i + cols);
      return out;
    }

    function size() {
      const rect = canvas.getBoundingClientRect();
      if (!rect.width || !rect.height) return;
      const dpr = Math.min(globalThis.devicePixelRatio || 1, 2);
      canvas.width = Math.floor(rect.width * dpr);
      canvas.height = Math.floor(rect.height * dpr);
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
      cols = Math.ceil(rect.width / CELL);
      rows = Math.ceil(rect.height / CELL);
      colony = new Uint8Array(cols * rows);
      frontier = [];
      measure();
      // Spores in the corners and mid-edges — lichen starts where the
      // wall meets the sky.
      const spots = [
        [1, 1],
        [cols - 2, 1],
        [1, rows - 2],
        [cols - 2, rows - 2],
        [(cols / 2) | 0, 0],
        [0, (rows / 2) | 0],
        [cols - 1, (rows / 2) | 0],
      ];
      for (const [x, y] of spots) {
        const i = y * cols + x;
        if (!blocked(i)) {
          colony[i] = 1;
          frontier.push(i);
        }
      }
      drawAll();
    }

    function drawCells(indices) {
      ctx.fillStyle = fgColor();
      ctx.font = CELL - 2 + 'px "Commit Mono", monospace';
      ctx.textBaseline = "top";
      for (const i of indices) {
        const x = (i % cols) * CELL;
        const y = Math.floor(i / cols) * CELL;
        ctx.globalAlpha = 0.14 + Math.random() * 0.22;
        ctx.fillText(GLYPHS[(Math.random() * GLYPHS.length) | 0], x, y - 1);
      }
      ctx.globalAlpha = 1;
    }

    function drawAll() {
      ctx.clearRect(0, 0, canvas.width, canvas.height);
      const grown = [];
      for (let i = 0; i < colony.length; i++) if (colony[i]) grown.push(i);
      drawCells(grown);
    }

    function step() {
      // Sample a few frontier cells; each tries to grow into one free
      // neighbor. Random sampling keeps the growth organic.
      const grew = [];
      const samples = Math.min(frontier.length, 3);
      for (let s = 0; s < samples; s++) {
        const cell = frontier[(Math.random() * frontier.length) | 0];
        for (const target of shuffle(neighbors(cell))) {
          if (colony[target] || blocked(target)) continue;
          colony[target] = 1;
          frontier.push(target);
          grew.push(target);
          break;
        }
      }
      if (grew.length) {
        dormantTicks = 0;
        drawCells(grew);
      } else if (++dormantTicks > GIVE_UP_AFTER) {
        stop(); // the colony has covered its medium — it rests
      }
    }

    function start() {
      if (timer) return;
      timer = setInterval(step, TICK_MS);
    }

    function stop() {
      if (timer) {
        clearInterval(timer);
        timer = null;
      }
    }

    size();
    if (reduced) return; // static frame only

    document.addEventListener(
      "visibilitychange",
      () => document.hidden ? stop() : start(),
    );
    globalThis.addEventListener("resize", () => {
      stop();
      clearTimeout(resizeTimer);
      resizeTimer = setTimeout(() => {
        size();
        start();
      }, 200);
    });
    start();
  }

  /* ------------------------------------------------------------------ *
   * Grain — fullscreen coarse monochrome noise, re-rolled with JS.      *
   * ------------------------------------------------------------------ */

  const GRAIN_W = 160;
  const GRAIN_H = 90;
  const GRAIN_ALPHA = 18; // out of 255 — keep it a whisper
  const GRAIN_MS = 120;

  function grain() {
    const canvas = document.getElementById("fx-grain");
    const ctx = canvas && canvas.getContext("2d");
    if (!ctx) return;

    canvas.width = GRAIN_W;
    canvas.height = GRAIN_H;
    const image = ctx.createImageData(GRAIN_W, GRAIN_H);
    const data = image.data;

    function frame() {
      for (let i = 0; i < data.length; i += 4) {
        const v = (Math.random() * 255) | 0;
        data[i] = v;
        data[i + 1] = v;
        data[i + 2] = v;
        data[i + 3] = GRAIN_ALPHA;
      }
      ctx.putImageData(image, 0, 0);
    }

    frame();
    if (reduced) return;

    setInterval(frame, GRAIN_MS);
  }

  /* ------------------------------------------------------------------ *
   * Clock — ticking 24h time (Zürich) in the footer.                    *
   * ------------------------------------------------------------------ */

  function clock() {
    const el = document.getElementById("fx-clock");
    if (!el) return;

    function paint() {
      el.textContent = new Date().toLocaleTimeString("de-CH", {
        hour: "2-digit",
        minute: "2-digit",
        second: "2-digit",
        hour12: false,
        timeZone: "Europe/Zurich",
      });
    }

    paint();
    if (reduced) return;

    setInterval(paint, 1000);
  }

  /* ------------------------------------------------------------------ *
   * Cursor — blink the block cursor in the headline.                    *
   * ------------------------------------------------------------------ */

  function cursor() {
    const el = document.querySelector(".cursor");
    if (!el || reduced) return;

    setInterval(() => {
      el.style.opacity = el.style.opacity === "0" ? "" : "0";
    }, 530);
  }

  function init() {
    lichen();
    grain();
    clock();
    cursor();
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init);
  } else {
    init();
  }
})();
