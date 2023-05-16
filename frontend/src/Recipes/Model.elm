module Recipes.Model exposing (..)

import Http
import Ingredients.Model exposing (Ingredient, IngredientMsg(..))
import Utils.Model exposing (Unit, WebData)
import Utils.Model exposing (RemoteData(..))


type alias Recipe =
    { id : Int
    , name : String
    , comment : Maybe String
    }


type alias RecipeEditor =
    { id : Maybe Int
    , name : String
    , comment : Maybe String
    , ingredients : WebData (List WeightedMetaIngredient)
    , activeIngredientIndex : Maybe Int
    , steps : WebData (List Step)
    , filter : String
    }

emptyRecipeEditor : RecipeEditor
emptyRecipeEditor =
    { id = Nothing
    , name = ""
    , comment = Nothing
    , ingredients = Success []
    , activeIngredientIndex = Nothing
    , steps = Success []
    , filter = ""
    }

editorFromReipe : Recipe -> RecipeEditor
editorFromReipe recipe =
    { id = Just recipe.id
    , name = recipe.name
    , comment = recipe.comment
    , ingredients = NotAsked
    , activeIngredientIndex = Nothing
    , steps = NotAsked
    , filter = ""
    }


type MetaIngredient
    = IsDirect Ingredient
    | IsSubRecipe Recipe


type alias WeightedMetaIngredient =
    { metaIngredient : MetaIngredient
    , amount : String
    , unit : Unit
    }


type alias Step =
    { id : Maybe Int
    , title : String
    , order : Float
    , description : String
    }


type RecipeWebData
    = RecipesData (Result Http.Error (List Recipe))
    | MetaIngredientData (Result Http.Error (List MetaIngredient))
    | RecipeIngredientData (Result Http.Error (List WeightedMetaIngredient))
    | RecipeId RecipeEditor (Result Http.Error Int)
    | PostResult (Result Http.Error ())


type RecipeMsg
    = AddRecipe
    | EditRecipe Int
    | DeleteRecipe Int
    | CloseModal
    | ModalMsg ModalMsg
    | GotWebData RecipeWebData
    | RecipeChanged RecipeEditor
    | EditFilter String
    | InitTab


type ModalMsg
    = AddIngredient
    | EditIngredient {old: Maybe WeightedMetaIngredient, new: Maybe WeightedMetaIngredient}
    | DeleteIngredient Int
    | EditName String
    | EditComment String
    | EditIngredientFilter String
    | EditActiveIngredientIndex Int
    | EditIngredientAmount String
    | EditIngredientUnit Unit


type Modal
    = NoModal
    | Add RecipeEditor
    | Edit RecipeEditor


type alias RecipeTabData =
    { recipes : WebData (List Recipe)
    , filter : String
    , modal : Modal
    , allIngredients : WebData (List MetaIngredient)
    }
