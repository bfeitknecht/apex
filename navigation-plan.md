# Apex — structure and navigation plan

## Core idea

A monochrome portfolio with a clear, shared structure across the browser and SSH. Two distinct graphical studies share the same place in the navigation: **Unicode in the browser**, and **ASCII in a native or SSH terminal**. Their exact visual concepts remain open. Lichen is a separate screensaver, not the portfolio background. No logo treatment.

## Navigation

1. **About** — introduction and interests.
2. **Projects** — project list, descriptions, and links.
3. **Contact** — contact links and a link to the CV. Experience and Education are not portfolio sections.
4. **Study** — opens the study for the current environment:
   - Browser: Unicode study. Enlarged glyphs and metadata are one possible direction, not a settled design.
   - Native / SSH: a distinct ASCII study, designed for terminal character cells. Not a degraded copy of the browser study.
5. **Lichen** — a separate fullscreen monochrome screensaver pane.

**Help** is reachable from any view and returns to the view that opened it. Returning from Study or Lichen restores the prior portfolio section, scroll position and selection.

## Interaction proposal

| Interaction | Portfolio | Study / Lichen | Help |
|---|---|---|---|
| Navigation / number keys | Choose a destination | Switch destination, or use Back | Close help first |
| Up / Down / scroll | Navigate or scroll **within** the section; never change sections automatically | Study-specific controls to be designed; Lichen remains a screensaver | Scroll help when needed |
| Enter | Activate selected link | Study-specific action, if any | — |
| Back / Escape | Deselect a link if needed | Return to the previous portfolio section | Return to the previous view |
| H / ? | Open help | Open help | Close help |
| Mouse / touch | Select tabs, scroll content, activate links | Visible Back control; additional controls as needed | Close control |
| Space / R | — | Lichen: pause / restart. Study: undecided | — |
| Q | Quit native / SSH terminal | Quit native / SSH terminal | Quit native / SSH terminal |

Browser links open normally; the terminal shows copyable URLs. Keyboard-only access and reduced-motion options should be supported. The browser must remain usable on touch devices.

## Implementation boundaries

- **Shared Rust / Ratatui core:** navigation, portfolio content, help, section state, link selection, and a frontend-independent mode interface.
- **Browser frontend:** Ratzilla portfolio, Unicode study renderer, square-cell Lichen canvas, pointer/touch and keyboard input.
- **Native / SSH frontend:** Ratatui portfolio, ASCII study renderer, terminal Lichen rendering, keyboard and terminal mouse input.
- **Inactive Crash study:** retain in the codebase, but do not include it in navigation.
- Keep both graphical studies separate from Lichen and from readable portfolio content. Exact appearance, timing, controls and algorithms are deliberately not decided yet.
- The browser can render Lichen with square pixels. Terminal half-block proportions depend on the user's font and terminal cell geometry; exact browser/native visual parity cannot be assumed.

## Suggested sequence

1. Agree on navigation placement, study entry and Lichen behavior.
2. Implement the simplified portfolio and shared navigation; remove the logo and old Lichen/portfolio transition.
3. Add empty environment-specific Study views with Back and Help; keep the visual concept undecided.
4. Restore Lichen as its own pane, with pause, restart and return controls.
5. Explore visual directions for the Unicode and ASCII studies separately before implementing them.

## Decisions to settle

- **Navigation layout:** top bar, left-side index, or another arrangement?
- **Study entry:** one environment-aware “Study” item (proposed), or explicitly labeled “Unicode” in browser and “ASCII” in terminal?
- **Lichen entry:** manual only, or also after inactivity? On return, resume growth or start a fresh seed?
- **Study presence:** should its entry be visually prominent on the portfolio landing view, or simply one navigation item?
