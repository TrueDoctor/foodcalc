module Recipes.Model exposing (..)

import Http
import Ingredients.Model exposing (Ingredient, IngredientMsg(..))
import Utils.Model exposing (DropdownData, RemoteData(..), Unit, WebData, newDropdownData)


type alias Recipe =
    { id : Int
    , name : String
    , comment : Maybe String
    }


type alias RecipeEditor =
    { id : Maybe Int
    , name : String
    , comment : Maybe String
    , ingredients : WebData (List ( WeightedMetaIngredient, RecipeIngredientEditor ))
    , steps : WebData (List Step)
    }


type alias RecipeIngredientEditor =
    { ingredientDropdown : DropdownData MetaIngredient
    , unitDropdown : DropdownData Unit
    , amountInput : String
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

type MetaId = IngredientId Int | SubRecipeId Int | NewId

type ModalMsg
    = EditName String
    | EditComment String
    | EditMetaIngredient MetaId RecipeIngredientMsg 
    | EditStep StepMsg Int


type StepMsg
    = SetTitle String
    | SetOrder String
    | SetDescription String
    | DeleteStep


type RecipeIngredientMsg 
    = SetIngredient MetaIngredient
    | SetAmount String
    | SetUnit Unit
    | SetIngredientFilter String
    | SetUnitFilter String
    | Delete


type Modal
    = NoModal
    | Add RecipeEditor
    | Edit RecipeEditor


type alias RecipeTabData =
    { recipes : WebData (List Recipe)
    , filter : String
    , modal : Modal
    , allIngredients : WebData (List MetaIngredient)
    , allUnits : WebData (List Unit)
    }


buildEditor : List MetaIngredient -> List Unit -> WeightedMetaIngredient -> RecipeIngredientEditor
buildEditor ingredientList unitList ingredient =
    { amountInput = ""
    , unitDropdown = newDropdownData unitList ingredient.unit
    , ingredientDropdown = newDropdownData ingredientList ingredient.metaIngredient
    }


emptyRecipeEditor : RecipeEditor
emptyRecipeEditor =
    { id = Nothing
    , name = ""
    , comment = Nothing
    , ingredients = Success []
    , steps = Success []
    }


editorFromReipe : Recipe -> RecipeEditor
editorFromReipe recipe =
    { id = Just recipe.id
    , name = recipe.name
    , comment = recipe.comment
    , ingredients = NotAsked
    , steps = NotAsked
    }

emptyRecipeTabData : RecipeTabData
emptyRecipeTabData =
    { recipes = NotAsked
    , filter = ""
    , modal = NoModal
    , allIngredients = NotAsked
    , allUnits = NotAsked
    }