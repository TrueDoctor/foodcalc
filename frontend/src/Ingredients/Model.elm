module Ingredients.Model exposing (..)

import Http
import Utils.Model exposing (WebData)
import Utils.Model exposing (RemoteData(..))


type alias Ingredient =
    { id : Int
    , name : String
    , energy : Float
    , comment : Maybe String
    }


type alias IngredientEditor =
    { id : Maybe Int
    , name : String
    , energy : String
    , comment : String
    }


type IngredientWebData
    = IngredientsList (Result Http.Error (List Ingredient))
    | SuccessfulPost (Result Http.Error Int)


type IngredientMsg
    = AddIngredient
    | EditIngredient Int
    | DeleteIngredient Int
    | CloseModal
    | ModalMsg ModalMsg
    | GotWebData IngredientWebData
    | EditFilter String
    | InitTab


type ModalMsg
    = EditName String
    | EditEnergy String
    | EditComment String
    | Save IngredientEditor


type Modal
    = Add IngredientEditor
    | Edit IngredientEditor
    | NoModal


type alias IngredientTabData =
    { ingredients : WebData (List Ingredient)
    , filter : String
    , modal : Modal
    }


-- INIT
emptyIngredientsTabData : IngredientTabData
emptyIngredientsTabData =
    { ingredients = NotAsked
    , filter = ""
    , modal = NoModal
    }