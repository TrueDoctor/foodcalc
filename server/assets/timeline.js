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

  // Convert a bar's current pixel geometry back to start/end minutes-since the
  function onPointerDown(e) {
    const bar = e.target.closest("[data-drag]");
    if (!bar) return;
    const root = bar.closest("#timeline-root");
    if (!root) return;

    const resizing = e.target.classList.contains("tl-resize");
    if (resizing && bar.dataset.resizable !== "true") return;

    const lane = laneOf(bar);
    const laneRect = lane.getBoundingClientRect();
    const barRect = bar.getBoundingClientRect();
    const mpp = minutesPerPixel(root, lane);

    drag = {
      bar,
      root,
      lane,
      resizing,
      pointerId: e.pointerId,
      startX: e.clientX,
      // Original geometry in px, relative to the lane, for the live preview.
      origLeftPx: barRect.left - laneRect.left,
      origWidthPx: barRect.width,
      laneWidthPx: laneRect.width,
      // Original start in minutes-since-DAY_START_HOUR, snapped. On a RESIZE the
      // start must never move, so we reuse this verbatim rather than re-measuring
      // the (possibly nudged) left edge at drop.
      origStartMin: toServerMin(root, (barRect.left - laneRect.left) * mpp),
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

  function onPointerMove(e) {
    if (!drag || e.pointerId !== drag.pointerId) return;
    const dx = e.clientX - drag.startX;
    if (Math.abs(dx) > 2) drag.moved = true;
    const pps = pixelsPerSlot(drag);

    if (drag.resizing) {
      // Grow / shrink from the right edge, snapping the width to whole slots so
      // the preview clicks into position (and the drop matches exactly).
      let newWidth = drag.origWidthPx + dx;
      newWidth = Math.round(newWidth / pps) * pps;
      newWidth = Math.max(pps, Math.min(newWidth, drag.laneWidthPx - drag.origLeftPx));
      drag.bar.style.width = (newWidth / drag.laneWidthPx) * 100 + "%";
    } else {
      // Slide left/right, snapping to whole slots, clamped within the lane.
      let newLeft = drag.origLeftPx + dx;
      newLeft = Math.round(newLeft / pps) * pps;
      newLeft = Math.max(0, Math.min(newLeft, drag.laneWidthPx - drag.origWidthPx));
      drag.bar.style.left = (newLeft / drag.laneWidthPx) * 100 + "%";
    }
  }

  function onPointerUp(e) {
    if (!drag || e.pointerId !== drag.pointerId) return;
    const { bar, root, lane, resizing, moved, origStartMin } = drag;
    bar.releasePointerCapture(e.pointerId);
    bar.classList.remove("cursor-grabbing", "opacity-80", "z-10");
    drag = null;

    // A click without movement is not a drag — let dblclick / navigation win.
    if (!moved) return;

    const laneRect = lane.getBoundingClientRect();
    const barRect = bar.getBoundingClientRect();
    const mpp = minutesPerPixel(root, lane);

    const body = new URLSearchParams();
    body.set("kind", bar.dataset.drag);
    body.set("id", bar.dataset.id);
    body.set("day", bar.dataset.day);

    if (resizing && bar.dataset.resizable === "true") {
      // RESIZE: the start is fixed — reuse the captured original, never the live
      // left edge (which can drift by a slot). Only the right edge moves.
      const endMin = toServerMin(root, (barRect.right - laneRect.left) * mpp);
      body.set("start_min", String(origStartMin));
      body.set("end_min", String(endMin));
    } else {
      // MOVE: the left edge is the source of truth; the server preserves
      // duration, so no end is sent.
      const startMin = toServerMin(root, (barRect.left - laneRect.left) * mpp);
      body.set("start_min", String(startMin));
    }

    postUpdate(root, body);
  }

  // POST the change and swap the fresh timeline into #content via htmx, so the
  // server's authoritative (snapped) layout replaces our optimistic drag.
  function postUpdate(root, body) {
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
            window.htmx.swap(content, html, { swapStyle: "innerHTML" });
          } else if (resp.ok && content) {
            content.innerHTML = html;
          }
        });
      })
      .catch(function () {
        // On failure, reload the timeline to discard the optimistic move.
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

  document.addEventListener("pointerdown", onPointerDown);
  document.addEventListener("pointermove", onPointerMove);
  document.addEventListener("pointerup", onPointerUp);
  document.addEventListener("pointercancel", onPointerUp);
  document.addEventListener("dblclick", onDblClick);
})();
