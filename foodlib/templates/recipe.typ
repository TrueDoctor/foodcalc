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
  let first =  calc.ceil(calc.log(x))
  let y = x / calc.pow(10,first)
  let rounded = calc.round(digits: 4,y)
  calc.round(digits: 3,rounded * calc.pow(10,first)) 
}

#let format_weight(w) = if w < 1 [
   #significant_digits(w * 1000) g
] else [
  #significant_digits(w) kg
]

#let format_duration(d) = if d < 60 [
  #{d} min
] else [
  #{d/60} h #calc.rem(d,60) min
]

#let list_ingredients(ingredients) =  ingredients.map(((ingredient, amount)) => 
rect(stroke: none, width: 100%)[
    #format_weight(amount) #ingredient
  ]
)

#let recipe(name,date, ingredients, steps) = {
  [
    #show: project.with(
      title: name,
      date: date,
    )
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

