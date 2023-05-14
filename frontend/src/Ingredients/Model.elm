module Ingredients.Model exposing (..)

import Http
import Utils.Model exposing (WebData)

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

type IngredientMsg
    = AddIngredient
    | EditIngredient Int
    | DeleteIngredient Int
    | CloseModal
    | ModalMsg ModalMsg
    | GotIngredients (Result Http.Error (List Ingredient))
    | IngredientChanged IngredientEditor
    | EditFilter String

type ModalMsg = EditName String | EditEnergy String | EditComment String

type Modal = Add IngredientEditor | Edit IngredientEditor | NoModal

type alias IngredientTabData =
    { ingredients : WebData (List Ingredient)
    , filter : String
    , modal: Modal
    }
