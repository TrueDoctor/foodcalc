// The project function defines how your document looks.
// It takes your content and some metadata and formats it.
// Go ahead and customize it to your liking!
#let project(title: "", authors: (), date: none, body) = {
  // Set the document's basic properties.
  // set document(author: authors, title: title)
  set page(numbering: "1", number-align: center)
  set text(font: "Source Sans Pro", lang: "de")

  // Title row.
  align(center)[
    #block(text(weight: 700, 1.75em, title))
    #v(1em, weak: true)
    #date
  ]

  // Author information.
  pad(
    top: 0.5em,
    bottom: 0.5em,
    x: 2em,
    grid(
      columns: (1fr,) * calc.min(3, authors.len()),
      gutter: 1em,
      ..authors.map(author => align(center, strong(author))),
    ),
  )

  // Main body.
  set par(justify: true)

  body
}

#let significant_digits(x) = {
  if x == 0 {
    0
  } else {
    let sign = if x < 0 { -1 } else { 1 }
    let ax = calc.abs(x)
    let first = calc.ceil(calc.log(ax))
    let y = ax / calc.pow(10, first)
    let rounded = calc.round(digits: 4, y)
    sign * calc.round(digits: 3, rounded * calc.pow(10, first))
  }
}

#let format_weight(w) = if w < 1 [
   #significant_digits(w * 1000) g
] else [
  #significant_digits(w) kg
]

#let format_duration(d) = if d < 60 [
  #{calc.round(d, digits: 2)} min
] else [
  #{calc.floor(d/60)} h #{calc.round(calc.rem(d,60))} min
]

#let list_ingredients(ingredients) =  ingredients.map(((ingredient, amount)) =>
rect(stroke: none, width: 100%)[
    #format_weight(amount) #ingredient
  ]
)

// Meta-ingredients (subrecipes) carry a third element: whether they come from
// food prep. Those are marked so the cook uses the prepped batch instead of
// making them fresh.
#let list_meta(items) = items.map(((ingredient, amount, from_prep)) =>
rect(stroke: none, width: 100%)[
    #format_weight(amount) #ingredient
    #if from_prep [ #box(stroke: 0.5pt + orange, inset: (x: 3pt, y: 1pt), radius: 2pt, fill: orange.lighten(85%))[#text(size: 0.75em)[aus Food Prep]] ]
  ]
)

#let allergen_label_de = (
  "milch": "Milch", "gluten": "Gluten", "weizen": "Weizen", "eier": "Eier",
  "soja": "Soja", "erdnüsse": "Erdnüsse", "schalenfrüchte": "Schalenfrüchte",
  "sesamsamen": "Sesam", "sellerie": "Sellerie", "senf": "Senf",
  "lupine": "Lupine", "schwefeldioxid & sulfite": "Sulfite", "fisch": "Fisch",
  "schwein": "Schwein", "fleisch": "Fleisch", "milchprodukt": "Milchprodukt",
  "käse": "Käse", "ei-produkt": "Ei-Produkt", "krebstiere": "Krebstiere",
  "weichtiere": "Weichtiere", "gelatine": "Gelatine",
)

#let pretty_allergen(name) = {
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

#let allergen_block(allergens) = {
  block(stroke: 0.5pt + gray, inset: 6pt, radius: 4pt, width: 100%)[
    #dietary_badges(allergens.dietary)
    #v(0.3em)
    #if allergens.contains.len() > 0 [
      *Enthält:* #allergens.contains.map(pretty_allergen).join(", ")
    ] else [
      #text(style: "italic")[Keine deklarationspflichtigen Allergene.]
    ]
    #if allergens.may_contain.len() > 0 [
      \
      #text(size: 0.85em, style: "italic")[Kann Spuren von #allergens.may_contain.map(pretty_allergen).join(", ") enthalten.]
    ]
  ]
}

// A prepped-ahead recipe: banner naming the meals it feeds and each meal's
// share of the batch, so the kitchen knows how to portion the prep.
#let prep_banner(prep_for) = {
  block(stroke: 0.5pt + orange, inset: 6pt, radius: 4pt, width: 100%, fill: orange.lighten(85%))[
    *Vorbereitung (Food Prep)*
    #if prep_for.len() > 0 [
      \
      #text(size: 0.9em)[für: #prep_for.map(((name, pct)) => name + " (" + str(calc.round(pct)) + "%)").join(", ")]
    ]
  ]
}

// Boiling-water helper, rendered inline right below the recipe that uses it.
// `kg` is the scaled amount of water for this usage site. The boil time is
// computed here rather than read from a stored recipe step: heating water is
// physical, and the constants are calibrated for a 3.5 kW induction stove —
// ~4.2 min pot/heat-up offset + ~1.4 min per kg (matching the historical
// "4:12 fix + 1:24/kg" step of recipe 282). Kept as one small block so it can
// appear below every usage without forcing its own page.
#let water_boil_minutes(kg) = 4.2 + 1.4 * kg
// `extras` is a list of (name, kg) for non-water ingredients (e.g. salt in
// Salzwasser) so they are not lost when the recipe is folded inline.
#let water_note(name, kg, extras) = {
  v(0.4em)
  block(stroke: 0.5pt + blue, inset: 6pt, radius: 4pt, width: 100%, fill: blue.lighten(92%), breakable: false)[
    *#name:* #format_weight(kg) Wasser#for (ename, ekg) in extras [ + #format_weight(ekg) #ename] #h(1fr) #sym.arrow.r aufkochen \~#format_duration(water_boil_minutes(kg)) #text(size: 0.85em, fill: gray)[(3.5 kW)]
  ]
}

// Badge confirming every ingredient amount was weighed and verified accurate
// to the gram ("eingewogen"). Only shown once the recipe is explicitly marked.
#let weighed_in_badge() = {
  box(stroke: 0.5pt + green, inset: (x: 6pt, y: 3pt), radius: 3pt, fill: green.lighten(80%))[*Eingewogen* #text(size: 0.85em)[(Mengen grammgenau)]]
}

#let recipe(name, date, ingredients, steps, allergens, prep_for: none, weighed_in: false, water: (), root: none) = {
  [
    #show: project.with(
      title: name,
      date: date,
    )
    // When this page is a subrecipe of a larger dish, name that dish so the cook
    // knows what the component belongs to.
    #if root != none and root != name [
      #align(center)[#text(size: 0.9em, style: "italic")[Teil von: #root]]
      #v(0.5em)
    ]
    #if prep_for != none [ #prep_banner(prep_for) #v(0.5em) ]
    #if weighed_in [ #weighed_in_badge() #v(0.5em) ]
    #allergen_block(allergens)
    #v(0.5em)
    #rect()[
      #if ingredients.ingredients.len() > 0 [
      #grid(
        columns: (1fr, 1fr),
        ..list_ingredients(ingredients.ingredients)
      )]
      #if ingredients.subrecipes.len() > 0 [
        #line(length: 100%)
        #grid(
          columns: (1fr,1fr),
        ..list_meta(ingredients.subrecipes)
      )]
    ]
    #for step in steps [
      +  *#step.title* #h(1fr) #format_duration(step.duration.fix) fix + #format_duration(step.duration.var) \
        #step.desc
    ]
    // Boiling-water helpers inline on this recipe's page.
    #for (wname, wkg, wextras) in water [
      #water_note(wname, wkg, wextras)
    ]
  ]
}

