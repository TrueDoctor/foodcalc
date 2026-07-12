// Timeline drag / resize / double-click interactions.
//
// The timeline is server-rendered (Maud + HTMX). This vanilla module adds the
// client-only pointer interactions on top of the static markup:
//
//   * drag a bar horizontally    -> change its start time (duration preserved)
//   * drag the right-edge handle -> change its end time (meals / prep only)
//   * double-click a meal bar    -> open the meal edit form
//
// Positions are converted from pixels to minutes using the day-track width and
// the window size the server exposes as data attributes on #timeline-root:
//   data-cols  = number of hour columns in the shared window
//   data-snap  = snap granularity in minutes (e.g. 30)
// A bar carries data-drag (meal|tour|prep), data-id, data-day (logical date),
// data-resizable and (meals) data-edit. On drop we POST to
//   /events/edit/timeline/{event}/move
// and swap the returned timeline into #content via htmx.
//
// Everything is event-delegated from the document so it keeps working after
// HTMX swaps replace the timeline; there is nothing to re-bind on settle.

(function () {
  "use strict";

  // Active drag gesture, or null when idle.
  let drag = null;

  // The lane track element that owns a bar (its positioning context).
  function laneOf(bar) {
    return bar.parentElement;
  }

  // Minutes-per-pixel for the given lane, derived from the shared window width.
  function minutesPerPixel(root, lane) {
    const cols = parseFloat(root.dataset.cols) || 24;
    const totalMinutes = cols * 60;
    return totalMinutes / lane.getBoundingClientRect().width;
  }

  // Snap a minute value to the configured granularity, clamped to [0, 24h].
  function snap(root, minutes) {
    const step = parseFloat(root.dataset.snap) || 30;
    const snapped = Math.round(minutes / step) * step;
    return Math.max(0, Math.min(24 * 60, snapped));
  }

  // Convert a bar's position measured relative to the LANE's left edge (which is
  // the window's left = data-start-hour) into minutes-since-DAY_START_HOUR — the
  // coordinate the /move endpoint expects — then snap. The window does not start
  // at DAY_START_HOUR, so we add the window's start-hour offset.
  function toServerMin(root, windowRelMinutes) {
    const startHour = parseFloat(root.dataset.startHour) || 0;
    return snap(root, windowRelMinutes + startHour * 60);
  }

  // Which day-row the pointer is over, as its ISO date string (data-tl-day), or
  // null if the pointer is outside every row. Used to route a cross-day drop:
  // the vertical position picks the DAY, the horizontal position picks the time.
  // We test each row's bounding box directly rather than elementFromPoint so a
  // bar being dragged (which sits under the cursor) doesn't shadow the row.
  function dayAtPoint(root, clientX, clientY) {
    const rows = root.querySelectorAll("[data-tl-day]");
    for (const row of rows) {
      const r = row.getBoundingClientRect();
      if (
        clientY >= r.top &&
        clientY <= r.bottom &&
        clientX >= r.left &&
        clientX <= r.right
      ) {
        return row.dataset.tlDay;
      }
    }
    return null;
  }

  // The inner track element (the positioning context that spans the axis + all
  // lanes) for a given day-row, so a cross-day drop can measure the drop-X
  // against the destination day's own track width, not the source lane's.
  function trackOfDay(root, isoDay) {
    const row = root.querySelector('[data-tl-day="' + isoDay + '"]');
    // The track is the `.relative.flex.flex-col` wrapper inside the row.
    return row ? row.querySelector(".relative.flex.flex-col") : null;
  }

  // Convert a bar's current pixel geometry back to start/end minutes-since the
  function onPointerDown(e) {
    const bar = e.target.closest("[data-drag]");
    if (!bar) return;
    const root = bar.closest("#timeline-root");
    if (!root) return;

    // Which gesture: grabbing an edge handle resizes; grabbing the body moves.
    // `data-edge` (left|right) says which end a resize drags.
    const handle = e.target.closest(".tl-resize");
    const resizing = !!handle;
    if (resizing && bar.dataset.resizable !== "true") return;
    const edge = handle ? handle.dataset.edge : null;

    const lane = laneOf(bar);
    const laneRect = lane.getBoundingClientRect();
    const barRect = bar.getBoundingClientRect();
    const mpp = minutesPerPixel(root, lane);

    drag = {
      bar,
      root,
      lane,
      resizing,
      edge,
      pointerId: e.pointerId,
      startX: e.clientX,
      // Original geometry in px, relative to the lane, for the live preview.
      origLeftPx: barRect.left - laneRect.left,
      origWidthPx: barRect.width,
      laneWidthPx: laneRect.width,
      // Original start/end in minutes-since-DAY_START_HOUR, snapped. On a resize
      // the fixed edge must never move, so we reuse these verbatim at drop rather
      // than re-measuring the (possibly nudged) live edge.
      origStartMin: toServerMin(root, (barRect.left - laneRect.left) * mpp),
      origEndMin: toServerMin(root, (barRect.right - laneRect.left) * mpp),
      mpp,
      moved: false,
    };

    bar.setPointerCapture(e.pointerId);
    bar.classList.add("cursor-grabbing", "opacity-80", "z-10");
    e.preventDefault();
  }

  // Pixels-per-slot for a lane, so the preview can snap to whole slots visually.
  function pixelsPerSlot(drag) {
    const step = parseFloat(drag.root.dataset.snap) || 30;
    return step / drag.mpp;
  }

  // Highlight the day-row the pointer is currently over during a cross-day move,
  // so the drop target is obvious. Clears the highlight from all other rows.
  function highlightDropDay(root, isoDay) {
    root.querySelectorAll("[data-tl-day]").forEach(function (row) {
      row.classList.toggle("tl-drop-target", row.dataset.tlDay === isoDay);
    });
  }

  function clearDropHighlight(root) {
    root.querySelectorAll(".tl-drop-target").forEach(function (row) {
      row.classList.remove("tl-drop-target");
    });
  }

  // Lift the bar out of its lane into `position: fixed` so it can float across
  // day-rows during a move. Captured once, on the first movement of a move drag.
  // We stash the original inline styles to restore them if the drop fails.
  function liftBar(drag, e) {
    const barRect = drag.bar.getBoundingClientRect();
    drag.floating = true;
    // Pointer offset within the bar, so the bar tracks the cursor 1:1.
    drag.grabOffsetX = e.clientX - barRect.left;
    drag.grabOffsetY = e.clientY - barRect.top;
    drag.savedStyle = drag.bar.getAttribute("style") || "";
    drag.bar.style.position = "fixed";
    drag.bar.style.margin = "0";
    drag.bar.style.width = barRect.width + "px";
    drag.bar.style.height = barRect.height + "px";
    drag.bar.style.left = barRect.left + "px";
    drag.bar.style.top = barRect.top + "px";
    drag.bar.style.pointerEvents = "none"; // don't shadow dayAtPoint hit-testing
    drag.bar.classList.add("shadow-lg");
  }

  // Restore a lifted bar's original inline styles (used when a move is abandoned
  // or the POST fails; a successful drop re-renders the whole timeline anyway).
  function dropBar(drag) {
    if (!drag.floating) return;
    drag.bar.setAttribute("style", drag.savedStyle);
    drag.bar.classList.remove("shadow-lg");
  }

  function onPointerMove(e) {
    if (!drag || e.pointerId !== drag.pointerId) return;
    const dx = e.clientX - drag.startX;
    if (Math.abs(dx) > 2) drag.moved = true;
    const pps = pixelsPerSlot(drag);

    if (drag.resizing && drag.edge === "left") {
      // LEFT-edge resize: the right edge stays put; the left edge (start time)
      // moves. Grow when dragging left (dx<0), shrink when dragging right.
      // Snap the left edge to whole slots and keep at least one slot of width.
      const rightPx = drag.origLeftPx + drag.origWidthPx;
      let newLeft = drag.origLeftPx + dx;
      newLeft = Math.round(newLeft / pps) * pps;
      newLeft = Math.max(0, Math.min(newLeft, rightPx - pps));
      drag.bar.style.left = (newLeft / drag.laneWidthPx) * 100 + "%";
      drag.bar.style.width = ((rightPx - newLeft) / drag.laneWidthPx) * 100 + "%";
    } else if (drag.resizing) {
      // RIGHT-edge resize: the left edge stays; grow / shrink from the right,
      // snapping the width to whole slots so the preview clicks into position.
      let newWidth = drag.origWidthPx + dx;
      newWidth = Math.round(newWidth / pps) * pps;
      newWidth = Math.max(pps, Math.min(newWidth, drag.laneWidthPx - drag.origLeftPx));
      drag.bar.style.width = (newWidth / drag.laneWidthPx) * 100 + "%";
    } else {
      // MOVE: float the bar with `position: fixed` so it can cross day-rows.
      // Lift it out of its lane on the first real movement.
      if (!drag.floating) liftBar(drag, e);

      // Which day-row is under the pointer right now (fall back to the origin
      // day when the pointer strays off all rows). Highlight it.
      const overDay = dayAtPoint(drag.root, e.clientX, e.clientY) || drag.bar.dataset.day;
      highlightDropDay(drag.root, overDay);

      // Vertical: follow the pointer directly (the bar rides with the cursor).
      const topPx = e.clientY - drag.grabOffsetY;

      // Horizontal: snap the bar's left edge to the target day's slot grid so the
      // preview lines up with the columns it will land on. Measure against that
      // day's own track; clamp so the bar stays within the track width.
      const destTrack = trackOfDay(drag.root, overDay);
      const trackRect = (destTrack || drag.lane).getBoundingClientRect();
      let relLeft = e.clientX - drag.grabOffsetX - trackRect.left;
      relLeft = Math.round(relLeft / pps) * pps;
      relLeft = Math.max(0, Math.min(relLeft, trackRect.width - drag.origWidthPx));

      drag.bar.style.left = trackRect.left + relLeft + "px";
      drag.bar.style.top = topPx + "px";
    }
  }

  function onPointerUp(e) {
    if (!drag || e.pointerId !== drag.pointerId) return;
    const gesture = drag;
    const { bar, root, lane, resizing, edge, moved, origStartMin, origEndMin } = drag;
    bar.releasePointerCapture(e.pointerId);
    bar.classList.remove("cursor-grabbing", "opacity-80", "z-10");
    drag = null;
    clearDropHighlight(root);

    // A click without movement is not a drag — let dblclick / navigation win.
    if (!moved) {
      dropBar(gesture);
      return;
    }

    const laneRect = lane.getBoundingClientRect();
    // Measure BEFORE restoring the bar's styles: while floating it's fixed-
    // positioned, but its viewport rect is still what the drop math wants.
    const barRect = bar.getBoundingClientRect();
    const mpp = minutesPerPixel(root, lane);

    const body = new URLSearchParams();
    body.set("kind", bar.dataset.drag);
    body.set("id", bar.dataset.id);

    if (resizing && bar.dataset.resizable === "true") {
      // RESIZE stays within the bar's own day; only one edge moves. The OTHER
      // edge is fixed — reuse its captured original rather than the live edge
      // (which can drift by a slot). Both start and end are always sent so the
      // server sets the exact span (it guards against an inverted bar).
      body.set("day", bar.dataset.day);
      if (edge === "left") {
        // Left edge moved -> new start, original end.
        const startMin = toServerMin(root, (barRect.left - laneRect.left) * mpp);
        body.set("start_min", String(startMin));
        body.set("end_min", String(origEndMin));
      } else {
        // Right edge moved -> original start, new end.
        const endMin = toServerMin(root, (barRect.right - laneRect.left) * mpp);
        body.set("start_min", String(origStartMin));
        body.set("end_min", String(endMin));
      }
    } else {
      // MOVE: horizontal position picks the clock time, vertical position picks
      // the DAY. If the pointer ended over a different day-row than the bar
      // started on, that's a cross-day move — send the destination day and
      // measure the drop-X against that day's own track (its width/geometry
      // matches the source, but resolving it explicitly keeps this correct even
      // if a future layout gives days different widths).
      const dropDay = dayAtPoint(root, e.clientX, e.clientY) || bar.dataset.day;
      const destTrack = trackOfDay(root, dropDay);
      const measureRect =
        dropDay !== bar.dataset.day && destTrack
          ? destTrack.getBoundingClientRect()
          : laneRect;
      const startMin = toServerMin(root, (barRect.left - measureRect.left) * mpp);
      body.set("day", dropDay);
      body.set("start_min", String(startMin));
    }

    postUpdate(root, body, gesture);
  }

  // POST the change and swap the fresh timeline into #content via htmx, so the
  // server's authoritative (snapped) layout replaces our optimistic drag.
  function postUpdate(root, body, gesture) {
    const eventId = root.dataset.event;
    const url = "/events/edit/timeline/" + eventId + "/move";
    fetch(url, {
      method: "POST",
      headers: {
        "Content-Type": "application/x-www-form-urlencoded",
        // Signal htmx_middleware to return just the fragment (not the full
        // page shell), so we can swap it straight into #content.
        "HX-Request": "true",
      },
      body: body.toString(),
    })
      .then(function (resp) {
        return resp.text().then(function (html) {
          const content = document.getElementById("content");
          if (resp.ok && content && window.htmx) {
            // The swap replaces the whole timeline (incl. the floating bar), so
            // no manual style restore is needed on success.
            window.htmx.swap(content, html, { swapStyle: "innerHTML" });
          } else if (resp.ok && content) {
            content.innerHTML = html;
          } else {
            // Non-OK: put the lifted bar back where it started.
            if (gesture) dropBar(gesture);
          }
        });
      })
      .catch(function () {
        // On failure, restore the optimistic bar, then reload the timeline.
        if (gesture) dropBar(gesture);
        if (window.htmx) {
          window.htmx.ajax("GET", "/events/edit/timeline/" + eventId, "#content");
        }
      });
  }

  // Double-click a meal bar to open its edit form (htmx navigation).
  function onDblClick(e) {
    const bar = e.target.closest("[data-edit]");
    if (!bar) return;
    const url = bar.dataset.edit;
    if (!url) return;
    if (window.htmx) {
      window.htmx.ajax("GET", url, "#content");
      if (window.history) window.history.pushState({}, "", url);
    } else {
      window.location.href = url;
    }
  }

  // Cancelled gesture (e.g. system interruption): abandon without POSTing —
  // restore a lifted bar and clear any highlight.
  function onPointerCancel(e) {
    if (!drag || e.pointerId !== drag.pointerId) return;
    const gesture = drag;
    drag.bar.classList.remove("cursor-grabbing", "opacity-80", "z-10");
    clearDropHighlight(drag.root);
    drag = null;
    dropBar(gesture);
  }

  document.addEventListener("pointerdown", onPointerDown);
  document.addEventListener("pointermove", onPointerMove);
  document.addEventListener("pointerup", onPointerUp);
  document.addEventListener("pointercancel", onPointerCancel);
  document.addEventListener("dblclick", onDblClick);

  // ── Live "now" cursor ──────────────────────────────────────────────────────
  //
  // The server renders the cursor line (a red playhead) on today's row from
  // OffsetDateTime::now_local(). We keep it live between server renders by
  // recomputing its position from the client clock every ~15s. Coordinates must
  // mirror the server: 0 == DAY_START_HOUR (04:00), wrapping into [0, 24).
  const DAY_START_HOUR = 4;
  const SLOTS_PER_DAY = 24;

  // Fractional hours-since-DAY_START_HOUR — the JS twin of the server `hour_pos`.
  function hourPos(hours, minutes) {
    let shifted = hours + minutes / 60 - DAY_START_HOUR;
    if (shifted < 0) shifted += SLOTS_PER_DAY;
    return shifted;
  }

  // Current wall clock as { hours, minutes }. The app treats the operator's
  // local wall clock as the canonical time (stored as UTC components; the server
  // cursor comes from now_local()), and for a single-site catering tool the
  // viewer is on-site — so the browser's local components are the right source.
  function nowWallClock() {
    const d = new Date();
    return { hours: d.getHours(), minutes: d.getMinutes() };
  }

  // Reposition every rendered cursor to the live "now" and refresh its time chip,
  // so the playhead glides between server renders. Hides it when "now" falls
  // outside the visible window (the server renders nothing in that case).
  function tickCursor() {
    const root = document.getElementById("timeline-root");
    if (!root) return;
    const cols = parseFloat(root.dataset.cols) || 24;
    const startHour = parseFloat(root.dataset.startHour) || 0;
    const wc = nowWallClock();
    const pos = hourPos(wc.hours, wc.minutes);
    const inWindow = pos >= startHour && pos < startHour + cols;
    const rel = Math.min(Math.max(pos - startHour, 0), cols);
    const leftPct = (rel / cols) * 100;
    const label =
      String(wc.hours).padStart(2, "0") + ":" + String(wc.minutes).padStart(2, "0");
    root.querySelectorAll(".tl-cursor").forEach(function (cursor) {
      cursor.style.display = inWindow ? "" : "none";
      cursor.style.left = leftPct.toFixed(3) + "%";
      const chip = cursor.querySelector(".tl-cursor-label");
      if (chip) chip.textContent = label;
    });
  }

  // ── Focus today's row on load ───────────────────────────────────────────────
  //
  // Scroll the row whose data-tl-day == data-today into view so a freshly opened
  // timeline lands on the current day rather than the top. We focus only ONCE per
  // page-load, not on the 10-min background refresh — otherwise every poll would
  // yank the user's scroll back to today after they'd scrolled elsewhere. The
  // guard is keyed on the event id so navigating to a different event re-focuses.
  let focusedEvent = null;
  function focusToday() {
    const root = document.getElementById("timeline-root");
    if (!root) return;
    if (focusedEvent === root.dataset.event) return;
    focusedEvent = root.dataset.event;
    const today = root.dataset.today;
    if (!today) return;
    const row = root.querySelector('[data-tl-day="' + today + '"]');
    if (row) row.scrollIntoView({ block: "center", behavior: "smooth" });
  }

  // ── Drag-aware background refresh ────────────────────────────────────────────
  //
  // The server hangs an `every 600s` htmx poll on #timeline-root. Veto it while a
  // drag is in flight so a refresh never rips the DOM out mid-gesture (the
  // dropped move POSTs its own fresh render anyway).
  document.body.addEventListener("htmx:beforeRequest", function (e) {
    const el = e.detail && e.detail.elt;
    if (drag && el && el.id === "timeline-root") {
      e.preventDefault();
    }
  });

  // Re-focus today and restart the cursor after every htmx swap (initial load,
  // background refresh, and post-move re-render all funnel through settle).
  document.body.addEventListener("htmx:afterSettle", function () {
    focusToday();
    tickCursor();
  });

  // Kick things off on first script run, then keep the cursor live.
  focusToday();
  tickCursor();
  setInterval(tickCursor, 15000);
})();
