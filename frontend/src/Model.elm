module Model exposing (..)

import Ingredients.Model exposing (IngredientMsg, IngredientTabData)
import Recipes.Model exposing (RecipeMsg, RecipeTabData)
import Utils.Cursor exposing (Cursor)
import Events exposing (EventsData)
import Events exposing (EventTabMsg)
import IngredientList exposing (IngredientListMsg)


type alias Model =
    { tabs : Cursor Tab
    , ingredients : IngredientTabData
    , recipes : RecipeTabData
    , events : EventsData
    , ingredientList: IngredientList.IngredientsList
    }


type Msg
    = None
    | ChangeTab Tab
    | IngredientMessage IngredientMsg
    | RecipeMessage RecipeMsg
    | EventsMessage EventTabMsg
    | IngredientUIMsg IngredientListMsg


type Tab
    = Ingredients IngredientTabData
    | Recipes RecipeTabData
    | Events
