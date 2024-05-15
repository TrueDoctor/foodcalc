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

#let list_ingredients(ingredients) = for (ingredient,amount) in ingredients {
  rect(stroke: none, width: 100%)[
    #format_weight(amount) #ingredient
  ]
}

#let recipe(name,date, ingredients, steps) = {
  [
    #show: project.with(
      title: name,
      date: date,
    )
    #rect()[
      #grid( 
        columns: (1fr, 1fr),
        ..list_ingredients(ingredients.ingredients).children
      )
      #line(length: 100%)
      #grid(
        columns: (1fr,1fr),
      ..list_ingredients(ingredients.subrecipes).children)
    ]
    #for step in steps [
      +  *#step.title* #h(1fr) #format_duration(step.duration.fix) fix + #format_duration(step.duration.var) \
        #step.desc
    ]
  ]
}
