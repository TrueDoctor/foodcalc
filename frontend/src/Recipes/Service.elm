module Recipes.Service exposing (..)

import Http
import Ingredients.Model exposing (IngredientEditor)
import Ingredients.Service exposing (decodeIngredient, encodeIngredient)
import Json.Decode as Decode
import Json.Encode as Encode
import Recipes.Model exposing (..)
import Utils.Decoding exposing (decodeUnit, encodeUnit, maybe)
import Utils.Model exposing (RemoteData(..), WebData)



-- Decoding


decodeRecipe : Decode.Decoder Recipe
decodeRecipe =
    Decode.map3 Recipe
        (Decode.field "recipe_id" Decode.int)
        (Decode.field "name" Decode.string)
        (Decode.field "comment" (Decode.nullable Decode.string))


decodeNestedWeightedMetaIngredients : Decode.Decoder (List WeightedMetaIngredient)
decodeNestedWeightedMetaIngredients =
    Decode.list decodeNestedWeightedMetaIngredient


decodeNestedWeightedMetaIngredient : Decode.Decoder WeightedMetaIngredient
decodeNestedWeightedMetaIngredient =
    Decode.map3 WeightedMetaIngredient
        (Decode.field "ingredient" decodeMetaIngredient)
        (Decode.field "amount" Decode.string)
        (Decode.field "unit" decodeUnit)


decodeMetaIngredient : Decode.Decoder MetaIngredient
decodeMetaIngredient =
    Decode.oneOf
        [ Decode.map IsSubRecipe <| Decode.field "MetaRecipe" decodeRecipe
        , Decode.map IsDirect <| Decode.field "Ingredient" decodeIngredient
        ]


decodeMetaIngredients : Decode.Decoder (List MetaIngredient)
decodeMetaIngredients =
    Decode.list decodeMetaIngredient


decodeRecipes : Decode.Decoder (List Recipe)
decodeRecipes =
    Decode.list decodeRecipe



-- Encoding


encodeRecipeEditor : RecipeEditor -> Encode.Value
encodeRecipeEditor editor =
    Encode.object
        [ ( "recipe_id", maybe Encode.int editor.id )
        , ( "name", Encode.string editor.name )
        , ( "comment", maybe Encode.string editor.comment )
        ]


encodeMetaIngredients : WebData (List WeightedMetaIngredient) -> Encode.Value
encodeMetaIngredients ingredients =
    case ingredients of
        Success i ->
            Encode.list encodeWeightedMetaIngredient i

        _ ->
            Encode.null


encodeWeightedMetaIngredient : WeightedMetaIngredient -> Encode.Value
encodeWeightedMetaIngredient ingredient =
    Encode.object
        [ ( "ingredient", encodeMetaIngredient ingredient.metaIngredient )
        , ( "amount", Encode.string (if ingredient.amount=="" then "0" else ingredient.amount) )
        , ( "unit", encodeUnit ingredient.unit )
        ]


encodeMetaIngredient : MetaIngredient -> Encode.Value
encodeMetaIngredient ingredient =
    case ingredient of
        IsSubRecipe recipe ->
            Encode.object
                [ ( "MetaRecipe", encodeRecipe recipe )
                ]

        IsDirect i ->
            Encode.object
                [ ( "Ingredient"
                  , Encode.object
                        [ ( "ingredient_id", Encode.int i.id )
                        , ( "name", Encode.string i.name )
                        , ( "comment", maybe Encode.string i.comment )
                        , ( "energy", Encode.float i.energy )
                        ]
                  )
                ]


encodeRecipe : Recipe -> Encode.Value
encodeRecipe recipe =
    Encode.object
        [ ( "recipe_id", Encode.int recipe.id )
        , ( "name", Encode.string recipe.name )
        , ( "comment", maybe Encode.string recipe.comment )
        ]


encodeSteps : WebData (List Step) -> Encode.Value
encodeSteps steps =
    case steps of
        Success s ->
            Encode.list encodeStep s

        _ ->
            Encode.null


encodeStep : Step -> Encode.Value
encodeStep step =
    Encode.object
        [ ( "step_id", maybe Encode.int step.id )
        , ( "title", Encode.string step.title )
        , ( "description", Encode.string step.description )
        , ( "order", Encode.float step.order )
        ]



-- Http requests


fetchRecipes : Cmd RecipeMsg
fetchRecipes =
    Http.get
        { url = "http://localhost:3000/recipes/list"
        , expect = Http.expectJson (GotWebData << RecipesData) decodeRecipes
        }


fetchAllMetaIngredients : Cmd RecipeMsg
fetchAllMetaIngredients =
    Http.get
        { url = "http://localhost:3000/recipes/meta_ingredients/list"
        , expect = Http.expectJson (GotWebData << MetaIngredientData) decodeMetaIngredients
        }


fetchRecipeIngredients : Int -> Cmd RecipeMsg
fetchRecipeIngredients recipeId =
    Http.get
        { url = "http://localhost:3000/recipes/" ++ String.fromInt recipeId ++ "/meta_ingredients/list"
        , expect = Http.expectJson (GotWebData << RecipeIngredientData) decodeNestedWeightedMetaIngredients
        }


updateRecipeEditor : String -> RecipeEditor -> Cmd RecipeMsg
updateRecipeEditor url editor =
    Http.post
        { url = "http://localhost:3000/recipes" ++ url
        , body = Http.jsonBody <| encodeRecipeEditor editor
        , expect = Http.expectJson (GotWebData << RecipeId editor) Decode.int
        }


updateRecipeIngredients : RecipeEditor -> Int -> Cmd RecipeMsg
updateRecipeIngredients editor id =
    Http.post
        { url = "http://localhost:3000/recipes/" ++ String.fromInt id ++ "/meta_ingredients/update"
        , body = Http.jsonBody <| encodeMetaIngredients editor.ingredients
        , expect = Http.expectJson (GotWebData << PostResult) (Decode.succeed ())
        }


updateRecipeSteps : RecipeEditor -> Int -> Cmd RecipeMsg
updateRecipeSteps editor id =
    Http.post
        { url = "http://localhost:3000/recipes/" ++ String.fromInt id ++ "/steps/update"
        , body = Http.jsonBody <| encodeSteps editor.steps
        , expect = Http.expectJson (GotWebData << PostResult) (Decode.succeed ())
        }


updateRecipeExtras : RecipeEditor -> Int -> Cmd RecipeMsg
updateRecipeExtras editor id =
    Cmd.batch
        [ updateRecipeIngredients editor id
        , updateRecipeSteps editor id
        ]


addOrUpdateRecipe : Modal -> Cmd RecipeMsg
addOrUpdateRecipe modal =
    case modal of
        Add editor ->
            updateRecipeEditor "/create" editor

        Edit editor ->
            case editor.id of
                Just id ->
                    updateRecipeEditor ("/" ++ String.fromInt id ++ "/update") editor

                Nothing ->
                    Cmd.none

        _ ->
            Cmd.none
