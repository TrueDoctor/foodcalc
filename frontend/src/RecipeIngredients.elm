module RecipeIngredients exposing (..)

import Element exposing (..)
import Element.Input
import Http
import Json.Decode as Decode
import Json.Encode as Encode
import Test.SearchDropdown exposing (searchDropdown)
import Test.StringUtils as StringUtils
import WebData exposing (RemoteData(..), WebData)


type alias RecipeIngredient =
    { ingredient : Maybe MetaIngredient
    , unit : Maybe Unit
    , amount : String
    , allIngredients : { list : WebData (List MetaIngredient), search : String, hidden : Bool }
    , allUnits : { list : WebData (List Unit), search : String, hidden : Bool }
    }


type MetaIngredient
    = Ingredient
        { id : Int
        , name : String
        }
    | Subrecipe { id : Int, name : String }


type alias Unit =
    { id : Int
    , name : String
    }


type RecipeIngredientMsg
    = AmountChanged String
    | IngredientChanged MetaIngredient
    | IngredientFilterChange String
    | IngredientFocus
    | UnitChanged Unit
    | UnitFilterChange String
    | UnitFocus


viewRecipeIngredient : RecipeIngredient -> Element RecipeIngredientMsg
viewRecipeIngredient recipeIngredient =
    let
        nameOf =
            Maybe.withDefault ""
                << Maybe.map
                    (\a ->
                        case a of
                            Ingredient { name } ->
                                name

                            Subrecipe { name } ->
                                name
                    )

        ingredientsSettings =
            { select = IngredientChanged
            , itemName = nameOf << Just
            , filterChange = IngredientFilterChange
            , onFocus = IngredientFocus
            , title = "Ingredients"
            }

        unitSettings =
            { select = UnitChanged
            , itemName = .name
            , filterChange = UnitFilterChange
            , onFocus = UnitFocus
            , title = "Units"
            }
    in
    case ( recipeIngredient.allIngredients.list, recipeIngredient.allUnits.list ) of
        ( Success i, Success u ) ->
            row [ width fill, spacing 20 ]
                [ el [ width (fillPortion 3) ]
                    (searchDropdown ingredientsSettings
                        { search = recipeIngredient.allIngredients.search
                        , items = i
                        , selection = List.head (List.filter (\e -> Just e == recipeIngredient.ingredient) i)
                        , hidden = recipeIngredient.allIngredients.hidden
                        }
                    )
                , el [ width (fillPortion 3) ]
                    (Element.Input.text [ width fill ]
                        { label = Element.Input.labelAbove [] <| text "Amount"
                        , onChange = AmountChanged
                        , placeholder = Just <| Element.Input.placeholder [] (text "Amount")
                        , text = recipeIngredient.amount
                        }
                    )
                , el [ width (fillPortion 1) ]
                    (searchDropdown unitSettings
                        { search = recipeIngredient.allUnits.search
                        , items = u
                        , selection = List.head (List.filter (\e -> Just e == recipeIngredient.unit) u)
                        , hidden = recipeIngredient.allUnits.hidden
                        }
                    )
                ]

        _ ->
            row [ width fill, padding 20 ]
                [ el [ width (fillPortion 3) ] (text <| nameOf recipeIngredient.ingredient)
                , el [ width (fillPortion 3) ] (text recipeIngredient.amount)
                , el [ width (fillPortion 1) ] (text <| Maybe.withDefault "" <| Maybe.map .name recipeIngredient.unit)
                ]


updateRecipeIngredient : RecipeIngredientMsg -> RecipeIngredient -> RecipeIngredient
updateRecipeIngredient msg ri =
    let
        allUnits =
            ri.allUnits

        allIngredients =
            ri.allIngredients
    in
    case msg of
        AmountChanged s ->
            { ri | amount = s }

        IngredientChanged ingredient ->
            { ri | ingredient = Just ingredient, allIngredients = { allIngredients | hidden = True } }

        IngredientFocus ->
            { ri | allIngredients = { allIngredients | hidden = not allIngredients.hidden } }

        IngredientFilterChange search ->
            { ri | allIngredients = { allIngredients | search = search } }

        UnitFocus ->
            { ri | allUnits = { allUnits | hidden = not allUnits.hidden } }

        UnitFilterChange search ->
            { ri | allUnits = { allUnits | search = search } }

        UnitChanged unit ->
            { ri | unit = Just unit, allUnits = { allUnits | hidden = True } }



-- Decoding


decodeUnit : Decode.Decoder Unit
decodeUnit =
    Decode.map2 Unit
        (Decode.field "unit_id" Decode.int)
        (Decode.field "name" Decode.string)


decodeNestedWeightedMetaIngredients : Decode.Decoder (List RecipeIngredient)
decodeNestedWeightedMetaIngredients =
    Decode.list decodeNestedWeightedMetaIngredient


decodeNestedWeightedMetaIngredient : Decode.Decoder RecipeIngredient
decodeNestedWeightedMetaIngredient =
    Decode.map3
        (\i a u ->
            { ingredient = Just i
            , amount = a
            , unit = Just u
            , allIngredients = { list = NotAsked, search = "", hidden = True }
            , allUnits = { list = NotAsked, search = "", hidden = True }
            }
        )
        (Decode.field "ingredient" decodeMetaIngredient)
        (Decode.field "amount" Decode.string)
        (Decode.field "unit" decodeUnit)


decodeMetaIngredient : Decode.Decoder MetaIngredient
decodeMetaIngredient =
    Decode.oneOf
        [ Decode.field "Ingredient" <| Decode.map2 (\id name -> Ingredient { id = id, name = name }) (Decode.field "ingredient_id" Decode.int) (Decode.field "name" Decode.string)
        , Decode.field "MetaRecipe" <| Decode.map2 (\id name -> Subrecipe { id = id, name = name }) (Decode.field "recipe_id" Decode.int) (Decode.field "name" Decode.string)
        ]


decodeMetaIngredients : Decode.Decoder (List MetaIngredient)
decodeMetaIngredients =
    Decode.list decodeMetaIngredient



-- Fetching


fetchRecipeIngredients : (Result Http.Error (List RecipeIngredient) -> msg) -> Int -> Cmd msg
fetchRecipeIngredients msg recipeId =
    Http.get
        { url = "http://localhost:3000/recipes/" ++ String.fromInt recipeId ++ "/meta_ingredients/list"
        , expect = Http.expectJson msg decodeNestedWeightedMetaIngredients
        }


fetchAllMetaIngredients : (Result Http.Error (List MetaIngredient) -> msg) -> Cmd msg
fetchAllMetaIngredients msg =
    Http.get
        { url = "http://localhost:3000/recipes/meta_ingredients/list"
        , expect = Http.expectJson msg decodeMetaIngredients
        }


fetchUnits : (Result Http.Error (List Unit) -> msg) -> Cmd msg
fetchUnits msg =
    Http.get
        { url = "http://localhost:3000/utils/units"
        , expect = Http.expectJson msg (Decode.list decodeUnit)
        }



-- Encoding


encodeMeta : MetaIngredient -> Encode.Value
encodeMeta meta =
    case meta of
        Ingredient i ->
            Encode.object
                [ ( "Ingredient"
                  , Encode.object
                        [ ( "ingredient_id", Encode.int i.id )
                        , ( "name", Encode.string i.name )
                        , ( "energy", Encode.float 0 )
                        ]
                  )
                ]

        Subrecipe r ->
            Encode.object
                [ ( "MetaRecipe"
                  , Encode.object
                        [ ( "recipe_id", Encode.int r.id )
                        , ( "name", Encode.string r.name )
                        ]
                  )
                ]


encodeUnit : Unit -> Encode.Value
encodeUnit u =
    Encode.object
        [ ( "unit_id", Encode.int u.id )
        , ( "name", Encode.string u.name )
        ]


encodeRecipeIngredient : RecipeIngredient -> Maybe Encode.Value
encodeRecipeIngredient ri =
    case ( ri.ingredient, ri.unit ) of
        ( Just i, Just u ) ->
            String.toFloat ri.amount
                |> Maybe.map
                    (\_ ->
                        Encode.object
                            [ ( "ingredient", encodeMeta i )
                            , ( "amount", Encode.string ri.amount )
                            , ( "unit", encodeUnit u )
                            ]
                    )

        _ ->
            Nothing


encodeIngredients : RemoteData (List RecipeIngredient) e -> Maybe Encode.Value
encodeIngredients wd =
    case wd of
        Success list ->
            let
                encodes =
                    List.map encodeRecipeIngredient list
            in
            if List.all (Maybe.withDefault False << Maybe.map (always True)) encodes then
                Just <| Encode.list identity (List.filterMap encodeRecipeIngredient list)

            else
                Nothing

        _ ->
            Nothing
