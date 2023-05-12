module Model exposing (..)

import Ingredients.Model exposing (IngredientMsg, IngredientTabData)
import Recipes.Model exposing (RecipeMsg, RecipeTabData)
import Utils.Cursor exposing (Cursor)


type alias Model =
    { tabs : Cursor Tab
    }


type Msg
    = None
    | ChangeTab Tab
    | IngredientMessage IngredientMsg
    | RecipeMessage RecipeMsg


type Tab
    = Ingredients IngredientTabData
    | Recipes RecipeTabData
    | Events
