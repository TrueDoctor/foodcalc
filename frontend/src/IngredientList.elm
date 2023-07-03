module IngredientList exposing (..)

import Element exposing (..)
import Test.ExpandableList as ExpandableList exposing (ExpandableList, ExpandableListMsg)
import Element.Input


type alias IngredientsList =
    ExpandableList Ingredient IngredientListMsg IngredientMsg


type Ingredients
    = Ingredients IngredientsList


type Ingredient
    = Ingredient { name : String, id : Int, energy : String, comment : Maybe String }


type IngredientMsg
    = Save


type IngredientListMsg
    = ListMsg (ExpandableListMsg Ingredient IngredientMsg)


init : ExpandableList Ingredient IngredientListMsg IngredientMsg
init =
    let
        filter : String -> Ingredient -> Bool
        filter string ingredient =
            case ingredient of
                Ingredient { name } ->
                    String.contains (String.toLower string) (String.toLower name)
    in
    { search = ""
    , filter = filter
    , items = [(False, Ingredient {id=1, name="test", energy="1", comment = Nothing})]
    , viewElement = viewIngredient
    , mapMsg = ListMsg
    , update = \_ a -> ( Cmd.none, a )
    , add = Just (\() -> Ingredient { name = "", id = -1, energy = "", comment = Maybe.Nothing })
    }


viewExpanded ingredient =
    case ingredient of 
        Ingredient {name, comment, energy} -> 
            column [] [
                text "hi"
            ]
viewIngredient : Bool -> Ingredient -> Element IngredientListMsg
viewIngredient expanded ingredient =
    case ingredient of
        Ingredient { id, name, comment, energy } ->
            column [width fill]
                [ row [spaceEvenly, width fill]
                    [ el [width (fillPortion 1)] (text (String.fromInt id))
                    , el [width (fillPortion 4)] (text name)
                    , el [width (fillPortion 2)] (text energy)
                    , el [width (fillPortion 5)] (text (Maybe.withDefault "" comment))
                    ],
                    if expanded then viewExpanded ingredient else none
                ]


view : IngredientsList -> Element IngredientListMsg
view model =
    ExpandableList.view model

update: IngredientListMsg -> IngredientsList -> (IngredientsList, Cmd IngredientListMsg)
update msg model =
    case msg of
        ListMsg m ->
            ExpandableList.update m model