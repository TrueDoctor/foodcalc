// Event overview — one-page (or few-page) management summary of an event:
// frame data with budget status, the planned meals with their planner
// numbers, the shopping tours and the food-prep tasks. Printed for the
// project lead and pinned in the kitchen.
//
// Data contract (arrays of dicts, all strings pre-escaped by the caller):
//   meta:  (span, meal_count, total_cost, budget_display, budget_state)
//   meals: (time, place, name, servings, weight, kj_per_portion, price)
//   tours: (date, store, item_count)
//   preps: (date, recipe, window)

#let flourish() = {
  box(baseline: -0.2em, stack(dir: ltr, spacing: 4pt,
    line(length: 2.5em, stroke: 0.6pt, start: (0%,0.36%)),
    rotate(45deg, square(size: 0.5em, fill: black)),
    line(length: 2.5em, stroke: 0.6pt, start: (0%,0.36%)),
  ))
}

#let section_title(title) = {
  v(0.8em)
  text(size: 1.1em, weight: "bold", style: "italic", title)
  v(0.3em)
}

#let event_overview(event_name, comment, meta, meals, tours, preps) = {
  set page(paper: "a4", margin: (top: 2cm, bottom: 2cm, left: 2cm, right: 2cm))
  set text(font: ("Linux Libertine", "New Computer Modern", "Source Sans Pro"), lang: "de", size: 10pt)
  align(center)[
    #flourish() #h(0.6em)
    #text(size: 1.8em, weight: "bold", style: "italic", event_name)
    #h(0.6em) #flourish()
  ]
  align(center, text(size: 0.9em, style: "italic")[Event-Übersicht])
  if comment != "" [
    #align(center, text(size: 0.9em, fill: gray, comment))
  ]
  v(1em)
  line(length: 100%, stroke: 0.5pt)

  // Frame data with budget status.
  grid(
    columns: (auto, 1fr),
    align: (left, right),
    gutter: 1em,
    text(fill: gray)[Zeitraum], meta.span,
    text(fill: gray)[Mahlzeiten], meta.meal_count,
    text(fill: gray)[Erwartete Kosten], text(weight: "bold", meta.total_cost),
    text(fill: gray)[Budget], meta.budget_display,
  )
  if meta.budget_state == "over" [
    #text(size: 0.85em, fill: red.darken(20%))[
      *Budget überschritten* — bitte nachjustieren (Portionen, Rezepte oder
      Quellen).
    ]
  ] else if meta.budget_state == "tight" [
    #text(size: 0.85em, fill: olive.darken(10%))[
      Das Budget ist zu #meta.budget_share% ausgeschöpft.
    ]
  ]
  v(0.4em)
  line(length: 100%, stroke: 0.25pt + gray.lighten(30%))

  // Meals table.
  section_title("Speiseplan")
  if meals.len() == 0 [
    #text(style: "italic", fill: gray)[Noch keine Mahlzeiten geplant.]
  ] else {
    table(
      columns: (auto, auto, 1fr, auto, auto, auto, auto),
      align: (left, left, left, right, right, right, right),
      stroke: (bottom: 0.4pt + gray.lighten(20%)),
      inset: (y: 4pt, x: 4pt),
      text(size: 0.85em, weight: "bold")[Zeit],
      text(size: 0.85em, weight: "bold")[Ort],
      text(size: 0.85em, weight: "bold")[Gericht],
      text(size: 0.85em, weight: "bold")[Port.],
      text(size: 0.85em, weight: "bold")[Gewicht],
      text(size: 0.85em, weight: "bold")[kJ/Port.],
      text(size: 0.85em, weight: "bold")[Kosten],
      ..meals.map(m => (
        text(size: 0.85em)[#m.time],
        text(size: 0.85em)[#m.place],
        text(size: 0.85em)[#m.name],
        text(size: 0.85em)[#m.servings],
        text(size: 0.85em)[#m.weight],
        text(size: 0.85em)[#m.kj_per_portion],
        text(size: 0.85em)[#m.price],
      )).flatten(),
    )
  }

  // Shopping tours.
  section_title("Einkäufe")
  if tours.len() == 0 [
    #text(style: "italic", fill: gray)[Noch keine Einkaufstouren angelegt.]
  ] else {
    table(
      columns: (auto, 1fr, auto),
      align: (left, left, right),
      stroke: (bottom: 0.4pt + gray.lighten(20%)),
      inset: (y: 4pt, x: 4pt),
      text(size: 0.85em, weight: "bold")[Datum],
      text(size: 0.85em, weight: "bold")[Händler],
      text(size: 0.85em, weight: "bold")[Positionen],
      ..tours.map(t => (
        text(size: 0.85em)[#t.date],
        text(size: 0.85em)[#t.store],
        text(size: 0.85em)[#t.item_count],
      )).flatten(),
    )
  }

  // Food prep tasks.
  section_title("Vorkochen")
  if preps.len() == 0 [
    #text(style: "italic", fill: gray)[Kein Vorkochen geplant.]
  ] else {
    table(
      columns: (auto, 1fr, auto),
      align: (left, left, right),
      stroke: (bottom: 0.4pt + gray.lighten(20%)),
      inset: (y: 4pt, x: 4pt),
      text(size: 0.85em, weight: "bold")[Datum],
      text(size: 0.85em, weight: "bold")[Rezept],
      text(size: 0.85em, weight: "bold")[Verwendbar],
      ..preps.map(p => (
        text(size: 0.85em)[#p.date],
        text(size: 0.85em)[#p.recipe],
        text(size: 0.85em)[#p.window],
      )).flatten(),
    )
  }

  v(1em)
  align(center, text(size: 0.75em, fill: gray, style: "italic")[
    Erstellt mit Foodcalc · Stand: #meta.generated
  ])
}
