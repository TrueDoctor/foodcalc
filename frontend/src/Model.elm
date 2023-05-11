module Model exposing (..)

import Utils.Cursor exposing (Cursor)

import Ingredients.Model exposing ( IngredientMsg)
import Ingredients.Model exposing (IngredientTabData)


type alias Model =
    { tabs : Cursor Tab
    }


type Msg
    = None
    | ChangeTab Tab
    | IngredientMessage IngredientMsg






type Tab
    = Ingredients IngredientTabData
    | Recipes
    | Events
