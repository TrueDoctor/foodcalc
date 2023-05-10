module State exposing (..)
import Html exposing (a)
import Http
import Cursor exposing (Cursor)
import Json.Decode as Decode


type alias Model =
    { tabs: Cursor Tab
    }


type Msg
    = None
    | ChangeTab Tab
    | GotIngredients (Result Http.Error  (List Ingredient))


type RemoteData a e = 
    NotAsked
    | Loading
    | Success a
    | Failure e

type alias WebData a = RemoteData a Http.Error


type alias Ingredient
    = { name : String
        , energy : Float
      }


type Tab
    = Ingredients(WebData (List Ingredient))
    | Recipes
    | Events

