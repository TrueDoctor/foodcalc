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

#let dietary_badges(d) = {
  set text(size: 0.85em)
  let parts = ()
  if d.vegan { parts.push(box(stroke: 0.5pt + green, inset: 3pt, radius: 3pt, fill: green.lighten(80%))[*vegan*]) }
  else if d.vegetarian { parts.push(box(stroke: 0.5pt + olive, inset: 3pt, radius: 3pt, fill: olive.lighten(80%))[*vegetarisch*]) }
  if d.lactose_free { parts.push(box(stroke: 0.5pt + blue, inset: 3pt, radius: 3pt, fill: blue.lighten(80%))[*laktosefrei*]) }
  if d.gluten_free { parts.push(box(stroke: 0.5pt + purple, inset: 3pt, radius: 3pt, fill: purple.lighten(80%))[*glutenfrei*]) }
  if parts.len() == 0 { [] } else { stack(dir: ltr, spacing: 4pt, ..parts) }
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

#let recipe(name, date, ingredients, steps, allergens) = {
  [
    #show: project.with(
      title: name,
      date: date,
    )
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
        ..list_ingredients(ingredients.subrecipes)
      )]
    ]
    #for step in steps [
      +  *#step.title* #h(1fr) #format_duration(step.duration.fix) fix + #format_duration(step.duration.var) \
        #step.desc
    ]
  ]
}

