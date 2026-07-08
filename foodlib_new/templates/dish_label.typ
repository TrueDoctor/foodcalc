// Dish labels — printable per-meal labels showing dish name + allergen disclosures.
//
// Two layouts:
//  • flat: 6-up A4 grid of small labels (for cutting apart and placing flat).
//  • tent: one label per A5 landscape page, top + bottom halves mirrored so
//          folding along the centerline produces a tent-card that stands up.

#let allergen_label_de = (
  "milch": "Milch",
  "gluten": "Gluten",
  "weizen": "Weizen",
  "eier": "Eier",
  "soja": "Soja",
  "erdnüsse": "Erdnüsse",
  "schalenfrüchte": "Schalenfrüchte",
  "sesamsamen": "Sesam",
  "sellerie": "Sellerie",
  "senf": "Senf",
  "lupine": "Lupine",
  "schwefeldioxid & sulfite": "Sulfite",
  "fisch": "Fisch",
  "schwein": "Schwein",
  "fleisch": "Fleisch",
  "milchprodukt": "Milchprodukt",
  "käse": "Käse",
  "ei-produkt": "Ei-Produkt",
  "krebstiere": "Krebstiere",
  "weichtiere": "Weichtiere",
  "gelatine": "Gelatine",
)

#let pretty(name) = {
  if name in allergen_label_de { allergen_label_de.at(name) }
  else { upper(name.at(0)) + name.slice(1) }
}

// Decorative flourish built from typst primitives — no glyph dependency.
#let flourish() = {
  box(baseline: -0.2em, stack(dir: ltr, spacing: 3pt,
    line(length: 1.5em, stroke: 0.5pt),
    rotate(45deg, square(size: 0.35em, fill: black)),
    line(length: 1.5em, stroke: 0.5pt),
  ))
}

#let dietary_badges(d) = {
  set text(size: 0.85em)
  let parts = ()
  if d.vegan { parts.push(box(stroke: 0.5pt + green, inset: 3pt, radius: 3pt, fill: green.lighten(80%))[*vegan*]) }
  else if d.vegetarian { parts.push(box(stroke: 0.5pt + olive, inset: 3pt, radius: 3pt, fill: olive.lighten(80%))[*vegetarisch*]) }
  if d.lactose_free { parts.push(box(stroke: 0.5pt + blue, inset: 3pt, radius: 3pt, fill: blue.lighten(80%))[*laktosefrei*]) }
  if d.gluten_free { parts.push(box(stroke: 0.5pt + purple, inset: 3pt, radius: 3pt, fill: purple.lighten(80%))[*glutenfrei*]) }
  if parts.len() == 0 { [] } else { stack(dir: ltr, spacing: 4pt, ..parts) }
}

#let label_content(meal, draw_flourish) = {
  set text(font: ("Linux Libertine", "New Computer Modern", "Source Sans Pro"), lang: "de")
  align(center)[
    #if draw_flourish { flourish() } #h(0.4em)
    #text(font: ("Linux Libertine"), size: 1.4em, weight: "bold", style: "italic", meal.name)
    #h(0.4em) #if draw_flourish { flourish() }
  ]
  v(0.2em)
  align(center, text(size: 0.85em, style: "italic")[#meal.place · #meal.time])
  v(0.4em)
  align(center, dietary_badges(meal.dietary))
  v(0.5em)
  if meal.contains.len() > 0 [
    *Enthält:* #meal.contains.map(pretty).join(", ")
    #linebreak()
  ]
  if meal.may_contain.len() > 0 [
    #text(size: 0.85em, style: "italic")[Kann Spuren von #meal.may_contain.map(pretty).join(", ") enthalten.]
  ]
}

#let flat_layout(meals) = {
  set page(paper: "a4", margin: 1cm)
  let cells = meals.map(meal => rect(stroke: 0.5pt + gray, inset: 8pt, radius: 4pt, width: 100%, height: 100%, label_content(meal, false)))
  // 2 columns × 3 rows = 6 labels per A4 page; typst grid automatically pages.
  grid(
    columns: (1fr, 1fr),
    rows: (1fr, 1fr, 1fr),
    gutter: 4mm,
    ..cells,
  )
}

// Folded-tent geometry. Each card is a fixed-size block split top/bottom along
// a horizontal fold line: the top half is rotated 180° so that folding along
// the centerline produces a free-standing tent whose two faces both read
// upright. Dimensions are absolute (mm) so cells never collapse — `1fr`/`100%`
// sizing degenerates when the parent has no fixed height, which is what made
// earlier attempts overlap.
#let card_w = 92mm      // folded card width
#let half_h = 52mm      // height of one face; card block is 2 × this tall

#let tent_face(meal) = align(horizon + center,
  block(width: card_w, height: half_h, inset: 4pt, label_content(meal, true)))

#let tent_card(meal) = {
  set text(size: 8pt)
  block(width: card_w, height: 2 * half_h, {
    // Top face: rotated 180° so it reads upright on the back once folded.
    rotate(180deg, reflow: true, tent_face(meal))
    // Bottom face: upright (front).
    tent_face(meal)
    // Dashed fold guide on the centerline, placed within this card block.
    place(top + center, dy: half_h,
      line(length: card_w, stroke: (paint: gray.lighten(20%), thickness: 0.4pt, dash: "dashed")))
  })
}

#let tent_layout(meals) = {
  // Standard A4 portrait so any office printer handles it natively. Four tent
  // cards per sheet in a 2×2 grid; cut apart on both axes, then fold each along
  // its dashed centerline. The grid has fixed row heights, so >4 meals auto-page.
  set page(paper: "a4", margin: 8mm)
  let cells = meals.map(meal => box(
    // Solid border doubles as the cut line for this card.
    stroke: (paint: gray.lighten(30%), thickness: 0.4pt),
    inset: 2mm,
    tent_card(meal),
  ))
  grid(
    columns: (auto, auto),
    rows: (auto, auto),
    align: center + horizon,
    column-gutter: 6mm,
    row-gutter: 6mm,
    ..cells,
  )
}

#let dish_labels(event_name, layout, meals) = {
  if layout == "tent" { tent_layout(meals) } else { flat_layout(meals) }
}
