// Event-wide allergen overview — one document listing every meal at an event
// with its allergens and dietary flags. Used by the kitchen / event team to
// see the full picture at a glance, and by guests with allergies.

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

#let dietary_badges(d) = {
  set text(size: 0.85em)
  let parts = ()
  if d.vegan { parts.push(box(stroke: 0.5pt + green, inset: 3pt, radius: 3pt, fill: green.lighten(80%))[*vegan*]) }
  else if d.vegetarian { parts.push(box(stroke: 0.5pt + olive, inset: 3pt, radius: 3pt, fill: olive.lighten(80%))[*vegetarisch*]) }
  if d.lactose_free { parts.push(box(stroke: 0.5pt + blue, inset: 3pt, radius: 3pt, fill: blue.lighten(80%))[*laktosefrei*]) }
  if d.gluten_free { parts.push(box(stroke: 0.5pt + purple, inset: 3pt, radius: 3pt, fill: purple.lighten(80%))[*glutenfrei*]) }
  if parts.len() == 0 { [] } else { stack(dir: ltr, spacing: 4pt, ..parts) }
}

// Decorative flourish built from typst primitives (no glyph dependency).
// A short horizontal line + small diamond + line, evoking an art-deco rule.
#let flourish() = {
  box(baseline: 0.3em, stack(dir: ltr, spacing: 4pt,
    line(length: 2.5em, stroke: 0.6pt),
    rotate(45deg, square(size: 0.4em, fill: black)),
    line(length: 2.5em, stroke: 0.6pt),
  ))
}

#let event_allergen_overview(event_name, meals) = {
  set page(paper: "a4", margin: (top: 2cm, bottom: 2cm, left: 2cm, right: 2cm))
  set text(font: ("Linux Libertine", "New Computer Modern", "Source Sans Pro"), lang: "de", size: 10pt)
  align(center)[
    #flourish() #h(0.6em)
    #text(size: 1.8em, weight: "bold", style: "italic", event_name)
    #h(0.6em) #flourish()
  ]
  align(center, text(size: 0.9em, style: "italic")[Allergen-Übersicht])
  v(1em)
  line(length: 100%, stroke: 0.5pt)
  v(0.5em)

  for meal in meals {
    block(breakable: false)[
      #grid(
        columns: (1fr, auto),
        align: (left, right),
        text(size: 1.15em, weight: "bold", style: "italic", meal.name),
        text(size: 0.9em, style: "italic")[#meal.time · #meal.place · #meal.servings Pers.],
      )
      #v(0.2em)
      #dietary_badges(meal.dietary)
      #v(0.3em)
      #if meal.contains.len() > 0 [
        *Enthält:* #meal.contains.map(pretty).join(", ") \
      ] else [
        #text(style: "italic")[Keine deklarationspflichtigen Allergene.] \
      ]
      #if meal.may_contain.len() > 0 [
        #text(size: 0.9em, style: "italic")[Kann Spuren von #meal.may_contain.map(pretty).join(", ") enthalten.]
      ]
    ]
    v(0.6em)
    line(length: 100%, stroke: 0.25pt + gray.lighten(30%))
    v(0.4em)
  }

  v(1em)
  align(center, text(size: 0.75em, fill: gray, style: "italic")[
    Erstellt mit Foodcalc · Angaben gem. LMIV Anhang II
  ])
}
