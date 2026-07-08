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
  // Non-EU-14 avoidances (rendered in the separate "Hinweise" section).
  "zwiebel": "Zwiebeln",
  "knoblauch": "Knoblauch",
  "pilz": "Pilze",
  "apfel": "Apfel",
  "karotte": "Karotte",
  "kichererbse": "Kichererbsen",
  "zitrus": "Zitrusfrüchte",
  "guar": "Guarkernmehl",
)

#let pretty(name) = {
  if name in allergen_label_de { allergen_label_de.at(name) }
  else { upper(name.at(0)) + name.slice(1) }
}

// A label carrying a may-contain trace on its axis gets a trailing asterisk;
// the footnote below explains it. `star(cond)` yields "*" or "".
#let dietary_badges(d) = {
  set text(size: 0.85em)
  let star(cond) = if cond [\*] else []
  let parts = ()
  if d.vegan { parts.push(box(stroke: 0.5pt + green, inset: 3pt, radius: 3pt, fill: green.lighten(80%))[*vegan#star(d.vegan_trace)*]) }
  else if d.vegetarian { parts.push(box(stroke: 0.5pt + olive, inset: 3pt, radius: 3pt, fill: olive.lighten(80%))[*vegetarisch#star(d.vegetarian_trace)*]) }
  if d.lactose_free { parts.push(box(stroke: 0.5pt + blue, inset: 3pt, radius: 3pt, fill: blue.lighten(80%))[*laktosefrei#star(d.lactose_free_trace)*]) }
  if d.gluten_free { parts.push(box(stroke: 0.5pt + purple, inset: 3pt, radius: 3pt, fill: purple.lighten(80%))[*glutenfrei#star(d.gluten_free_trace)*]) }
  let has_trace = d.vegan_trace or d.vegetarian_trace or d.lactose_free_trace or d.gluten_free_trace
  if parts.len() == 0 { [] } else {
    stack(dir: ltr, spacing: 4pt, ..parts)
    if has_trace [
      #v(0.2em)
      #text(size: 0.7em, style: "italic")[\* vegan / vegetarisch / laktosefrei / glutenfrei ohne Berücksichtigung von Spuren.]
    ]
  }
}

// Decorative flourish built from typst primitives (no glyph dependency).
// A short horizontal line + small diamond + line, evoking an art-deco rule.
#let flourish() = {
  box(baseline: -0.2em, stack(dir: ltr, spacing: 4pt,
    line(length: 2.5em, stroke: 0.6pt, start: (0%,0.36%)),
    rotate(45deg, square(size: 0.5em, fill: black)),
    line(length: 2.5em, stroke: 0.6pt, start: (0%,0.36%)),
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
        #text(size: 0.9em, style: "italic")[Kann Spuren von #meal.may_contain.map(pretty).join(", ") enthalten.] \
      ]
      #if meal.at("hinweise", default: ()).len() > 0 [
        #text(size: 0.9em)[*Hinweise (keine EU-Allergene):* #meal.hinweise.map(pretty).join(", ")]
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
