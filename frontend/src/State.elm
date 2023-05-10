module State exposing (..)

import Cursor exposing (Cursor)
import Http


type alias Model =
    { tabs : Cursor Tab
    }

type IngredientMsg
    = AddIngredient
    | EditIngredient Int
    | DeleteIngredient Int
    | GotIngredients (Result Http.Error (List Ingredient))
    | IngredientChanged Ingredient
    | EditFilter String

type Msg
    = None
    | ChangeTab Tab
    | IngredientMessage IngredientMsg


type RemoteData a e
    = NotAsked
    | Loading
    | Success a
    | Failure e


type alias WebData a =
    RemoteData a Http.Error


type alias Ingredient =
    { id : Int
    , name : String
    , energy : Float
    , comment : Maybe String
    }


type Tab
    = Ingredients { ingredients : WebData (List Ingredient), filter : String }
    | Recipes
    | Events
