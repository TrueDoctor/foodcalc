module Recipes.Model exposing (..)

import Http
import Ingredients.Model exposing (IngredientMsg(..))
import Utils.Model exposing (WebData)


type alias Recipe =
    { id : Int
    , name : String
    , comment : Maybe String
    }


type alias RecipeEditor =
    { id : Maybe Int
    , name : String
    , comment : Maybe String
    , ingredient_ids : List RecipeIngredient
    , steps : List Step
    }


type alias RecipeIngredient =
    { ingredient_id : Int
    , amount : Float
    , unit : String
    }

type alias Step =
    { id : Maybe Int
    , title : String
    , order : Float
    , description : String
    }


type RecipeMsg
    = AddRecipe
    | EditRecipe Int
    | DeleteRecipe Int
    | CloseModal
    | ModalMsg ModalMsg
    | GotRecipes (Result Http.Error (List Recipe))
    | RecipeChanged RecipeEditor
    | EditFilter String
    | InitTab

type ModalMsg
    = AddIngredient
    | EditIngredient Int
    | DeleteIngredient Int
    | CloseIngredientModal
    | IngredientMsg IngredientMsg

type Modal
    = NoModal
    | Add RecipeEditor
    | Edit RecipeEditor

type alias RecipeTabData =
 {
    recipes: WebData (List Recipe)
    , filter: String
    , modal: Modal
    
 } 