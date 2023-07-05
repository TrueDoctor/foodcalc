module RecipesList exposing (..)

import Element exposing (..)
import Element.Background
import Element.Border
import Element.Input
import Http
import Json.Decode as Decode
import Platform.Cmd as Cmd
import Recipes.Model exposing (RecipeMsg)
import Test.ExpandableList as ExpandableList exposing (ExpandableList, ExpandableListMsg, mapElementMsg)
import Test.Styles exposing (white)
import WebData exposing (RemoteData(..), WebData)


type alias RecipesList =
    { recipes : WebData (ExpandableList Recipe RecipeListMsg RecipeMsg)
    , ingredients : WebData (List MetaIngredient)
    , units : WebData (List Unit)
    }


type Recipes
    = Recipes RecipesList


type alias RecipeData =
    { name : String
    , id : Maybe Int
    , comment : Maybe String
    , ingredients : WebData (List RecipeIngredient)
    }


type alias RecipeIngredient =
    { ingredientId : Int
    , ingredient : String
    , unitId : Int
    , unit : String
    , amount : String
    }


type alias MetaIngredient =
    { id : Int
    , name : String
    }


type alias Unit =
    { id : Int
    , name : String
    }


type Recipe
    = Recipe { data : RecipeData, edit : RecipeData }


type RecipeListMsg
    = ListMsg (ExpandableListMsg Recipe RecipeMsg)
    | GotWebData WebDataMsg


type WebDataMsg
    = GotRecipes (Result Http.Error (List Recipe))
    | GotRecipeIngredients Int (Result Http.Error (List RecipeIngredient))


type RecipeMsg
    = NameChange String
    | CommentChange String
    | RecipeIngredientChange
    | Save
    | Cancel
    | FetchIngredients


stateOf : String -> List ( Bool, Recipe ) -> WebData (ExpandableList Recipe RecipeListMsg RecipeMsg)
stateOf search items =
    let
        filter : String -> Recipe -> Bool
        filter string ingredient =
            case ingredient of
                Recipe { data } ->
                    String.contains (String.toLower string) (String.toLower data.name)
    in
    Success
        { search = search
        , filter = filter
        , items = items
        , viewElement = viewRecipe
        , mapMsg = ListMsg
        , update = updateRecipe
        , add =
            Just
                (\() ->
                    newRecipe Nothing "" (Just "") []
                )
        , expandItem =
            Just <|
                \recipe ->
                    case recipe of
                        Recipe { data } ->
                            Maybe.withDefault Cmd.none <| Maybe.map fetchRecipeIngredients data.id
        }


emptyRecipesData : RecipesList
emptyRecipesData =
    { recipes = NotAsked
    , ingredients = NotAsked
    , units = NotAsked
    }


newRecipe : Maybe Int -> String -> Maybe String -> List RecipeIngredient -> Recipe
newRecipe id name comment ingredients =
    let
        data =
            { id = id, name = name, comment = comment, ingredients = NotAsked }
    in
    Recipe { data = data, edit = data }


view : RecipesList -> Element RecipeListMsg
view model =
    case model.recipes of
        Success data ->
            ExpandableList.view data

        Failure _ ->
            el [] (text "Failed to load ingredients")

        _ ->
            el [] (text "Loading")


viewRecipe : Attribute RecipeListMsg -> Bool -> Recipe -> Element RecipeListMsg
viewRecipe expand expanded recipe =
    case recipe of
        Recipe { data } ->
            column [ width fill ]
                [ el [ expand, width fill ] (viewRow data)
                , if expanded then
                    viewExpanded recipe

                  else
                    none
                ]


viewRow : RecipeData -> Element msg
viewRow data =
    row [ spaceEvenly, width fill, paddingXY 50 20 ]
        [ el [ width (fillPortion 1) ] (text (Maybe.withDefault "" (Maybe.map String.fromInt data.id)))
        , el [ width (fillPortion 4) ] (text data.name)
        , el [ width (fillPortion 5) ] (text (Maybe.withDefault "" data.comment))
        ]


viewExpanded : Recipe -> Element RecipeListMsg
viewExpanded recipe =
    case recipe of
        Recipe { edit } ->
            column [ Element.Background.color white, width fill, padding 10, spacing 10, Element.Border.rounded 5 ]
                [ Element.map (ListMsg << ExpandableList.mapElementMsg recipe)
                    (Element.Input.text []
                        { onChange = NameChange
                        , label = Element.Input.labelLeft [] (text "Name:")
                        , placeholder = Just (Element.Input.placeholder [] (text "Name"))
                        , text = edit.name
                        }
                    )
                , Element.map
                    (ListMsg << ExpandableList.mapElementMsg recipe)
                    (Element.Input.text []
                        { onChange = CommentChange
                        , label = Element.Input.labelLeft [] (text "Comment:")
                        , placeholder = Just (Element.Input.placeholder [] (text "Comment"))
                        , text = Maybe.withDefault "" edit.comment
                        }
                    )
                , viewIngredients edit.ingredients
                , Element.map
                    (ListMsg << ExpandableList.mapElementMsg recipe)
                    (row [ width fill, spacing 25 ]
                        [ Element.Input.button [ alignRight ] { onPress = Just Save, label = el [ padding 10 ] <| text "Save" }
                        , Element.Input.button [ alignRight ] { onPress = Just Cancel, label = el [ padding 10 ] <| text "Cancel" }
                        ]
                    )
                ]


viewRecipeIngredient : RecipeIngredient -> Element RecipeListMsg
viewRecipeIngredient recipeIngredient =
    row [ width fill, padding 20 ]
        [ el [ width (fillPortion 3) ] (text recipeIngredient.ingredient)
        , el [ width (fillPortion 3) ] (text recipeIngredient.amount)
        , el [ width (fillPortion 1) ] (text recipeIngredient.unit)
        ]


viewIngredients : WebData (List RecipeIngredient) -> Element RecipeListMsg
viewIngredients wd =
    case wd of
        Failure _ ->
            text "Failure loading ingredients"

        Success list ->
            column [ width fill ]
                (List.map viewRecipeIngredient list)

        _ ->
            text "Loading ingredients"


replaceId : (( Bool, Recipe ) -> ( Bool, Recipe )) -> Int -> List ( Bool, Recipe ) -> List ( Bool, Recipe )
replaceId f id list =
    List.map
        (\( expanded, r ) ->
            case r of
                Recipe { data } ->
                    if data.id == Just id then
                        f ( expanded, r )

                    else
                        ( expanded, r )
        )
        list


updateRecipe : RecipeMsg -> Recipe -> ( Recipe, Cmd RecipeListMsg )
updateRecipe msg rc =
    let
        updateEdit f =
            case rc of
                Recipe i ->
                    let
                        edit =
                            i.edit
                    in
                    Recipe { i | edit = f edit }
    in
    case msg of
        NameChange name ->
            ( updateEdit (\e -> { e | name = name }), Cmd.none )

        CommentChange comment ->
            ( updateEdit (\e -> { e | comment = Just comment }), Cmd.none )

        Save ->
            ( rc, Cmd.none )

        Cancel ->
            case rc of
                Recipe { data } ->
                    ( Recipe { data = data, edit = data }, Cmd.none )

        RecipeIngredientChange ->
            ( rc, Cmd.none )

        FetchIngredients ->
            case rc of
                Recipe { data } ->
                    ( rc, Maybe.withDefault Cmd.none <| Maybe.map fetchRecipeIngredients data.id )


handleWebData : WebDataMsg -> RecipesList -> ( RecipesList, Cmd RecipeListMsg )
handleWebData msg model =
    case ( msg, model.recipes ) of
        ( GotRecipes result, Success list ) ->
            case result of
                Ok new ->
                    ( { model
                        | recipes = Success { list | items = List.map (Tuple.pair False) new }
                      }
                    , Cmd.none
                    )

                Err e ->
                    ( { model | recipes = Failure e }, Cmd.none )

        ( GotRecipes result, _ ) ->
            case result of
                Ok new ->
                    ( { model | recipes = stateOf "" <| List.map (Tuple.pair False) new }, Cmd.none )

                Err e ->
                    ( { model | recipes = Failure e }, Cmd.none )

        ( GotRecipeIngredients id result, Success list ) ->
            let
                ingredients =
                    case result of
                        Ok i ->
                            Success i

                        Err e ->
                            Failure e
            in
            ( { model
                | recipes =
                    Success
                        { list
                            | items =
                                replaceId
                                    (\( e, r ) ->
                                        case r of
                                            Recipe { data, edit } ->
                                                ( e
                                                , Recipe
                                                    { data = { data | ingredients = ingredients }
                                                    , edit = { edit | ingredients = ingredients }
                                                    }
                                                )
                                    )
                                    id
                                    list.items
                        }
              }
            , Cmd.none
            )

        ( GotRecipeIngredients _ _, _ ) ->
            ( model, Cmd.none )


update : RecipeListMsg -> RecipesList -> ( RecipesList, Cmd RecipeListMsg )
update msg model =
    case ( msg, model.recipes ) of
        ( ListMsg m, Success data ) ->
            Tuple.mapFirst
                (\result -> { model | recipes = Success result })
                (ExpandableList.update m data)

        ( GotWebData wd, _ ) ->
            handleWebData wd model

        _ ->
            ( model, Cmd.none )



-- Decoding


decodeUnit : Decode.Decoder Unit
decodeUnit =
    Decode.map2 Unit
        (Decode.field "unit_id" Decode.int)
        (Decode.field "name" Decode.string)


decodeRecipe : Decode.Decoder Recipe
decodeRecipe =
    Decode.map4 newRecipe
        (Decode.field "recipe_id" (Decode.nullable Decode.int))
        (Decode.field "name" Decode.string)
        (Decode.field "comment" (Decode.nullable Decode.string))
        (Decode.succeed [])


decodeNestedWeightedMetaIngredients : Decode.Decoder (List RecipeIngredient)
decodeNestedWeightedMetaIngredients =
    Decode.list decodeNestedWeightedMetaIngredient


decodeNestedWeightedMetaIngredient : Decode.Decoder RecipeIngredient
decodeNestedWeightedMetaIngredient =
    Decode.map3 (\i a u -> { ingredientId = i.id, ingredient = i.name, amount = a, unitId = u.id, unit = u.name })
        (Decode.field "ingredient" decodeMetaIngredient)
        (Decode.field "amount" Decode.string)
        (Decode.field "unit" decodeUnit)


decodeMetaIngredient : Decode.Decoder MetaIngredient
decodeMetaIngredient =
    let
        decodeMeta d =
            Decode.oneOf [ Decode.field "Ingredient" d, Decode.field "MetaRecipe" d ]
    in
    Decode.map2 MetaIngredient
        (decodeMeta (Decode.oneOf [ Decode.field "ingredient_id" Decode.int, Decode.field "recipe_id" Decode.int ]))
        (decodeMeta (Decode.field "name" Decode.string))


decodeMetaIngredients : Decode.Decoder (List MetaIngredient)
decodeMetaIngredients =
    Decode.list decodeMetaIngredient


decodeRecipes : Decode.Decoder (List Recipe)
decodeRecipes =
    Decode.list decodeRecipe


fetchRecipes : Cmd RecipeListMsg
fetchRecipes =
    Http.get
        { url = "http://localhost:3000/recipes/list"
        , expect = Http.expectJson (GotWebData << GotRecipes) decodeRecipes
        }


fetchRecipeIngredients recipeId =
    Http.get
        { url = "http://localhost:3000/recipes/" ++ String.fromInt recipeId ++ "/meta_ingredients/list"
        , expect = Http.expectJson (GotWebData << GotRecipeIngredients recipeId) decodeNestedWeightedMetaIngredients
        }
